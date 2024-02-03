use crate::{
	collection::set::Slice,
	control_flow::{Nodes, NodesMut, Var},
};

/// A Repeat `R` is structured when it is all of the following:
/// - In `R`, there exists `A`, the single entry point.
/// - In `R`, there may exist `B`, the single exit point.
/// - `A` has a single predecessor, which is `B` if it exists, or any node in `R` otherwise.
///
/// Let `E` be the set of entry points and `X` be the set of exit points.
///
/// - If `|E| > 1`, let `A` be a new selection, funnel all predecessors of `E` not in `R` to `A`, and funnel `A` to `E`.
/// - If `|X| > 1`, let `B` be a new selection, funnel all successors of `X` not in `R` to `B`, and funnel `B` to all successors of `X` not in `R`.
/// - If `A` has a predecessor in `R` that is not `B`, let `D` be a new selection,
/// funnel relevant predecessors of `A` to `D`,
/// funnel all predecessors of `B` to `D`,
/// fork `D` to `A` and `B`.
#[derive(Default)]
pub struct Single {
	point_in: Vec<usize>,
	point_out: Vec<usize>,

	insertions: Vec<usize>,
}

impl Single {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			point_in: Vec::new(),
			point_out: Vec::new(),

			insertions: Vec::new(),
		}
	}

	fn find_set_bonds<N: Nodes>(&mut self, nodes: &N, set: Slice) -> (&[usize], &[usize]) {
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

		self.point_in.sort_unstable();
		self.point_out.sort_unstable();

		(&self.point_in, &self.point_out)
	}

	fn find_start_if_structured<N: Nodes>(&mut self, nodes: &N, set: Slice) -> Option<usize> {
		let (point_in, point_out) = self.find_set_bonds(nodes, set);

		if point_in.len() > 1 || point_out.len() > 1 {
			return None;
		}

		let start = point_in.first().copied().expect("entry point should exist");
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

				self.insertions.push(destination);
				self.insertions.push(repetition);
			}
		}
	}

	fn restructure_start<N: NodesMut>(&mut self, nodes: &mut N, set: Slice) -> usize {
		let selection = nodes.add_selection(Var::Destination);

		self.insertions.push(selection);

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

				self.insertions.push(destination);
			}

			nodes.add_link(selection, entry);
		}

		selection
	}

	fn restructure_end<N: NodesMut>(&mut self, nodes: &mut N, set: Slice, latch: usize) -> usize {
		let selection = nodes.add_selection(Var::Destination);

		self.insertions.push(selection);

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

				self.insertions.push(destination);
				self.insertions.push(repetition);
			}
		}

		selection
	}

	#[must_use]
	pub fn insertions(&self) -> &[usize] {
		&self.insertions
	}

	pub fn restructure<N: NodesMut>(&mut self, nodes: &mut N, set: Slice) -> usize {
		if let Some(start) = self.find_start_if_structured(nodes, set) {
			self.insertions.clear();

			return start;
		}

		let latch = nodes.add_selection(Var::Repetition);

		self.insertions.clear();
		self.insertions.push(latch);

		let start = if let &[start] = self.point_in.as_slice() {
			start
		} else {
			self.restructure_start(nodes, set)
		};

		let end = if let &[end] = self.point_out.as_slice() {
			end
		} else {
			self.restructure_end(nodes, set, latch)
		};

		self.restructure_continues(nodes, set, latch);

		nodes.add_link(latch, end);
		nodes.add_link(latch, start);

		start
	}
}
