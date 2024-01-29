use std::{iter::Copied, slice::Iter};

use perfect_reconstructibility::{
	collection::set::Set,
	control_flow::{Nodes, NodesMut, Var},
	restructurer::linear::Linear,
};

enum Instruction {
	NoOperation,

	SetInteger {
		var: &'static str,
		value: i64,
	},

	Add {
		lhs: &'static str,
		rhs: &'static str,
		to: &'static str,
	},
	Sub {
		lhs: &'static str,
		rhs: &'static str,
		to: &'static str,
	},
	Mul {
		lhs: &'static str,
		rhs: &'static str,
		to: &'static str,
	},
	Div {
		lhs: &'static str,
		rhs: &'static str,
		to: &'static str,
	},

	GreaterThan {
		lhs: &'static str,
		rhs: &'static str,
		to: &'static str,
	},

	Selection {
		var: &'static str,
	},

	Return {
		var: &'static str,
	},
}

struct Node {
	predecessors: Vec<usize>,
	successors: Vec<usize>,
	instruction: Instruction,
}

struct Slice<'nodes> {
	nodes: &'nodes [Node],
}

impl<'nodes> Nodes for Slice<'nodes> {
	fn predecessors(&self, id: usize) -> Copied<Iter<'_, usize>> {
		self.nodes[id].predecessors.iter().copied()
	}

	fn successors(&self, id: usize) -> Copied<Iter<'_, usize>> {
		self.nodes[id].successors.iter().copied()
	}
}

struct SliceMut<'nodes> {
	nodes: &'nodes mut Vec<Node>,
}

impl SliceMut<'_> {
	fn add_instruction(&mut self, instruction: Instruction) -> usize {
		let node = Node {
			predecessors: Vec::new(),
			successors: Vec::new(),
			instruction,
		};

		let id = self.nodes.len();

		for &predecessor in &node.predecessors {
			self.nodes[predecessor].successors.push(id);
		}

		self.nodes.push(node);

		id
	}
}

impl<'nodes> Nodes for SliceMut<'nodes> {
	fn predecessors(&self, id: usize) -> Copied<Iter<'_, usize>> {
		self.nodes[id].predecessors.iter().copied()
	}

	fn successors(&self, id: usize) -> Copied<Iter<'_, usize>> {
		self.nodes[id].successors.iter().copied()
	}
}

impl<'nodes> NodesMut for SliceMut<'nodes> {
	fn add_no_operation(&mut self) -> usize {
		self.add_instruction(Instruction::NoOperation)
	}

	fn add_variable(&mut self, var: Var, value: usize) -> usize {
		self.add_instruction(Instruction::SetInteger {
			var: match var {
				Var::Destination => "destination",
				Var::Repetition => "repetition",
				Var::Branch => "branch",
			},
			value: value.try_into().unwrap(),
		})
	}

	fn add_selection(&mut self, var: Var) -> usize {
		self.add_instruction(Instruction::Selection {
			var: match var {
				Var::Destination => "destination",
				Var::Repetition => "repetition",
				Var::Branch => "branch",
			},
		})
	}

	fn add_link(&mut self, from: usize, to: usize) {
		self.nodes[from].successors.push(to);
		self.nodes[to].predecessors.push(from);
	}

	fn replace_link(&mut self, from: usize, to: usize, new: usize) {
		let successor = self.nodes[from]
			.successors
			.iter()
			.position(|&id| id == to)
			.unwrap();

		self.nodes[from].successors[successor] = new;
		self.nodes[new].predecessors.push(from);

		let predecessor = self.nodes[to]
			.predecessors
			.iter()
			.position(|&id| id == from)
			.unwrap();

		self.nodes[to].predecessors.remove(predecessor);
	}
}

fn write_nodes(nodes: &[Node], writer: &mut dyn std::io::Write) -> std::io::Result<()> {
	const NODE_ATTRIBUTES: &str = r##"shape = plain, style = filled, fillcolor = "#DDDDFF""##;

	writeln!(writer, "digraph {{")?;
	writeln!(writer, "\tnode [{NODE_ATTRIBUTES}];")?;
	writeln!(writer, "\tstyle = filled;")?;

	for (id, node) in nodes.iter().enumerate() {
		for &predecessor in &node.predecessors {
			writeln!(writer, "\tnode_{predecessor} -> node_{id};")?;
		}

		write!(writer, "\tnode_{id} [label=\"NODE {id}\\l")?;

		match node.instruction {
			Instruction::NoOperation => write!(writer, "no operation"),
			Instruction::SetInteger { var, value } => {
				write!(writer, "{var} := {value}")
			}
			Instruction::Add { lhs, rhs, to } => {
				write!(writer, "{to} := {lhs} + {rhs}")
			}
			Instruction::Sub { lhs, rhs, to } => {
				write!(writer, "{to} := {lhs} - {rhs}")
			}
			Instruction::Mul { lhs, rhs, to } => {
				write!(writer, "{to} := {lhs} * {rhs}")
			}
			Instruction::Div { lhs, rhs, to } => {
				write!(writer, "{to} := {lhs} / {rhs}")
			}
			Instruction::GreaterThan { lhs, rhs, to } => {
				write!(writer, "{to} := {lhs} > {rhs}")
			}
			Instruction::Selection { var } => write!(writer, "select {var}"),
			Instruction::Return { var } => write!(writer, "return {var}"),
		}?;

		writeln!(writer, "\"];")?;
	}

	writeln!(writer, "}}")?;

	Ok(())
}

