use crate::{
	collection::set::{Set, Slice},
	control_flow::{Nodes, NodesMut, Var},
};

use super::analysis::dominator_finder::DominatorFinder;

pub enum Element {
	Full { items: Vec<usize>, start: usize },
	Empty { tail: usize },
}

pub struct Branch {
	branches: Vec<Element>,
	tails: Vec<usize>,

	dominator_finder: DominatorFinder,
}

impl Branch {
	pub const fn new() -> Self {
		Self {
			branches: Vec::new(),
			tails: Vec::new(),

			dominator_finder: DominatorFinder::new(),
		}
	}

	fn find_branch_head<N: Nodes>(nodes: &N, set: &mut Set, mut start: usize) -> usize {
		loop {
			let mut successors = nodes.successors(start);

			if let (Some(next), None) = (successors.next(), successors.next()) {
				set.remove(start);

				start = next;
			} else {
				break;
			}
		}

		start
	}

	fn initialize_fields<N: Nodes>(&mut self, nodes: &N, head: usize) {
		let successors = nodes.successors(head).count();
		let construct = || Element::Empty { tail: usize::MAX };

		self.branches.clear();
		self.branches.resize_with(successors, construct);

		self.tails.clear();
	}

	fn find_branch_elements<N: Nodes>(&mut self, nodes: &N, set: Slice, head: usize) {
		// Elements with only head as predecessor are full, otherwise empty
		for (branch, start) in self.branches.iter_mut().zip(nodes.successors(head)) {
			if nodes.predecessors(start).all(|id| id == head) {
				*branch = Element::Full {
					items: Vec::new(),
					start,
				};
			} else {
				*branch = Element::Empty { tail: start };
			}
		}

		// Find all nodes dominated by the branch start
		'dominated: for id in set.iter_ones() {
			for branch in &mut self.branches {
				if let Element::Full { items, start } = branch {
					if self.dominator_finder.is_dominator_of(*start, id) {
						items.push(id);

						continue 'dominated;
					}
				}
			}

			self.tails.push(id);
		}

		// Sort all items in the branches, this allows for faster than dominators search
		for branch in &mut self.branches {
			if let Element::Full { items, .. } = branch {
				debug_assert!(!items.is_empty(), "branches should not be empty");

				items.sort_unstable();
			}
		}

		self.tails.sort_unstable();
	}

	fn restructure_full<N: NodesMut>(&mut self, nodes: &mut N, items: &[usize], exit: usize) {
		let mut predecessors = Vec::new();

		// Find all tail connections
		for &tail in &self.tails {
			let exits = nodes.predecessors(tail).filter_map(|predecessor| {
				items
					.binary_search(&predecessor)
					.map(|_| (predecessor, tail))
					.ok()
			});

			predecessors.extend(exits);
		}

		// If there is more than one tail connection, add a funnel
		let funnel = if predecessors.len() == 1 {
			exit
		} else {
			let temp = nodes.add_no_operation();

			nodes.add_link(temp, exit);

			temp
		};

		// Replace all tail connections with the funnel
		for (predecessor, tail) in predecessors {
			let variable = self.tails.binary_search(&tail).unwrap();
			let destination = nodes.add_variable(Var::Branch, variable);

			nodes.replace_link(predecessor, tail, destination);
			nodes.add_link(destination, funnel);
		}
	}

	fn restructure_empty<N: NodesMut>(
		&mut self,
		nodes: &mut N,
		head: usize,
		tail: usize,
		exit: usize,
	) {
		let variable = self.tails.binary_search(&tail).unwrap();
		let destination = nodes.add_variable(Var::Branch, variable);

		nodes.replace_link(head, tail, destination);
		nodes.add_link(destination, exit);
	}

	fn restructure_branches<N: NodesMut>(&mut self, nodes: &mut N, head: usize) {
		let exit = nodes.add_selection(Var::Branch);

		for &tail in &self.tails {
			nodes.add_link(exit, tail);
		}

		let branches = std::mem::take(&mut self.branches);

		for branch in &branches {
			match *branch {
				Element::Full { ref items, .. } => self.restructure_full(nodes, items, exit),
				Element::Empty { tail } => self.restructure_empty(nodes, head, tail, exit),
			}
		}

		self.branches = branches;
	}

	pub fn restructure<N: NodesMut>(
		&mut self,
		nodes: &mut N,
		set: &mut Set,
		start: usize,
	) -> &mut Vec<Element> {
		let head = Self::find_branch_head(nodes, set, start);

		self.dominator_finder.run(nodes, set.as_slice(), head);

		set.remove(head);

		self.initialize_fields(nodes, head);
		self.find_branch_elements(nodes, set.as_slice(), head);

		if self.tails.len() != 1 {
			self.restructure_branches(nodes, head);
		}

		&mut self.branches
	}
}

impl Default for Branch {
	fn default() -> Self {
		Self::new()
	}
}
