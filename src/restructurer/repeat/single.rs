use crate::{
	collection::set::Slice,
	control_flow::{Nodes, NodesMut, Var},
};

/// This structure implements a single pass of this algorithm. It assumes that the set
/// provided is a strongly connected component and that there is at least one edge
/// from outside the set coming in.
#[derive(Default)]
pub struct Single {
	point_in: Vec<usize>,
	point_out: Vec<usize>,

	synthetics: Vec<usize>,
}

impl Single {
	/// Creates a new instance of the restructurer.
	#[must_use]
	pub const fn new() -> Self {
		Self {
			point_in: Vec::new(),
			point_out: Vec::new(),

			synthetics: Vec::new(),
		}
	}

	fn find_ins_and_outs<N: Nodes>(&mut self, nodes: &N, set: Slice) {
		self.point_in.clear();
		self.point_out.clear();

		for id in set.ones() {
			if nodes.predecessors(id).any(|id| !set.get(id)) {
				self.point_in.push(id);
			}

			if nodes.successors(id).any(|id| !set.get(id)) {
				self.point_out.push(id);
			}
		}
	}

	fn find_start_if_structured<N: Nodes>(&mut self, nodes: &N, set: Slice) -> Option<usize> {
		self.find_ins_and_outs(nodes, set);

		if let &[start] = self.point_in.as_slice() {
			if self.point_out.len() <= 1
				&& nodes.predecessors(start).filter(|&id| set.get(id)).count() == 1
			{
				return Some(start);
			}
		}

		None
	}

	fn restructure_continues<N: NodesMut>(&mut self, nodes: &mut N, set: Slice, latch: usize) {
		// Predecessor -> Entry
		// Predecessor -> Destination -> Repetition -> Latch -> Selection -> Entry
		for (index, &entry) in self.point_in.iter().enumerate() {
			let destination = nodes.add_variable(Var::Destination, index);
			let repetition = nodes.add_variable(Var::Repetition, 1);

			let predecessors: Vec<_> = nodes
				.predecessors(entry)
				.filter(|&id| set.get(id))
				.collect();

			for predecessor in predecessors {
				nodes.replace_link(predecessor, entry, destination);
			}

			nodes.add_link(destination, repetition);
			nodes.add_link(repetition, latch);

			self.synthetics.push(destination);
			self.synthetics.push(repetition);
		}
	}

	fn restructure_start<N: NodesMut>(&mut self, nodes: &mut N, set: Slice) -> usize {
		let selection = nodes.add_selection(Var::Destination);

		self.synthetics.push(selection);

		// Predecessor -> Entry
		// Predecessor -> Destination -> Selection -> Entry
		for (index, &entry) in self.point_in.iter().enumerate() {
			let destination = nodes.add_variable(Var::Destination, index);
			let predecessors: Vec<_> = nodes
				.predecessors(entry)
				.filter(|&id| !set.get(id))
				.collect();

			for predecessor in predecessors {
				nodes.replace_link(predecessor, entry, destination);
			}

			nodes.add_link(destination, selection);
			nodes.add_link(selection, entry);

			self.synthetics.push(destination);
		}

		selection
	}

	fn restructure_end<N: NodesMut>(&mut self, nodes: &mut N, set: Slice, latch: usize) -> usize {
		let selection = nodes.add_selection(Var::Destination);

		self.synthetics.push(selection);

		// Exit -> Successor
		// Exit -> Destination -> Repetition -> Latch -> Selection -> Successor
		for (index, &exit) in self.point_out.iter().enumerate() {
			let destination = nodes.add_variable(Var::Destination, index);
			let repetition = nodes.add_variable(Var::Repetition, 0);
			let successors: Vec<_> = nodes.successors(exit).filter(|&id| !set.get(id)).collect();

			for successor in successors {
				nodes.replace_link(exit, successor, destination);
				nodes.add_link(selection, successor);
			}

			nodes.add_link(destination, repetition);
			nodes.add_link(repetition, latch);

			self.synthetics.push(destination);
			self.synthetics.push(repetition);
		}

		selection
	}

	/// Returns the synthetic nodes created during the restructuring.
	#[must_use]
	pub fn synthetics(&self) -> &[usize] {
		&self.synthetics
	}

	/// Applies the restructuring algorithm to the given set of nodes.
	/// The start node of the structured repetition is returned.
	pub fn restructure<N: NodesMut>(&mut self, nodes: &mut N, set: Slice) -> usize {
		if let Some(start) = self.find_start_if_structured(nodes, set) {
			self.synthetics.clear();

			return start;
		}

		let latch = nodes.add_selection(Var::Repetition);

		self.synthetics.clear();
		self.synthetics.push(latch);

		let start = if let &[start] = self.point_in.as_slice() {
			start
		} else {
			self.restructure_start(nodes, set)
		};

		let end = self.restructure_end(nodes, set, latch);

		self.restructure_continues(nodes, set, latch);

		nodes.add_link(latch, start);
		nodes.add_link(latch, end);

		start
	}
}
