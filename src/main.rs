use std::{iter::Copied, slice::Iter};

use perfect_reconstructibility::{
	control_flow::{
		nodes::Nodes,
		nodes_mut::{Var, ViewMut},
	},
	restructurer::linear::Linear,
};

struct Instruction {}

struct Node {
	predecessors: Vec<usize>,
	successors: Vec<usize>,
	instructions: Vec<Instruction>,
}

struct Slice<'nodes> {
	nodes: &'nodes mut [Node],
	set: Vec<usize>,
}

impl<'nodes> Nodes for Slice<'nodes> {
	type Predecessors<'iter> = Copied<Iter<'iter, usize>> where 'nodes: 'iter;
	type Successors<'iter> = Copied<Iter<'iter, usize>> where 'nodes: 'iter;
	type Iter<'iter> = Copied<Iter<'iter, usize>> where 'nodes: 'iter;
	type View<'parent> = Slice<'parent> where 'nodes: 'parent;

	fn predecessors(&self, id: usize) -> Self::Predecessors<'_> {
		self.nodes[id].predecessors.iter().copied()
	}

	fn successors(&self, id: usize) -> Self::Successors<'_> {
		self.nodes[id].successors.iter().copied()
	}

	fn iter(&self) -> Self::Iter<'_> {
		self.set.iter().copied()
	}

	fn view<I: IntoIterator<Item = usize>>(&mut self, set: I) -> Self::View<'_> {
		let set: Vec<_> = set.into_iter().collect();

		debug_assert!(set.iter().all(|id| self.set.contains(id)));

		Self::View {
			nodes: &mut *self.nodes,
			set,
		}
	}
}

impl<'nodes> ViewMut for Slice<'nodes> {
	fn remove_node(&mut self, id: usize) {
		let index = self.set.binary_search(&id).unwrap();

		self.set.remove(index);
	}

	fn add_selection(&mut self, var: Var) -> usize {
		todo!()
	}

	fn add_assignment(&mut self, var: Var, value: usize, successor: usize) -> usize {
		todo!()
	}

	fn add_link(&mut self, from: usize, to: usize) {
		todo!()
	}

	fn replace_link(&mut self, from: usize, to: usize, new: usize) {
		todo!()
	}
}

fn main() {
	let mut linear = Linear::new();
	// let mut nodes = Slice {};

	// linear.restructure(&mut nodes);
}
