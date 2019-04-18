
/// Hello world.
#[struct_layout::explicit(size = 64, align = 4, check(Copy))]
#[derive(Copy, Clone, Debug, Default)]
pub struct A {
	#[field(offset = 1, get, set)]
	pub unaligned: u16,

	#[field(offset = 4)]
	pub int: i32,
}

#[test]
fn main() {
	let mut test: Test = unsafe { std::mem::zeroed() };
	test.set_field(42);
	panic!("{:?}", &test);
}
