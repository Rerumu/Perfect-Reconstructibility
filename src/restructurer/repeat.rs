use crate::{
	collection::set::Slice,
	control_flow::{Nodes, NodesMut, Var},
};

#[derive(Default)]
pub struct Repeat {
	point_in: Vec<usize>,
	point_out: Vec<usize>,
}

impl Repeat {
	pub const fn new() -> Self {
		Self {
			point_in: Vec::new(),
			point_out: Vec::new(),
		}
	}

	fn find_set_bonds<N: Nodes>(&mut self, nodes: &N, set: Slice) -> (&[usize], &[usize]) {
		self.point_in.clear();
		self.point_out.clear();

		for id in set.iter_ones() {
			if nodes.predecessors(id).any(|id| !set.get(id)) {
				self.point_in.push(id);
			}

			if nodes.successors(id).any(|id| !set.get(id)) {
				self.point_out.push(id);
			}
		}

		assert!(!self.point_in.is_empty(), "no entry points found");

		self.point_in.sort_unstable();
		self.point_out.sort_unstable();

		(&self.point_in, &self.point_out)
	}

	fn find_start_if_structured<N: Nodes>(&mut self, nodes: &N, set: Slice) -> Option<usize> {
		let (point_in, point_out) = self.find_set_bonds(nodes, set);

		if point_in.len() > 1 || point_out.len() > 1 {
			return None;
		}

		let start = point_in.first().copied().expect("nodes should be an SCC");
		let repeats = nodes.predecessors(start).filter(|&id| set.get(id)).count();

		(repeats == 1).then_some(start)
	}

	fn restructure_continues<N: NodesMut>(&mut self, nodes: &mut N, set: Slice, latch: usize) {
		// Predecessor -> Entry
		// Predecessor -> Destination -> Repetition -> Latch -> Selection -> Entry
		for (index, &entry) in self.point_in.iter().enumerate() {
			let predecessors: Vec<_> = nodes
				.predecessors(entry)
				.filter(|&id| set.get(id))
				.collect();

			for predecessor in predecessors {
				let destination = nodes.add_variable(Var::Destination, index);
				let repetition = nodes.add_variable(Var::Repetition, 1);

				nodes.replace_link(predecessor, entry, destination);
				nodes.add_link(destination, repetition);
				nodes.add_link(repetition, latch);
			}
		}
	}

	fn restructure_start<N: NodesMut>(&mut self, nodes: &mut N, set: Slice) -> usize {
		let selection = nodes.add_selection(Var::Destination);

		// Predecessor -> Entry
		// Predecessor -> Destination -> Selection -> Entry
		for (index, &entry) in self.point_in.iter().enumerate() {
			let predecessors: Vec<_> = nodes
				.predecessors(entry)
				.filter(|&id| !set.get(id))
				.collect();

			for predecessor in predecessors {
				let destination = nodes.add_variable(Var::Destination, index);

				nodes.replace_link(predecessor, entry, destination);
				nodes.add_link(destination, selection);
			}

			nodes.add_link(selection, entry);
		}

		selection
	}

	fn restructure_end<N: NodesMut>(&mut self, nodes: &mut N, set: Slice, latch: usize) -> usize {
		let selection = nodes.add_selection(Var::Destination);

		// Exit -> Successor
		// Exit -> Destination -> Repetition -> Latch -> Selection -> Successor
		for (index, &exit) in self.point_out.iter().enumerate() {
			let successors: Vec<_> = nodes.successors(exit).filter(|&id| !set.get(id)).collect();

			for successor in successors {
				let destination = nodes.add_variable(Var::Destination, index);
				let repetition = nodes.add_variable(Var::Repetition, 0);

				nodes.replace_link(exit, successor, destination);
				nodes.add_link(selection, successor);

				nodes.add_link(destination, repetition);
				nodes.add_link(repetition, latch);
			}
		}

		selection
	}

	pub fn restructure<N: NodesMut>(&mut self, nodes: &mut N, set: Slice) -> usize {
		if let Some(start) = self.find_start_if_structured(nodes, set) {
			return start;
		}

		let latch = nodes.add_selection(Var::Repetition);

		self.restructure_continues(nodes, set, latch);

		let start = self.restructure_start(nodes, set);
		let end = self.restructure_end(nodes, set, latch);

		nodes.add_link(latch, end);
		nodes.add_link(latch, start);

		start
	}
}
