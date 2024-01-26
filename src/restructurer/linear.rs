// Resources:
// "Perfect Reconstructability of Control Flow from Demand Dependence Graphs",
//     by Helge Bahmann, Google Zurich, Nico Reissmann, Magnus Jahre, and Jan Christian Meyer

use crate::control_flow::{Nodes, NodesMut};

use super::{
	analysis::strongly_connected_finder::StronglyConnectedFinder, branch::Branch, repeat::Repeat,
};

pub struct Linear {
	strongly_connected_finder: StronglyConnectedFinder,
	repeat_restructurer: Repeat,
	branch_restructurer: Branch,

	vec_components: Vec<Vec<usize>>,
}

impl Linear {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			strongly_connected_finder: StronglyConnectedFinder::new(),
			repeat_restructurer: Repeat::new(),
			branch_restructurer: Branch::new(),

			vec_components: Vec::new(),
		}
	}

	fn find_next_component<N: Nodes>(&mut self, nodes: &N) -> Option<Vec<usize>> {
		let components = self.strongly_connected_finder.run(nodes);

		self.vec_components.append(components);
		self.vec_components.pop()
	}

	fn restructure_repeats<N: NodesMut>(&mut self, nodes: &mut N) {
		while let Some(nested) = self.find_next_component(nodes) {
			nodes.set_included(nested);

			let start = self.repeat_restructurer.restructure(nodes);

			nodes.add_excluded([start]);
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
