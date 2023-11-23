use crate::control_flow::{
	nodes::Nodes,
	nodes_mut::{Var, ViewMut},
};

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

	fn find_set_bonds<N: Nodes>(&mut self, nodes: &N, set: &[usize]) {
		self.point_in.clear();
		self.point_out.clear();

		for id in nodes.iter() {
			if nodes.predecessors(id).any(|id| !set.contains(&id)) {
				self.point_in.push(id);
			}

			if nodes.successors(id).any(|id| !set.contains(&id)) {
				self.point_out.push(id);
			}
		}
	}

	fn restructure_continues<N: ViewMut>(&mut self, nodes: &mut N, set: &[usize], handler: usize) {
		// Predecessor -> Entry
		// Predecessor -> Destination -> Repetition -> Selection -> Entry
		for (index, &id) in self.point_in.iter().enumerate() {
			self.vec_usize
				.extend(nodes.predecessors(id).filter(|id| set.contains(id)));

			for predecessor in self.vec_usize.drain(..) {
				let repetition = nodes.add_assignment(Var::B, 1, handler);
				let destination = nodes.add_assignment(Var::A, index, repetition);

				nodes.replace_link(predecessor, id, destination);
			}
		}
	}

	fn restructure_start<N: ViewMut>(&mut self, nodes: &mut N, set: &[usize]) -> usize {
		let selection = nodes.add_selection(Var::A);

		// Predecessor -> Entry
		// Predecessor -> Destination -> Selection -> Entry
		for (index, &entry) in self.point_in.iter().enumerate() {
			let assignment = nodes.add_assignment(Var::A, index, selection);

			nodes.add_link(selection, entry);

			self.vec_usize
				.extend(nodes.predecessors(entry).filter(|id| !set.contains(id)));

			for predecessor in self.vec_usize.drain(..) {
				nodes.replace_link(predecessor, entry, assignment);
			}
		}

		selection
	}

	fn restructure_end<N: ViewMut>(
		&mut self,
		nodes: &mut N,
		set: &[usize],
		handler: usize,
	) -> usize {
		let selection = nodes.add_selection(Var::A);

		// Exit -> Successor
		// Exit -> Destination -> Repetition -> Selection -> Successor
		for (index, &exit) in self.point_out.iter().enumerate() {
			self.vec_usize
				.extend(nodes.successors(exit).filter(|id| !set.contains(id)));

			for successor in self.vec_usize.drain(..) {
				let repetition = nodes.add_assignment(Var::B, 0, handler);
				let destination = nodes.add_assignment(Var::A, index, repetition);

				nodes.add_link(selection, successor);
				nodes.replace_link(exit, successor, destination);
			}
		}

		selection
	}

	pub fn restructure<N: ViewMut>(&mut self, nodes: &mut N, set: &[usize]) -> usize {
		let handler = nodes.add_selection(Var::B);

		self.find_set_bonds(nodes, set);
		self.restructure_continues(nodes, set, handler);

		let start = self.restructure_start(nodes, set);
		let end = self.restructure_end(nodes, set, handler);

		nodes.add_link(handler, end);
		nodes.add_link(handler, start);

		start
	}
}

impl Default for Repeat {
	fn default() -> Self {
		Self::new()
	}
}
