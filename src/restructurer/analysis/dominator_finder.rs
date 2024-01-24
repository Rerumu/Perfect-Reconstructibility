// Resources:
// "A Simple, Fast Dominance Algorithm",
//     by Keith D. Cooper, Timothy J. Harvey, and Ken Kennedy

use crate::control_flow::Nodes;

use super::depth_first_searcher::DepthFirstSearcher;

pub struct DominatorFinder {
	dominators: Vec<usize>,

	post_to_id: Vec<usize>,
	id_to_post: Vec<usize>,

	depth_first_searcher: DepthFirstSearcher,
}

impl DominatorFinder {
	pub const fn new() -> Self {
		Self {
			dominators: Vec::new(),
			post_to_id: Vec::new(),
			id_to_post: Vec::new(),

			depth_first_searcher: DepthFirstSearcher::new(),
		}
	}

	fn initialize_fields<N: Nodes>(&mut self, nodes: &N, start: usize) {
		let len = nodes.iter().count();
		let last_id = nodes.iter().max().map_or(0, |id| id + 1);

		self.dominators.clear();
		self.dominators.resize(len, usize::MAX);

		if let Some(entry) = self.dominators.first_mut() {
			*entry = 0;
		}

		self.post_to_id.clear();
		self.id_to_post.clear();
		self.id_to_post.resize(last_id, usize::MAX);

		self.depth_first_searcher.initialize(nodes);
		self.depth_first_searcher.run(nodes, start, |id, post| {
			if !post {
				return;
			}

			self.id_to_post[id] = len - self.post_to_id.len() - 1;
			self.post_to_id.push(id);
		});

		self.post_to_id.reverse();
	}

	fn find_intersection(&self, mut id_1: usize, mut id_2: usize) -> usize {
		while id_1 != id_2 {
			while id_2 < id_1 {
				id_1 = self.dominators[id_1];
			}

			while id_1 < id_2 {
				id_2 = self.dominators[id_2];
			}
		}

		id_1
	}

	fn run_iterations<N: Nodes>(&mut self, nodes: &N) {
		loop {
			let mut changed = false;

			for &id in &self.post_to_id {
				let dominator = nodes.predecessors(id).fold(None, |dominator, predecessor| {
					let predecessor = self.id_to_post[predecessor];

					if self.dominators[predecessor] == usize::MAX {
						dominator
					} else {
						let dominator = dominator.map_or(predecessor, |dominator| {
							self.find_intersection(predecessor, dominator)
						});

						Some(dominator)
					}
				});

				if let Some(dominator) = dominator {
					let index = self.id_to_post[id];

					if self.dominators[index] != dominator {
						self.dominators[index] = dominator;

						changed = true;
					}
				}
			}

			if !changed {
				break;
			}
		}
	}

	pub fn is_dominator(&self, dominator: usize, id: usize) -> bool {
		let dominator = self.id_to_post[dominator];
		let id = self.id_to_post[id];

		self.find_intersection(dominator, id) == dominator
	}

	pub fn run<N: Nodes>(&mut self, nodes: &N, start: usize) {
		self.initialize_fields(nodes, start);
		self.run_iterations(nodes);
	}
}