fn load_example_repeat(slice: &mut SliceMut<'_>) -> usize {
	let node_1 = slice.add_instruction(Instruction::SetInteger { var: "a", value: 1 });
	let node_2 = slice.add_instruction(Instruction::Selection { var: "a" });

	let node_3 = slice.add_instruction(Instruction::Add {
		lhs: "x",
		rhs: "y",
		to: "b",
	});
	let node_4 = slice.add_instruction(Instruction::Mul {
		lhs: "b",
		rhs: "z",
		to: "b",
	});
	let node_5 = slice.add_instruction(Instruction::Selection { var: "b" });
	let node_6 = slice.add_instruction(Instruction::Sub {
		lhs: "b",
		rhs: "10",
		to: "u",
	});

	let node_7 = slice.add_instruction(Instruction::Sub {
		lhs: "x",
		rhs: "y",
		to: "c",
	});
	let node_8 = slice.add_instruction(Instruction::Div {
		lhs: "c",
		rhs: "z",
		to: "c",
	});
	let node_9 = slice.add_instruction(Instruction::Selection { var: "c" });
	let node_10 = slice.add_instruction(Instruction::Add {
		lhs: "c",
		rhs: "10",
		to: "u",
	});

	let node_11 = slice.add_instruction(Instruction::Return { var: "u" });

	slice.add_link(node_1, node_2);
	slice.add_link(node_2, node_3);
	slice.add_link(node_2, node_7);

	slice.add_link(node_3, node_4);
	slice.add_link(node_4, node_5);
	slice.add_link(node_5, node_6);
	slice.add_link(node_5, node_7);

	slice.add_link(node_7, node_8);
	slice.add_link(node_8, node_9);
	slice.add_link(node_9, node_10);
	slice.add_link(node_9, node_3);

	slice.add_link(node_6, node_11);
	slice.add_link(node_10, node_11);

	node_1
}

fn load_example_branch(slice: &mut SliceMut<'_>) -> usize {
	let node_1 = slice.add_instruction(Instruction::GreaterThan {
		lhs: "y",
		rhs: "0",
		to: "a",
	});

	let node_2 = slice.add_instruction(Instruction::Selection { var: "a" });

	let node_3 = slice.add_instruction(Instruction::GreaterThan {
		lhs: "x",
		rhs: "0",
		to: "b",
	});

	let node_4 = slice.add_instruction(Instruction::Selection { var: "b" });

	let node_5 = slice.add_instruction(Instruction::Add {
		lhs: "x",
		rhs: "y",
		to: "x",
	});

	let node_6 = slice.add_instruction(Instruction::Sub {
		lhs: "x",
		rhs: "y",
		to: "x",
	});

	let node_7 = slice.add_instruction(Instruction::Mul {
		lhs: "x",
		rhs: "x",
		to: "x",
	});

	let node_8 = slice.add_instruction(Instruction::Return { var: "x" });

	slice.add_link(node_1, node_2);
	slice.add_link(node_2, node_3);
	slice.add_link(node_2, node_7);

	slice.add_link(node_3, node_4);
	slice.add_link(node_4, node_5);
	slice.add_link(node_4, node_6);

	slice.add_link(node_5, node_8);
	slice.add_link(node_6, node_7);

	slice.add_link(node_7, node_8);

	node_1
}

fn main() {
	let mut nodes = vec![];
	let mut slice = SliceMut { nodes: &mut nodes };

	let node_1 = load_example_branch(&mut slice);
	let mut set: Set = (0..slice.nodes.len()).collect();

	write_nodes(slice.nodes, &mut std::io::stdout()).unwrap();

	Linear::new().restructure(&mut slice, &mut set, node_1);

	write_nodes(slice.nodes, &mut std::io::stdout()).unwrap();
}
