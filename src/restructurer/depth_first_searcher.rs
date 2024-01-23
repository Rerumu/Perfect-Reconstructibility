use crate::control_flow::Nodes;

struct Item {
	id: usize,
	successors: Vec<usize>,
}

#[derive(Default)]
pub struct DepthFirstSearcher {
	items: Vec<Item>,
	seen: Vec<bool>,

	vec_successors: Vec<Vec<usize>>,
}

impl DepthFirstSearcher {
	pub const fn new() -> Self {
		Self {
			items: Vec::new(),
			seen: Vec::new(),

			vec_successors: Vec::new(),
		}
	}

	fn insert_new_item<N, H>(&mut self, nodes: &N, id: usize, mut handler: H)
	where
		N: Nodes,
		H: FnMut(usize, bool),
	{
		if self.seen.get(id).copied().unwrap_or(true) {
			return;
		}

		let mut successors = self.vec_successors.pop().unwrap_or_default();

		successors.extend(nodes.successors(id));
		successors.reverse();

		self.items.push(Item { id, successors });
		self.seen[id] = true;

		handler(id, false);
	}

	pub fn initialize<N: Nodes>(&mut self, nodes: &N) {
		let last_id = nodes.iter().max().map_or(0, |id| id + 1);

		self.seen.clear();
		self.seen.resize(last_id, true);

		for id in nodes.iter() {
			self.seen[id] = false;
		}
	}

	pub fn run<N, H>(&mut self, nodes: &N, start: usize, mut handler: H)
	where
		N: Nodes,
		H: FnMut(usize, bool),
	{
		self.insert_new_item(nodes, start, &mut handler);

		while let Some(mut item) = self.items.pop() {
			if let Some(successor) = item.successors.pop() {
				self.items.push(item);

				self.insert_new_item(nodes, successor, &mut handler);
			} else {
				handler(item.id, true);

				self.vec_successors.push(item.successors);
			}
		}
	}
}
