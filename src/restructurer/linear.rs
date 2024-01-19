use crate::control_flow::NodesMut;

use super::{branch::Branch, repeat::Repeat, strongly_connected_finder::StronglyConnectedFinder};

pub struct Linear {
	strongly_connected_finder: StronglyConnectedFinder,
	repeat_restructurer: Repeat,
	branch_restructurer: Branch,

	vec_usize: Vec<usize>,
}

impl Linear {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			strongly_connected_finder: StronglyConnectedFinder::new(),
			repeat_restructurer: Repeat::new(),
			branch_restructurer: Branch::new(),

			vec_usize: Vec::new(),
		}
	}

	pub fn restructure<N: NodesMut>(&mut self, nodes: &mut N) {
		let start = self.repeat_restructurer.restructure(nodes);

		nodes.exclude_node(start);

		let strong = self.strongly_connected_finder.run(nodes);

		for scc in std::mem::take(strong) {
			self.restructure(&mut nodes.view_mut(scc));
		}

		self.vec_usize.clear();
		self.vec_usize.extend(nodes.successors(start));

		for &successor in &self.vec_usize {
			self.branch_restructurer.restructure(nodes, successor);
		}
	}
}
