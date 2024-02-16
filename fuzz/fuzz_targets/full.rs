#![no_main]

use libfuzzer_sys::fuzz_target;
use list::List;
use perfect_reconstructibility::structurer::{branch, repeat};

mod list;

fuzz_target!(|list: List| {
	let mut list = list;
	let mut set = list.ids();

	repeat::Bulk::new().run(&mut list, &mut set);
	branch::Bulk::new().run(&mut list, &mut set, 0);
});
