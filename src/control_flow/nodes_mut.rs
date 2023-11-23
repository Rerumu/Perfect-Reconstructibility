use super::nodes::Nodes;

pub enum Var {
	A,
	B,
	C,
}

pub trait ViewMut: Nodes {
	#[must_use]
	fn add_assignment(&mut self, var: Var, value: usize, successor: usize) -> usize;

	#[must_use]
	fn add_selection(&mut self, var: Var) -> usize;

	fn add_link(&mut self, from: usize, to: usize);

	fn replace_link(&mut self, from: usize, to: usize, new: usize);

	fn remove_node(&mut self, id: usize);
}
