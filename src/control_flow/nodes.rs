pub trait Nodes {
	type Predecessors<'nodes>: Iterator<Item = usize> + 'nodes
	where
		Self: 'nodes;

	type Successors<'nodes>: Iterator<Item = usize> + 'nodes
	where
		Self: 'nodes;

	type Iter<'nodes>: Iterator<Item = usize> + 'nodes
	where
		Self: 'nodes;

	type View<'parent>: Nodes + 'parent
	where
		Self: 'parent;

	fn predecessors(&self, id: usize) -> Self::Predecessors<'_>;

	fn successors(&self, id: usize) -> Self::Successors<'_>;

	fn iter(&self) -> Self::Iter<'_>;

	#[must_use]
	fn view<I: IntoIterator<Item = usize>>(&mut self, set: I) -> Self::View<'_>;
}
