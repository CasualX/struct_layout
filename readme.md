Struct layout
=============

Customize your struct layout with explicit control over the fields.

Examples
--------

Apply the `#[struct_layout::explicit]` attribute to a struct definition and put `#[field]` attributes on every field declaration.

```rust
#[struct_layout::explicit(size = 64, align = 4, check(Copy))]
#[derive(Copy, Clone, Debug)]
pub struct A {
	#[field(offset = 1, get, set)]
	pub unaligned: u16,

	#[field(offset = 4)]
	pub int: i32,
}
```

There's a lot to unpack here, let's go through it step by step.

### The struct_layout::explicit attribute

This attribute must be applied to a struct definition, the attribute arguments order is fixed:

The size and alignment of the structure are required and follow the format `size = <usize>` and `align = <usize>`.

Following is an optional `check(..)` argument which specifies a trait bound which all field members must implement.
This allows a custom trait to guarantee that all field types are safe to be used. If absent all fields are required to implement `Copy + 'static`.

### Additional attributes

Because of the invasive nature of the transformation, only a small set of whitelisted attributes are supported:

* `doc` comments are allowed and applied to the final generated structure.
* `derive` with a set of whitelisted identifiers.

### The generated structure

The proc macro generates a newtype tuple structure wrapping a byte array with length specified by the `size` argument.
The structure is adorned by the `C` and `align` representation with the alignment specified by the `align` argument.
The visibility specified is applied to the generated structure.
Any doc comments are also added.

### Supported auto derived traits

The only supported traits to be auto derived are `Copy`, `Clone` and `Debug`.

Future extensions may allow more traits to be supported.

### Structure field syntax

Every field in the structure must be accompanied by a single `#[field(..)]` attribute. No other attributes except `doc` comments are allowed.

The field attribute must start with specifying the offset of the field using `offset = <usize>`.
Followed by a list of methods for how to implement access to the field.

Supported methods are `get`, `set`, `ref` or `mut`. If no methods are specified, they will all be implemented for this field.
The accessor methods optionally add a where clause requiring the field type to implement the `check` trait specified in the `struct_layout::explicit` attribute.

The accessors are implemented via methods with the following signatures:

* get: `fn field(&self) -> T`
* set: `fn set_field(&mut self, value: T)`
* ref: `fn field_ref(&self) -> &T`
* mut: `fn field_mut(&mut self) -> &mut T`

Get and set allow unaligned offsets. If any of the fields have ref or mut accessors then the field offset and the structure specified alignment must be a multiple of the field type's alignment.

If the field offset is out of bounds or unaligned (where required) an incomprehensible error is generated complaining about const evaluation failing.
This is unfortunate due to how these constraints are statically asserted (using the array size trick).

### Safety

All fields are required to implement `Copy + 'static` or the trait bound specified by the `check` argument.
This restriction makes things safer, but by no means perfect. By using this library you commit to not doing stupid things and perhaps drop an issue in the bugtracker where safety can be improved.

Under no circumstance should a field be allowed to implement `Drop` or be anything but `'static`. It is not supported.

### How to construct an instance

Currently no constructor is provided, use the unsafe `std::mem::zeroed` to create a zero initialized instance if this makes sense.

It is possible to add constructors or traits like `Default` by manually implementing them on the generated struct.

### Compatibility with no_std

The generated code is compatible with `no_std`!

License
-------

Licensed under [MIT License](https://opensource.org/licenses/MIT), see [license.txt](license.txt).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, shall be licensed as above, without any additional terms or conditions.
