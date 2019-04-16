
use struct_layout::struct_layout;

/// Hello world.
#[struct_layout(explicit, size = 8, align = 4, check(Copy))]
#[derive(Copy, Clone, Debug)]
pub struct Test {
	/// Docstring
	#[field(offset = 1, get, set)]
	pub field: i32,
}

#[test]
fn main() {
	let mut test: Test = unsafe { std::mem::zeroed() };
	test.set_field(42);
	panic!("{:?}", &test);
}
