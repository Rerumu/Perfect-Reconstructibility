use crate::control_flow::nodes_mut::ViewMut;

pub struct Branch {}

impl Branch {
	pub const fn new() -> Self {
		Self {}
	}

	pub fn restructure<N: ViewMut>(&mut self, nodes: &mut N, start: usize) {}
}

impl Default for Branch {
	fn default() -> Self {
		Self::new()
	}
}
