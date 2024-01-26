// Resources:
// "Perfect Reconstructability of Control Flow from Demand Dependence Graphs",
//     by Helge Bahmann, Google Zurich, Nico Reissmann, Magnus Jahre, and Jan Christian Meyer

use crate::control_flow::NodesMut;

use super::{
	analysis::strongly_connected_finder::StronglyConnectedFinder,
	branch::{Branch, Element},
	repeat::Repeat,
};

pub struct Linear {
	strongly_connected_finder: StronglyConnectedFinder,
	repeat_restructurer: Repeat,
	branch_restructurer: Branch,

	vec_branches: Vec<(Vec<usize>, usize)>,
	vec_components: Vec<Vec<usize>>,
}

impl Linear {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			strongly_connected_finder: StronglyConnectedFinder::new(),
			repeat_restructurer: Repeat::new(),
			branch_restructurer: Branch::new(),

			vec_branches: Vec::new(),
			vec_components: Vec::new(),
		}
	}

	fn restructure_repeats<N: NodesMut>(&mut self, nodes: &mut N) {
		loop {
			let components = self.strongly_connected_finder.run(nodes);

			self.vec_components.append(components);

			if let Some(nested) = self.vec_components.pop() {
				nodes.set_included(nested);

				let start = self.repeat_restructurer.restructure(nodes);

				nodes.add_excluded([start]);
			} else {
				break;
			}
		}
	}

	fn restructure_branch<N: NodesMut>(&mut self, nodes: &mut N, start: usize) {
		let branches = self.branch_restructurer.restructure(nodes, start);
	}

	pub fn restructure<N: NodesMut>(&mut self, nodes: &mut N, start: usize) {
		self.restructure_repeats(nodes);
		self.restructure_branch(nodes, start);
	}
}
