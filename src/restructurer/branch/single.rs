use crate::{
	collection::set::{Set, Slice},
	control_flow::{Nodes, NodesMut, Var},
};

use super::dominator_finder::DominatorFinder;

pub enum Branch {
	Full { items: Set, start: usize },
	Empty { tail: usize },
}

#[derive(Default)]
pub struct Single {
	tail: Set,
	continuations: Vec<usize>,
	branches: Vec<Branch>,

	insertions: Vec<usize>,
	dominator_finder: DominatorFinder,
}

impl Single {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			tail: Set::new(),
			continuations: Vec::new(),
			branches: Vec::new(),

			insertions: Vec::new(),
			dominator_finder: DominatorFinder::new(),
		}
	}

	fn initialize_fields<N: Nodes>(&mut self, nodes: &N, head: usize) {
		let successors = nodes.successors(head).count();
		let construct = || Branch::Empty { tail: usize::MAX };

		self.tail.clear();
		self.continuations.clear();

		self.branches.clear();
		self.branches.resize_with(successors, construct);

		self.insertions.clear();
	}

	fn initialize_branches<N: Nodes>(&mut self, nodes: &N, head: usize) {
		// Elements with only head as predecessor are full, otherwise empty
		for (branch, start) in self.branches.iter_mut().zip(nodes.successors(head)) {
			*branch = if nodes.predecessors(start).all(|id| id == head) {
				Branch::Full {
					items: Set::new(),
					start,
				}
			} else {
				Branch::Empty { tail: start }
			}
		}
	}

	fn find_branch_elements<N: Nodes>(&mut self, nodes: &N, set: Slice, head: usize) {
		self.dominator_finder.run(nodes, set, head);

		// Find all nodes dominated by the branch start
		'dominated: for id in set.ones() {
			for branch in &mut self.branches {
				if let Branch::Full { items, start } = branch {
					if self.dominator_finder.is_dominator_of(*start, id) {
						items.insert(id);

						continue 'dominated;
					}
				}
			}

			self.tail.insert(id);
		}

		self.tail.remove(head);

		for tail in self.tail.ones() {
			if nodes.predecessors(tail).any(|id| !self.tail.get(id)) {
				self.continuations.push(tail);
			}
		}
	}

	fn patch_single_tail(&mut self, tail: usize) {
		for branch in &mut self.branches {
			if let Branch::Full { items, .. } = branch {
				items.insert(tail);
			}
		}
	}

	fn restructure_full<N: NodesMut>(&mut self, nodes: &mut N, items: &mut Set, exit: usize) {
		let mut predecessors = Vec::new();

		// Find all tail connections
		for &tail in &self.continuations {
			let exits = nodes
				.predecessors(tail)
				.filter_map(|predecessor| items.get(predecessor).then_some((predecessor, tail)));

			predecessors.extend(exits);
		}

		// If there is more than one tail connection, add a funnel
		let funnel = if predecessors.len() == 1 {
			exit
		} else {
			let temp = nodes.add_no_operation();

			nodes.add_link(temp, exit);

			items.insert(temp);
			self.insertions.push(temp);

			temp
		};

		// Replace all tail connections with the funnel
		for (predecessor, tail) in predecessors {
			let variable = self.continuations.binary_search(&tail).unwrap();
			let destination = nodes.add_variable(Var::Branch, variable);

			nodes.replace_link(predecessor, tail, destination);
			nodes.add_link(destination, funnel);

			items.insert(destination);
			self.insertions.push(destination);
		}
	}

	fn restructure_empty<N: NodesMut>(
		&mut self,
		nodes: &mut N,
		head: usize,
		tail: usize,
		exit: usize,
	) {
		let variable = self.continuations.binary_search(&tail).unwrap();
		let destination = nodes.add_variable(Var::Branch, variable);

		nodes.replace_link(head, tail, destination);
		nodes.add_link(destination, exit);

		self.insertions.push(destination);
	}

	fn restructure_branches<N: NodesMut>(&mut self, nodes: &mut N, head: usize) -> usize {
		let exit = nodes.add_selection(Var::Branch);

		self.tail.insert(exit);
		self.insertions.push(exit);

		for &tail in &self.continuations {
			nodes.add_link(exit, tail);
		}

		let mut branches = std::mem::take(&mut self.branches);

		for branch in &mut branches {
			match branch {
				Branch::Full { items, .. } => self.restructure_full(nodes, items, exit),
				Branch::Empty { tail } => self.restructure_empty(nodes, head, *tail, exit),
			}
		}

		self.branches = branches;

		exit
	}

	#[must_use]
	pub fn insertions(&self) -> &[usize] {
		&self.insertions
	}

	pub fn tail_mut(&mut self) -> &mut Set {
		&mut self.tail
	}

	pub fn branches_mut(&mut self) -> &mut Vec<Branch> {
		&mut self.branches
	}

	pub fn restructure<N: NodesMut>(&mut self, nodes: &mut N, set: Slice, head: usize) -> usize {
		self.initialize_fields(nodes, head);
		self.initialize_branches(nodes, head);
		self.find_branch_elements(nodes, set, head);

		if let &[exit] = self.continuations.as_slice() {
			self.patch_single_tail(exit);

			exit
		} else {
			self.restructure_branches(nodes, head)
		}
	}
}
