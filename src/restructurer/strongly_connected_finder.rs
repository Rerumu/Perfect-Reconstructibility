use crate::control_flow::Nodes;

enum Record {
	Unseen,
	Seen,
	Named { lowest: u32 },
}

impl Record {
	const fn lowest(&self) -> u32 {
		match self {
			Self::Unseen | Self::Seen => u32::MAX,
			Self::Named { lowest } => *lowest,
		}
	}

	fn set_lowest(&mut self, value: u32) {
		match self {
			Self::Unseen | Self::Seen => {}
			Self::Named { lowest } => *lowest = value.min(*lowest),
		}
	}
}

struct Item {
	name: u32,
	id: usize,
	successors: Vec<usize>,
}

pub struct StronglyConnectedFinder {
	results: Vec<Vec<usize>>,
	records: Vec<Record>,
	history: Vec<usize>,
	items: Vec<Item>,
	names: u32,
}

impl StronglyConnectedFinder {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			results: Vec::new(),
			records: Vec::new(),
			history: Vec::new(),
			items: Vec::new(),
			names: 0,
		}
	}

	fn initialize_fields(&mut self, record_count: usize) {
		self.results.clear();
		self.records.clear();
		self.records.resize_with(record_count, || Record::Seen);

		self.names = 0;
	}

	fn initialize_item<N: Nodes>(&mut self, nodes: &N, id: usize) {
		let successors = nodes.successors(id).collect();

		self.history.push(id);
		self.items.push(Item {
			name: self.names,
			id,
			successors,
		});

		self.records[id] = Record::Named { lowest: self.names };
		self.names += 1;
	}

	fn finalize_item(&mut self, id: usize) {
		let position = self.history.iter().rposition(|&other| id == other).unwrap();

		for &id in &self.history[position..] {
			self.records[id] = Record::Seen;
		}

		let result = self.history.drain(position..);

		if result.len() > 1 {
			self.results.push(result.collect());
		}
	}

	#[allow(clippy::match_on_vec_items)]
	fn handle_from_successor<N: Nodes>(&mut self, nodes: &N, id: usize, successor: usize) {
		match self.records[successor] {
			Record::Unseen => self.initialize_item(nodes, successor),
			Record::Seen => {}
			Record::Named { lowest } => self.records[id].set_lowest(lowest),
		}
	}

	fn handle_to_predecessor(&mut self, predecessor: usize, id: usize) {
		let lowest = self.records[id].lowest();

		self.records[predecessor].set_lowest(lowest);
	}

	fn run_at_position<N: Nodes>(&mut self, nodes: &N, start: usize) {
		self.handle_from_successor(nodes, start, start);

		while let Some(mut item @ Item { name, id, .. }) = self.items.pop() {
			if let Some(successor) = item.successors.pop() {
				self.items.push(item);

				self.handle_from_successor(nodes, id, successor);
			} else {
				if let Some(predecessor) = self.items.last() {
					self.handle_to_predecessor(predecessor.id, id);
				}

				if name == self.records[id].lowest() {
					self.finalize_item(id);
				}
			}
		}
	}

	pub fn run<N: Nodes>(&mut self, nodes: &N) -> &mut Vec<Vec<usize>> {
		let record_count = nodes.iter().max().map_or(0, |id| id + 1);

		self.initialize_fields(record_count);

		for id in nodes.iter() {
			self.records[id] = Record::Unseen;
		}

		for id in nodes.iter() {
			self.run_at_position(nodes, id);
		}

		debug_assert!(self.items.is_empty(), "items is not empty");
		debug_assert!(self.history.is_empty(), "history is not empty");

		&mut self.results
	}
}
