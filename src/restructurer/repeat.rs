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

	fn find_set_bonds<N: Nodes>(&mut self, nodes: &N) {
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
	}

	fn restructure_continues<N: NodesMut>(&mut self, nodes: &mut N, handler: usize) {
		// Predecessor -> Entry
		// Predecessor -> Destination -> Repetition -> Selection -> Entry
		for (index, &id) in self.point_in.iter().enumerate() {
			let predecessors = nodes.predecessors(id).filter(|&id| nodes.contains(id));

			self.vec_usize.clear();
			self.vec_usize.extend(predecessors);

			for &predecessor in &self.vec_usize {
				let repetition = nodes.add_assignment(Var::B, 1, handler);
				let destination = nodes.add_assignment(Var::A, index, repetition);

				nodes.replace_link(predecessor, id, destination);
			}
		}
	}

	fn restructure_start<N: NodesMut>(&mut self, nodes: &mut N) -> usize {
		let selection = nodes.add_selection(Var::A);

		// Predecessor -> Entry
		// Predecessor -> Destination -> Selection -> Entry
		for (index, &entry) in self.point_in.iter().enumerate() {
			let assignment = nodes.add_assignment(Var::A, index, selection);

			nodes.insert_link(selection, entry);

			let predecessors = nodes.predecessors(entry).filter(|&id| !nodes.contains(id));

			self.vec_usize.clear();
			self.vec_usize.extend(predecessors);

			for &predecessor in &self.vec_usize {
				nodes.replace_link(predecessor, entry, assignment);
			}
		}

		selection
	}

	fn restructure_end<N: NodesMut>(&mut self, nodes: &mut N, handler: usize) -> usize {
		let selection = nodes.add_selection(Var::A);

		// Exit -> Successor
		// Exit -> Destination -> Repetition -> Selection -> Successor
		for (index, &exit) in self.point_out.iter().enumerate() {
			let successors = nodes.successors(exit).filter(|&id| !nodes.contains(id));

			self.vec_usize.clear();
			self.vec_usize.extend(successors);

			for &successor in &self.vec_usize {
				let repetition = nodes.add_assignment(Var::B, 0, handler);
				let destination = nodes.add_assignment(Var::A, index, repetition);

				nodes.insert_link(selection, successor);
				nodes.replace_link(exit, successor, destination);
			}
		}

		selection
	}

	pub fn restructure<N: NodesMut>(&mut self, nodes: &mut N) -> usize {
		let handler = nodes.add_selection(Var::B);

		self.find_set_bonds(nodes);
		self.restructure_continues(nodes, handler);

		let start = self.restructure_start(nodes);
		let end = self.restructure_end(nodes, handler);

		nodes.insert_link(handler, end);
		nodes.insert_link(handler, start);

		start
	}
}

impl Default for Repeat {
	fn default() -> Self {
		Self::new()
	}
}
