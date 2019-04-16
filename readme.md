Struct layout
=============

Customize your struct layout with explicit control over the fields.

Examples
--------

Apply the `#[struct_layout]` attribute to a struct definition and put `#[field]` attributes on every field declaration.

```rust
use struct_layout::struct_layout;

#[struct_layout(explicit, size = 64, align = 4, check(Copy))]
#[derive(Copy, Clone, Debug)]
pub struct A {
	#[field(offset = 1, get, set)]
	pub unaligned: u16,

	#[field(offset = 4)]
	pub int: i32,
}
```

There's a lot to unpack here, let's start with the `struct_layout` attribute itself:

TODO!

License
-------

Licensed under [MIT License](https://opensource.org/licenses/MIT), see [license.txt](license.txt).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, shall be licensed as above, without any additional terms or conditions.
