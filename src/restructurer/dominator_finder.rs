use crate::control_flow::Nodes;

struct Item {
	name: u32,
	id: usize,
	successors: Vec<usize>,
}

pub struct DominatorFinder {
	// dominators: Vec<Option<usize>>,
	reverse_post_order: Vec<usize>,
}

impl DominatorFinder {
	pub const fn new() -> Self {
		Self {
			// dominators: Vec::new(),
			reverse_post_order: Vec::new(),
		}
	}

	fn initialize_fields<N: Nodes>(&mut self, nodes: &N) {
		self.reverse_post_order.clear();
	}

	pub fn run<N: Nodes>(&mut self, nodes: &N) {
		loop {
			let mut changed = false;

			if !changed {
				break;
			}
		}
	}
}
