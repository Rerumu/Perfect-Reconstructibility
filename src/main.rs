use std::{iter::Copied, slice::Iter};

use perfect_reconstructibility::{
	control_flow::{Nodes, NodesMut, Var},
	restructurer::linear::Linear,
};

enum Instruction {
	SetInteger { var: u16, value: i64 },

	Move { from: u16, to: u16 },

	Add { lhs: u16, rhs: u16, to: u16 },
	Sub { lhs: u16, rhs: u16, to: u16 },
	Mul { lhs: u16, rhs: u16, to: u16 },
	Div { lhs: u16, rhs: u16, to: u16 },
	Mod { lhs: u16, rhs: u16, to: u16 },

	Selection { var: u16 },
}

struct Node {
	predecessors: Vec<usize>,
	successors: Vec<usize>,
	instructions: Vec<Instruction>,
}

struct Slice<'nodes> {
	nodes: &'nodes [Node],
	set: Vec<usize>,
}

impl<'nodes> Nodes for Slice<'nodes> {
	fn predecessors(&self, id: usize) -> Copied<Iter<'_, usize>> {
		self.nodes[id].predecessors.iter().copied()
	}

	fn successors(&self, id: usize) -> Copied<Iter<'_, usize>> {
		self.nodes[id].successors.iter().copied()
	}

	fn iter(&self) -> Copied<Iter<'_, usize>> {
		self.set.iter().copied()
	}

	fn contains(&self, id: usize) -> bool {
		self.set.binary_search(&id).is_ok()
	}

	fn view<I: IntoIterator<Item = usize>>(&self, set: I) -> Slice<'_> {
		let result = Slice {
			nodes: self.nodes,
			set: set.into_iter().collect(),
		};

		debug_assert!(
			result.set.iter().all(|id| self.set.contains(id)),
			"set contains invalid ids"
		);

		result
	}
}

struct SliceMut<'nodes> {
	nodes: &'nodes mut Vec<Node>,
	set: Vec<usize>,
}

impl<'nodes> Nodes for SliceMut<'nodes> {
	fn predecessors(&self, id: usize) -> Copied<Iter<'_, usize>> {
		self.nodes[id].predecessors.iter().copied()
	}

	fn successors(&self, id: usize) -> Copied<Iter<'_, usize>> {
		self.nodes[id].successors.iter().copied()
	}

	fn iter(&self) -> Copied<Iter<'_, usize>> {
		self.set.iter().copied()
	}

	fn contains(&self, id: usize) -> bool {
		self.set.binary_search(&id).is_ok()
	}

	fn view<I: IntoIterator<Item = usize>>(&self, set: I) -> Slice<'_> {
		let result = Slice {
			nodes: self.nodes,
			set: set.into_iter().collect(),
		};

		debug_assert!(
			result.set.iter().all(|id| self.set.contains(id)),
			"set contains invalid ids"
		);

		debug_assert!(
			result.set.windows(2).all(|w| w[0] < w[1]),
			"set is not sorted"
		);

		result
	}
}

const fn to_var_index(var: Var) -> u16 {
	let offset = match var {
		Var::A => 0,
		Var::B => 1,
		Var::C => 2,
	};

	u16::MAX - offset
}

impl<'nodes> NodesMut for SliceMut<'nodes> {
	fn add_selection(&mut self, var: Var) -> usize {
		let id = self.nodes.len();

		self.nodes.push(Node {
			predecessors: vec![],
			successors: vec![],
			instructions: vec![Instruction::Selection {
				var: to_var_index(var),
			}],
		});

		self.set.push(id);

		id
	}

	fn add_assignment(&mut self, var: Var, value: usize, successor: usize) -> usize {
		let id = self.nodes.len();

		self.nodes.push(Node {
			predecessors: vec![],
			successors: vec![successor],
			instructions: vec![Instruction::SetInteger {
				var: to_var_index(var),
				value: value as i64,
			}],
		});

		self.set.push(id);

		id
	}

	fn exclude_node(&mut self, id: usize) {
		let index = self.set.binary_search(&id).unwrap();

		self.set.remove(index);
	}

	fn insert_link(&mut self, from: usize, to: usize) {
		self.nodes[from].successors.push(to);
		self.nodes[to].predecessors.push(from);
	}

	fn replace_link(&mut self, from: usize, to: usize, new: usize) {
		let predecessor = self.nodes[from]
			.predecessors
			.iter_mut()
			.find(|&&mut id| id == to)
			.unwrap();

		*predecessor = new;

		let successor = self.nodes[to]
			.successors
			.iter_mut()
			.find(|&&mut id| id == from)
			.unwrap();

		*successor = new;

		self.nodes[new].predecessors.push(from);
	}

	fn view_mut<I: IntoIterator<Item = usize>>(&mut self, set: I) -> SliceMut<'_> {
		let result = SliceMut {
			nodes: &mut *self.nodes,
			set: set.into_iter().collect(),
		};

		debug_assert!(
			result.set.iter().all(|id| self.set.contains(id)),
			"set contains invalid ids"
		);

		debug_assert!(
			result.set.windows(2).all(|w| w[0] < w[1]),
			"set is not sorted"
		);

		result
	}
}

fn main() {
	let mut linear = Linear::new();
	let mut nodes = SliceMut {
		nodes: &mut Vec::new(),
		set: vec![],
	};

	linear.restructure(&mut nodes);
}
