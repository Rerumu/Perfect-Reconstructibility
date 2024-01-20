pub trait Nodes {
	fn predecessors(&self, id: usize) -> impl Iterator<Item = usize> + '_;

	fn successors(&self, id: usize) -> impl Iterator<Item = usize> + '_;

	fn iter(&self) -> impl Iterator<Item = usize> + '_;

	fn contains(&self, id: usize) -> bool;

	fn add_excluded<I: IntoIterator<Item = usize>>(&mut self, set: I);

	fn set_included<I: IntoIterator<Item = usize>>(&mut self, set: I);
}

pub enum Var {
	Destination,
	Repetition,
	Branch,
}

pub trait NodesMut: Nodes {
	fn add_no_operation(&mut self) -> usize;

	fn add_selection(&mut self, var: Var) -> usize;

	fn add_variable(&mut self, var: Var, value: usize) -> usize;

	fn add_link(&mut self, from: usize, to: usize);

	fn remove_link(&mut self, from: usize, to: usize);
}
