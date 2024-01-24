// Resources:
// "Path-based depth-first search for strong and biconnected components",
//     by Harold N. Gabow

use crate::control_flow::Nodes;

use super::depth_first_searcher::DepthFirstSearcher;

#[derive(Default)]
pub struct StronglyConnectedFinder {
	names: Vec<usize>,
	path: Vec<usize>,
	stack: Vec<usize>,

	results: Vec<Vec<usize>>,
	vec_unused: Vec<Vec<usize>>,
	depth_first_searcher: DepthFirstSearcher,
}

impl StronglyConnectedFinder {
	pub const fn new() -> Self {
		Self {
			names: Vec::new(),
			path: Vec::new(),
			stack: Vec::new(),

			results: Vec::new(),
			vec_unused: Vec::new(),
			depth_first_searcher: DepthFirstSearcher::new(),
		}
	}

	fn initialize_fields<N: Nodes>(&mut self, nodes: &N) {
		let last_id = nodes.iter().max().map_or(0, |id| id + 1);

		self.names.clear();
		self.names.resize(last_id, usize::MAX);

		self.vec_unused.append(&mut self.results);
	}

	fn on_pre_order<N: Nodes>(&mut self, nodes: &N, id: usize) {
		let index = self.path.len();

		self.names[id] = index;

		self.path.push(id);
		self.stack.push(index);

		for successor in nodes.successors(id).filter(|&id| id != usize::MAX) {
			if let Some(&index) = self.names.get(successor) {
				let last = self.stack.iter().rposition(|&id| id <= index).unwrap();

				self.stack.truncate(last + 1);
			}
		}
	}

	fn on_post_order(&mut self, id: usize) {
		let index = self.stack.pop().unwrap();

		if self.names[id] != index {
			self.stack.push(index);

			return;
		}

		let mut result = self.vec_unused.pop().unwrap_or_default();

		result.clear();
		result.extend(self.path.drain(index..));

		for &id in &result {
			self.names[id] = usize::MAX;
		}

		if result.len() > 1 {
			self.results.push(result);
		} else {
			self.vec_unused.push(result);
		}
	}

	pub fn run<N: Nodes>(&mut self, nodes: &N) -> &mut Vec<Vec<usize>> {
		let mut depth_first_searcher = std::mem::take(&mut self.depth_first_searcher);

		depth_first_searcher.initialize(nodes);

		self.initialize_fields(nodes);

		for id in nodes.iter() {
			depth_first_searcher.run(nodes, id, |id, post| {
				if post {
					self.on_post_order(id);
				} else {
					self.on_pre_order(nodes, id);
				}
			});
		}

		self.depth_first_searcher = depth_first_searcher;

		debug_assert!(self.path.is_empty(), "path is not empty");
		debug_assert!(self.stack.is_empty(), "stack is not empty");

		&mut self.results
	}
}
