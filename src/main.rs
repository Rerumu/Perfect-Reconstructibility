use list::List;
use perfect_reconstructibility::{
	control_flow::NodesMut,
	restructurer::{branch, repeat},
};

use self::list::Instruction;

mod list;

fn load_example_repeat(list: &mut List) -> (usize, usize) {
	let node_1 = list.add_instruction(Instruction::Start);
	let node_2 = list.add_no_operation();

	let node_3 = list.add_no_operation();
	let node_4 = list.add_no_operation();
	let node_5 = list.add_no_operation();
	let node_6 = list.add_no_operation();

	let node_7 = list.add_no_operation();
	let node_8 = list.add_no_operation();
	let node_9 = list.add_no_operation();
	let node_10 = list.add_no_operation();

	let node_11 = list.add_instruction(Instruction::End);

	list.add_link(node_1, node_2);
	list.add_link(node_2, node_3);
	list.add_link(node_2, node_7);

	list.add_link(node_3, node_4);
	list.add_link(node_4, node_5);
	list.add_link(node_5, node_6);
	list.add_link(node_5, node_7);

	list.add_link(node_7, node_8);
	list.add_link(node_8, node_9);
	list.add_link(node_9, node_10);
	list.add_link(node_9, node_3);

	list.add_link(node_6, node_11);
	list.add_link(node_10, node_11);

	(node_1, node_11)
}

fn load_example_branch(list: &mut List) -> (usize, usize) {
	let node_1 = list.add_instruction(Instruction::Start);
	let node_2 = list.add_no_operation();
	let node_3 = list.add_no_operation();
	let node_4 = list.add_no_operation();
	let node_5 = list.add_no_operation();
	let node_6 = list.add_no_operation();
	let node_7 = list.add_no_operation();
	let node_8 = list.add_instruction(Instruction::End);

	list.add_link(node_1, node_2);
	list.add_link(node_2, node_3);
	list.add_link(node_2, node_7);

	list.add_link(node_3, node_4);
	list.add_link(node_4, node_5);
	list.add_link(node_4, node_6);

	list.add_link(node_5, node_8);
	list.add_link(node_6, node_7);

	list.add_link(node_7, node_8);

	(node_1, node_8)
}

fn load_example_nested(list: &mut List) -> (usize, usize) {
	let node_1 = list.add_no_operation();
	let node_2 = list.add_no_operation();
	let node_3 = list.add_no_operation();
	let node_4 = list.add_no_operation();
	let node_5 = list.add_no_operation();

	list.add_link(node_1, node_2);
	list.add_link(node_2, node_3);
	list.add_link(node_3, node_4);
	list.add_link(node_4, node_5);

	list.add_link(node_4, node_2);
	list.add_link(node_1, node_5);

	(node_1, node_5)
}

fn load_example_double(list: &mut List) -> (usize, usize) {
	let node_1 = list.add_no_operation();
	let node_2 = list.add_no_operation();
	let node_3 = list.add_no_operation();
	let node_4 = list.add_no_operation();

	list.add_link(node_1, node_2);
	list.add_link(node_2, node_2);
	list.add_link(node_2, node_3);
	list.add_link(node_2, node_3);
	list.add_link(node_3, node_4);

	(node_1, node_4)
}

fn load_example_unhinged(list: &mut List) -> (usize, usize) {
	let node_0 = list.add_no_operation();
	let node_1 = list.add_no_operation();
	let node_2 = list.add_no_operation();

	list.add_link(node_0, node_1);
	list.add_link(node_0, node_2);
	list.add_link(node_1, node_1);

	(node_0, node_2)
}

fn main() {
	let mut list = List::new();

	let (entry, _) = load_example_unhinged(&mut list);

	list.set_synthetic(true);

	// let (br_in, br_out) = load_example_branch(&mut list);
	// let (rp_in, rp_out) = load_example_repeat(&mut list);
	// let entry = list.add_no_operation();
	// let exit = list.add_no_operation();

	// list.add_link(entry, br_in);
	// list.add_link(entry, rp_in);

	// list.add_link(br_out, exit);
	// list.add_link(rp_out, exit);

	let mut set = list.ids();

	println!("{list:?}");

	repeat::bulk::Bulk::new().restructure(&mut list, &mut set);

	println!("{list:?}");

	branch::bulk::Bulk::new().restructure(&mut list, &mut set, entry);

	println!("{list:?}");
}
