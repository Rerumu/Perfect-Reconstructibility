use crate::control_flow::NodesMut;

use super::analysis::dominator_finder::DominatorFinder;

pub struct Branch {
	branches: Vec<usize>,

	dominator_finder: DominatorFinder,
}

impl Branch {
	pub const fn new() -> Self {
		Self {
			branches: Vec::new(),

			dominator_finder: DominatorFinder::new(),
		}
	}

	pub fn restructure<N: NodesMut>(&mut self, nodes: &mut N, mut start: usize) -> &mut Vec<usize> {
		loop {
			let mut successors = nodes.successors(start);

			if let (Some(next), None) = (successors.next(), successors.next()) {
				start = next;
			} else {
				break;
			}
		}

		self.branches.clear();
		self.dominator_finder.run(nodes, start);

		// TODO: Branches just contain the most dominated paths; it could be trivially
		// implemented by just counting references in one pass... Except for loops.

		// But this should also fail or return nothing when called on a non branch, what about
		// its loop counterpart?

		&mut self.branches
	}
}

impl Default for Branch {
	fn default() -> Self {
		Self::new()
	}
}
