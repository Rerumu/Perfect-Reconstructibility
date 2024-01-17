use std::{iter::Copied, slice::Iter};

use perfect_reconstructibility::{
	control_flow::{Nodes, NodesMut, Var},
	restructurer::linear::Linear,
};

struct Instruction {}

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
	nodes: &'nodes mut [Node],
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

		result
	}
}

impl<'nodes> NodesMut for SliceMut<'nodes> {
	fn add_selection(&mut self, var: Var) -> usize {
		todo!()
	}

	fn add_assignment(&mut self, var: Var, value: usize, successor: usize) -> usize {
		todo!()
	}

	fn remove_node(&mut self, id: usize) {
		let index = self.set.binary_search(&id).unwrap();

		self.set.remove(index);
	}

	fn add_link(&mut self, from: usize, to: usize) {
		todo!()
	}

	fn replace_link(&mut self, from: usize, to: usize, new: usize) {
		todo!()
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

		result
	}
}

fn main() {
	let mut linear = Linear::new();
	let mut nodes = SliceMut {
		nodes: &mut [],
		set: vec![],
	};

	linear.restructure(&mut nodes);
}
