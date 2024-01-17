use crate::control_flow::NodesMut;

pub struct Branch {}

impl Branch {
	pub const fn new() -> Self {
		Self {}
	}

	pub fn restructure<N: NodesMut>(&mut self, nodes: &mut N, start: usize) {}
}

impl Default for Branch {
	fn default() -> Self {
		Self::new()
	}
}
