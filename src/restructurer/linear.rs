// Resources:
// "Perfect Reconstructability of Control Flow from Demand Dependence Graphs",
//     by Helge Bahmann, Google Zurich, Nico Reissmann, Magnus Jahre, and Jan Christian Meyer

use crate::{collection::set::Set, control_flow::NodesMut};

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

	fn restructure_repeats<N: NodesMut>(&mut self, nodes: &mut N, set: &mut Set) {
		loop {
			let components = self.strongly_connected_finder.run(nodes, set.as_slice());

			self.vec_components.append(components);

			if let Some(nested) = self.vec_components.pop() {
				set.clear();
				set.extend(nested);

				let start = self.repeat_restructurer.restructure(nodes, set.as_slice());

				set.remove(start);
			} else {
				break;
			}
		}
	}

	fn restructure_branch<N: NodesMut>(&mut self, nodes: &mut N, set: &mut Set, mut start: usize) {
		loop {
			let branches = self.branch_restructurer.restructure(nodes, set, start);
			let iter = branches.drain(..).filter_map(|element| {
				if let Element::Full { items, start } = element {
					Some((items, start))
				} else {
					None
				}
			});

			self.vec_branches.extend(iter);

			if let Some((elements, next)) = self.vec_branches.pop() {
				set.clear();
				set.extend(elements);

				start = next;
			} else {
				break;
			}
		}
	}

	pub fn restructure<N: NodesMut>(&mut self, nodes: &mut N, set: &mut Set, start: usize) {
		self.restructure_repeats(nodes, set);
		self.restructure_branch(nodes, set, start);
	}
}
