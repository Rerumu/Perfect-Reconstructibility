// Resources:
// "Path-based depth-first search for strong and biconnected components",
//     by Harold N. Gabow

use crate::{nodes::Successors, set::Set};

use super::depth_first_searcher::DepthFirstSearcher;

#[derive(Default)]
pub struct StronglyConnectedFinder {
	names: Vec<usize>,
	path: Vec<usize>,
	stack: Vec<usize>,

	depth_first_searcher: DepthFirstSearcher,
}

impl StronglyConnectedFinder {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			names: Vec::new(),
			path: Vec::new(),
			stack: Vec::new(),

			depth_first_searcher: DepthFirstSearcher::new(),
		}
	}

	fn fill_names(&mut self) {
		let last = self
			.depth_first_searcher
			.unseen()
			.ones()
			.max()
			.map_or(0, |index| index + 1);

		self.names.clear();
		self.names.resize(last, usize::MAX);
	}

	fn on_pre_order<N: Successors>(&mut self, nodes: &N, id: usize) {
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

	fn on_post_order(&mut self, id: usize) -> Option<Set> {
		let index = self.stack.pop().unwrap();

		if self.names[id] != index {
			self.stack.push(index);

			return None;
		}

		for &id in &self.path[index..] {
			self.names[id] = usize::MAX;
		}

		let result = self.path.drain(index..);

		(result.len() > 1).then(|| result.collect())
	}

	fn run_search<N, H, S>(&mut self, nodes: &N, set: S, mut handler: H)
	where
		N: Successors,
		H: FnMut(Set),
		S: IntoIterator<Item = usize>,
	{
		let mut depth_first_searcher = core::mem::take(&mut self.depth_first_searcher);

		for id in set {
			depth_first_searcher.run(nodes, id, |id, post| {
				if post {
					if let Some(component) = self.on_post_order(id) {
						handler(component);
					}
				} else {
					self.on_pre_order(nodes, id);
				}
			});
		}

		self.depth_first_searcher = depth_first_searcher;
	}

	pub fn run<N, H, S>(&mut self, nodes: &N, set: S, handler: H)
	where
		N: Successors,
		H: FnMut(Set),
		S: IntoIterator<Item = usize> + Clone,
	{
		self.depth_first_searcher.restrict(set.clone());

		self.fill_names();
		self.run_search(nodes, set, handler);
	}
}
