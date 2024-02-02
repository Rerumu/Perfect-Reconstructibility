#![no_main]

use libfuzzer_sys::fuzz_target;
use list::List;
use perfect_reconstructibility::restructurer::{branch, repeat};

mod list;

fuzz_target!(|list: List| {
	let mut list = list;

	repeat::bulk::Bulk::new().restructure(&mut list, &mut list.ids());
});
