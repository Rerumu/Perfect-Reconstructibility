use crate::control_flow::{Nodes, NodesMut, Var};

pub struct Repeat {
	point_in: Vec<usize>,
	point_out: Vec<usize>,

	vec_usize: Vec<usize>,
}

impl Repeat {
	pub const fn new() -> Self {
		Self {
			point_in: Vec::new(),
			point_out: Vec::new(),

			vec_usize: Vec::new(),
		}
	}

	fn find_set_bonds<N: Nodes>(&mut self, nodes: &N) -> (&[usize], &[usize]) {
		self.point_in.clear();
		self.point_out.clear();

		for id in nodes.iter() {
			if nodes.predecessors(id).any(|id| !nodes.contains(id)) {
				self.point_in.push(id);
			}

			if nodes.successors(id).any(|id| !nodes.contains(id)) {
				self.point_out.push(id);
			}
		}

		assert!(!self.point_in.is_empty(), "no entry points found");

		(&self.point_in, &self.point_out)
	}

	fn find_start_if_structured<N: Nodes>(&mut self, nodes: &N) -> Option<usize> {
		let (point_in, point_out) = self.find_set_bonds(nodes);

		if point_in.len() > 1 || point_out.len() > 1 {
			return None;
		}

		let start = point_in.first().copied().expect("nodes should be an SCC");
		let repeats = nodes
			.predecessors(start)
			.filter(|&predecessor| nodes.contains(predecessor))
			.count();

		(repeats == 1).then_some(start)
	}

	fn restructure_continues<N: NodesMut>(&mut self, nodes: &mut N, latch: usize) {
		// Predecessor -> Entry
		// Predecessor -> Destination -> Repetition -> Latch -> Selection -> Entry
		for (index, &entry) in self.point_in.iter().enumerate() {
			let predecessors = nodes.predecessors(entry).filter(|&id| nodes.contains(id));

			self.vec_usize.clear();
			self.vec_usize.extend(predecessors);

			for &predecessor in &self.vec_usize {
				let destination = nodes.add_variable(Var::Destination, index);
				let repetition = nodes.add_variable(Var::Repetition, 1);

				nodes.remove_link(predecessor, entry);
				nodes.add_link(predecessor, destination);
				nodes.add_link(destination, repetition);
				nodes.add_link(repetition, latch);
			}
		}
	}

	fn restructure_start<N: NodesMut>(&mut self, nodes: &mut N) -> usize {
		let selection = nodes.add_selection(Var::Destination);

		// Predecessor -> Entry
		// Predecessor -> Destination -> Selection -> Entry
		for (index, &entry) in self.point_in.iter().enumerate() {
			let predecessors = nodes.predecessors(entry).filter(|&id| !nodes.contains(id));

			self.vec_usize.clear();
			self.vec_usize.extend(predecessors);

			for &predecessor in &self.vec_usize {
				let destination = nodes.add_variable(Var::Destination, index);

				nodes.remove_link(predecessor, entry);
				nodes.add_link(predecessor, destination);
				nodes.add_link(destination, selection);
			}

			nodes.add_link(selection, entry);
		}

		selection
	}

	fn restructure_end<N: NodesMut>(&mut self, nodes: &mut N, latch: usize) -> usize {
		let selection = nodes.add_selection(Var::Destination);

		// Exit -> Successor
		// Exit -> Destination -> Repetition -> Latch -> Selection -> Successor
		for (index, &exit) in self.point_out.iter().enumerate() {
			let successors = nodes.successors(exit).filter(|&id| !nodes.contains(id));

			self.vec_usize.clear();
			self.vec_usize.extend(successors);

			for &successor in &self.vec_usize {
				let destination = nodes.add_variable(Var::Destination, index);
				let repetition = nodes.add_variable(Var::Repetition, 0);

				nodes.remove_link(exit, successor);
				nodes.add_link(selection, successor);

				nodes.add_link(exit, destination);
				nodes.add_link(destination, repetition);
				nodes.add_link(repetition, latch);
			}
		}

		selection
	}

	pub fn restructure<N: NodesMut>(&mut self, nodes: &mut N) -> usize {
		if let Some(start) = self.find_start_if_structured(nodes) {
			return start;
		}

		let latch = nodes.add_selection(Var::Repetition);

		self.restructure_continues(nodes, latch);

		let start = self.restructure_start(nodes);
		let end = self.restructure_end(nodes, latch);

		nodes.add_link(latch, end);
		nodes.add_link(latch, start);

		start
	}
}

impl Default for Repeat {
	fn default() -> Self {
		Self::new()
	}
}
