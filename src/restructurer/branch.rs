use crate::control_flow::{Nodes, NodesMut, Var};

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

	fn find_branch_head<N: Nodes>(&mut self, nodes: &mut N, mut start: usize) -> usize {
		let mut temp = Vec::new();

		loop {
			let mut successors = nodes.successors(start);

			if let (Some(next), None) = (successors.next(), successors.next()) {
				temp.push(start);

				start = next;
			} else {
				break;
			}
		}

		nodes.add_excluded(temp);

		start
	}

	fn initialize_fields<N: Nodes>(&mut self, nodes: &N, head: usize) {
		let successors = nodes.successors(head).count();

		self.branches.clear();
		self.branches
			.resize_with(successors, || Element::Empty { tail: usize::MAX });

		self.tails.clear();
	}

	fn find_branch_elements<N: Nodes>(&mut self, nodes: &N, head: usize) {
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
		'dominated: for id in nodes.iter() {
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

		// Head is always treated as a tail, so remove it
		let index = self.tails.iter().position(|&id| id == head).unwrap();

		self.tails.swap_remove(index);
		self.tails.sort_unstable();
	}

	fn restructure_full<N: NodesMut>(&mut self, nodes: &mut N, items: &[usize], exit: usize) {
		let mut predecessors = Vec::new();

		// Find all tail connections
		for &tail in &self.tails {
			predecessors.extend(nodes.predecessors(tail).filter_map(|predecessor| {
				items
					.binary_search(&predecessor)
					.ok()
					.map(|_| (predecessor, tail))
			}));
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

	pub fn restructure<N: NodesMut>(&mut self, nodes: &mut N, start: usize) -> &mut Vec<Element> {
		let head = self.find_branch_head(nodes, start);

		self.dominator_finder.run(nodes, head);

		self.initialize_fields(nodes, head);
		self.find_branch_elements(nodes, head);

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
