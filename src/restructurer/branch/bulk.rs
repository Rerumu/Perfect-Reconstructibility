// Resources:
// "Perfect Reconstructability of Control Flow from Demand Dependence Graphs",
//     by Helge Bahmann, Google Zurich, Nico Reissmann, Magnus Jahre, and Jan Christian Meyer

use crate::{
	collection::set::Set,
	control_flow::{Nodes, NodesMut},
};

use super::single::{Branch, Single};

pub struct Bulk {
	single: Single,

	set: Set,
	branches: Vec<(Set, usize)>,
}

impl Bulk {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			single: Single::new(),

			set: Set::new(),
			branches: Vec::new(),
		}
	}

	fn find_branch_head<N: Nodes>(&mut self, nodes: &N, mut start: usize) -> Option<usize> {
		loop {
			let mut successors = nodes.successors(start);
			let successor = successors.next()?;

			if !self.set.get(successor) {
				return None;
			}

			if successors.next().is_some() {
				return Some(start);
			}

			self.set.remove(start);

			start = successor;
		}
	}

	fn restructure_branch<N: NodesMut>(&mut self, nodes: &mut N, head: usize) {
		let exit = self.single.restructure(nodes, self.set.as_slice(), head);
		let tail = std::mem::take(self.single.tail_mut());

		self.branches.push((tail, exit));

		let iter = self.single.branches_mut().drain(..).filter_map(|branch| {
			if let Branch::Full { items, start } = branch {
				Some((items, start))
			} else {
				None
			}
		});

		self.branches.extend(iter);
	}

	pub fn restructure<N: NodesMut>(&mut self, nodes: &mut N, set: &mut Set, mut start: usize) {
		self.set.clone_from(set);

		loop {
			if let Some(head) = self.find_branch_head(nodes, start) {
				self.restructure_branch(nodes, head);

				set.extend(self.single.insertions().iter().copied());
			}

			if let Some((set, next)) = self.branches.pop() {
				self.set.clone_from(&set);

				start = next;
			} else {
				break;
			}
		}
	}
}
