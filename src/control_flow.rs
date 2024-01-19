pub trait Nodes {
	fn predecessors(&self, id: usize) -> impl Iterator<Item = usize> + '_;

	fn successors(&self, id: usize) -> impl Iterator<Item = usize> + '_;

	fn iter(&self) -> impl Iterator<Item = usize> + '_;

	fn contains(&self, id: usize) -> bool;

	fn view<I: IntoIterator<Item = usize>>(&self, set: I) -> impl Nodes + '_;
}

pub enum Var {
	A,
	B,
	C,
}

pub trait NodesMut: Nodes {
	fn add_assignment(&mut self, var: Var, value: usize, successor: usize) -> usize;

	fn add_selection(&mut self, var: Var) -> usize;

	fn exclude_node(&mut self, id: usize);

	fn insert_link(&mut self, from: usize, to: usize);

	fn replace_link(&mut self, from: usize, to: usize, new: usize);

	fn view_mut<I: IntoIterator<Item = usize>>(&mut self, set: I) -> impl NodesMut + '_;
}
