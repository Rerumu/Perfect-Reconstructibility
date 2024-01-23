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

	fn insert_new_item<N, B>(&mut self, nodes: &N, id: usize, mut pre_order: B)
	where
		N: Nodes,
		B: FnMut(usize),
	{
		if self.seen[id] {
			return;
		}

		let mut successors = self.vec_successors.pop().unwrap_or_default();

		successors.extend(nodes.successors(id));
		successors.reverse();

		self.items.push(Item { id, successors });
		self.seen[id] = true;

		pre_order(id);
	}

	pub fn initialize<N: Nodes>(&mut self, nodes: &N) {
		let last_id = nodes.iter().max().map_or(0, |id| id + 1);

		self.seen.clear();
		self.seen.resize(last_id, false);
	}

	pub fn run<N, B, A>(&mut self, nodes: &N, start: usize, mut pre_order: B, mut post_order: A)
	where
		N: Nodes,
		B: FnMut(usize),
		A: FnMut(usize),
	{
		self.insert_new_item(nodes, start, &mut pre_order);

		while let Some(mut item) = self.items.pop() {
			if let Some(successor) = item.successors.pop() {
				self.items.push(item);

				self.insert_new_item(nodes, successor, &mut pre_order);
			} else {
				post_order(item.id);

				self.vec_successors.push(item.successors);
			}
		}
	}
}
