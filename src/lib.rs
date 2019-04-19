/*!
Learn by example, for a detailed description see the readme.

# Examples

Most simple example, specify the required size and alignment and annotate the fields with an offset.
Use `zeroed` to create an instance and use the generated accessors to access the field.

```
#[struct_layout::explicit(size = 16, align = 4)]
struct Foo {
	#[field(offset = 4)]
	field: i32,
}

let mut foo: Foo = unsafe { std::mem::zeroed() };

foo.set_field(13);
assert_eq!(foo.field(), 13);

*foo.field_mut() = 42;
assert_eq!(foo.field_ref(), &42);
```

## Auto derive traits

Minimal set of whitelisted auto derived traits are supported.

Don't forget you can implement additional methods and traits on the generated type!

```
#[struct_layout::explicit(size = 16, align = 4)]
#[derive(Copy, Clone, Debug, Default)]
struct Foo {
	#[field(offset = 4)]
	field: char,
}

let mut foo = Foo::default().clone();
let _ = foo;
foo.set_field('a');
assert_eq!(format!("{:?}", foo), "Foo { field: 'a' }");
```

## Check argument to ensure safety

The check attribute requires that all fields meet this trait bound.
It is meant to support custom pod-like traits to ensure safe usage of the explicit attribute.

```
unsafe trait Pod {}
unsafe impl Pod for i32 {}

#[struct_layout::explicit(size = 16, align = 4, check(Pod))]
struct Foo {
	#[field(offset = 4)]
	field: i32,
}
```

## Unaligned fields

Annotate a field with `get` or `set` accessors allows unaligned fields.

```
#[struct_layout::explicit(size = 16, align = 4)]
struct Foo {
	#[field(offset = 3, get, set)]
	field: i64,
}
```

 */

use std::vec;

extern crate proc_macro;
use proc_macro::*;

//----------------------------------------------------------------
// Definitions

#[derive(Clone, Debug)]
struct KeyValue {
	ident: Ident,
	punct: Punct,
	value: Literal,
}

#[derive(Clone, Debug)]
struct ExplicitLayout {
	size: usize,
	align: usize,
	check: Option<String>,
}

#[derive(Clone, Debug)]
struct FieldLayout {
	offset: usize,
	method_get: bool,
	method_set: bool,
	method_ref: bool,
	method_mut: bool,
}

#[derive(Clone, Debug)]
struct MetaV1 {
	ident: Ident,
	args: Group,
}

#[derive(Clone, Debug)]
struct Attribute {
	punct: Punct,
	meta: Group,
}

#[derive(Copy, Clone, Debug)]
enum DerivedTrait {
	Copy, Clone, Debug, Default
}

#[derive(Clone, Debug)]
struct Vis(Vec<TokenTree>);

#[derive(Clone, Debug)]
struct Structure {
	attrs: Vec<Attribute>,
	derived: Vec<DerivedTrait>,
	layout: ExplicitLayout,
	vis: Vis,
	stru: Ident,
	name: Ident,
	fields: Vec<Field>,
}

#[derive(Clone, Debug)]
struct Type(Vec<TokenTree>);

#[derive(Clone, Debug)]
struct Field {
	attrs: Vec<Attribute>,
	layout: FieldLayout,
	vis: Vis,
	name: Ident,
	ty: Type,
}

//----------------------------------------------------------------
// Lookahead queries

fn is_ident(tokens: &[TokenTree]) -> bool {
	match tokens.first() {
		Some(TokenTree::Ident(_)) => true,
		_ => false,
	}
}
fn is_keyword(tokens: &[TokenTree], name: &str) -> bool {
	match tokens.first() {
		Some(TokenTree::Ident(ident)) => ident.to_string() == name,
		_ => false,
	}
}
fn is_punct(tokens: &[TokenTree], chr: char) -> bool {
	match tokens.first() {
		Some(TokenTree::Punct(punct)) => punct.as_char() == chr,
		_ => false,
	}
}
fn is_literal(tokens: &[TokenTree]) -> bool {
	match tokens.first() {
		Some(TokenTree::Literal(_)) => true,
		_ => false,
	}
}
fn is_group(tokens: &[TokenTree], delim: Delimiter) -> bool {
	match tokens.first() {
		Some(TokenTree::Group(group)) => group.delimiter() == delim,
		_ => false,
	}
}
fn is_end(tokens: &[TokenTree]) -> bool {
	tokens.len() == 0
}

//----------------------------------------------------------------
// Parser elements

fn parse_punct(tokens: &mut vec::IntoIter<TokenTree>, punct: char) -> Option<Punct> {
	if !is_punct(tokens.as_slice(), punct) {
		return None;
	}
	match tokens.next() {
		Some(TokenTree::Punct(punct)) => Some(punct),
		_ => unreachable!(),
	}
}
fn parse_ident(tokens: &mut vec::IntoIter<TokenTree>) -> Option<Ident> {
	match tokens.next() {
		Some(TokenTree::Ident(ident)) => Some(ident),
		_ => None,
	}
}
fn parse_keyword(tokens: &mut vec::IntoIter<TokenTree>, keyword: &str) -> Option<Ident> {
	if !is_keyword(tokens.as_slice(), keyword) {
		return None;
	}
	match tokens.next() {
		Some(TokenTree::Ident(ident)) => Some(ident),
		_ => unreachable!(),
	}
}
fn parse_group(tokens: &mut vec::IntoIter<TokenTree>, delim: Delimiter) -> Option<Group> {
	if !is_group(tokens.as_slice(), delim) {
		return None;
	}
	match tokens.next() {
		Some(TokenTree::Group(group)) => Some(group),
		_ => unreachable!(),
	}
}
fn parse_comma(tokens: &mut vec::IntoIter<TokenTree>) -> Option<()> {
	if is_end(tokens.as_slice()) {
		Some(())
	}
	else if is_punct(tokens.as_slice(), ',') {
		let _ = tokens.next();
		Some(())
	}
	else {
		None
	}
}
fn parse_end(tokens: &mut vec::IntoIter<TokenTree>) -> Option<()> {
	if tokens.len() != 0 {
		return None;
	}
	Some(())
}
fn parse_meta_v1(tokens: &mut vec::IntoIter<TokenTree>) -> Option<MetaV1> {
	let slice = tokens.as_slice();
	if !(is_ident(&slice[0..]) && is_group(&slice[1..], Delimiter::Parenthesis)) {
		return None;
	}
	let ident = match tokens.next() {
		Some(TokenTree::Ident(ident)) => ident,
		_ => unreachable!(),
	};
	let args = match tokens.next() {
		Some(TokenTree::Group(group)) => group,
		_ => unreachable!(),
	};
	Some(MetaV1 { ident, args })
}
fn parse_kv(tokens: &mut vec::IntoIter<TokenTree>) -> Option<KeyValue> {
	let slice = tokens.as_slice();
	if !(is_ident(&slice[0..]) && is_punct(&slice[1..], '=') && is_literal(&slice[2..])) {
		return None;
	}
	let ident = match tokens.next() {
		Some(TokenTree::Ident(ident)) => ident,
		_ => unreachable!(),
	};
	let punct = match tokens.next() {
		Some(TokenTree::Punct(punct)) => punct,
		_ => unreachable!(),
	};
	let value = match tokens.next() {
		Some(TokenTree::Literal(literal)) => literal,
		_ => unreachable!(),
	};
	Some(KeyValue { ident, punct, value })
}
fn parse_attr(tokens: &mut vec::IntoIter<TokenTree>) -> Option<Attribute> {
	if !is_punct(tokens.as_slice(), '#') {
		return None;
	}
	let punct = match tokens.next() {
		Some(TokenTree::Punct(punct)) => punct,
		_ => unreachable!(),
	};
	let meta = match tokens.next() {
		Some(TokenTree::Group(group)) => group,
		_ => unreachable!(),
	};
	Some(Attribute { punct, meta })
}
fn parse_attrs(tokens: &mut vec::IntoIter<TokenTree>) -> Vec<Attribute> {
	let mut attrs = Vec::new();
	while let Some(attr) = parse_attr(tokens) {
		attrs.push(attr);
	}
	attrs
}
fn parse_vis(tokens: &mut vec::IntoIter<TokenTree>) -> Vis {
	let mut vis = Vec::new();
	if is_keyword(tokens.as_slice(), "pub") {
		vis.push(tokens.next().unwrap());
		if is_group(tokens.as_slice(), Delimiter::Brace) {
			vis.push(tokens.next().unwrap());
		}
	}
	Vis(vis)
}
fn parse_ty(tokens: &mut vec::IntoIter<TokenTree>) -> Type {
	let mut ty = Vec::new();
	let mut depth = 0;
	loop {
		match tokens.next() {
			Some(TokenTree::Punct(punct)) => {
				match punct.as_char() {
					',' if depth == 0 => break,
					'<' => depth += 1,
					'>' => depth -= 1,
					_ => (),
				}
				ty.push(TokenTree::Punct(punct));
			},
			Some(tt) => ty.push(tt),
			None => break,
		}
	}
	Type(ty)
}

//----------------------------------------------------------------
// Parse struct layout attribute

fn parse_explicit_layout(tokens: TokenStream) -> ExplicitLayout {
	let tokens: Vec<TokenTree> = tokens.into_iter().collect();
	let mut tokens = tokens.into_iter();
	let size = parse_layout_size(&mut tokens);
	let align = parse_layout_align(&mut tokens);
	let check = parse_layout_check(&mut tokens);
	parse_layout_end(&mut tokens);
	ExplicitLayout { size, align, check }
}
fn parse_layout_size(tokens: &mut vec::IntoIter<TokenTree>) -> usize {
	let attr_value = match parse_kv(tokens) {
		Some(kv) => kv,
		None => panic!("parse struct_layout: invalid format for size argument, expecting `size = <usize>`"),
	};
	if let None = parse_comma(tokens) {
		panic!("parse struct_layout: invalid format for size argument, expecting `size = <usize>`");
	}
	let size = match attr_value.value.to_string().parse::<usize>() {
		Ok(ok) => ok,
		Err(err) => panic!("parse struct_layout: error parsing size argument: {}", err),
	};
	size
}
fn parse_layout_align(tokens: &mut vec::IntoIter<TokenTree>) -> usize {
	let attr_value = match parse_kv(tokens) {
		Some(kv) => kv,
		None => panic!("parse struct_layout: invalid format for align argument, expecting `align = <usize>`"),
	};
	if let None = parse_comma(tokens) {
		panic!("parse struct_layout: invalid format for align argument, expecting `align = <usize>`");
	}
	let align = match attr_value.value.to_string().parse::<usize>() {
		Ok(ok) => ok,
		Err(err) => panic!("parse struct_layout: error parsing align argument: {}", err),
	};
	align
}
fn parse_layout_check(tokens: &mut vec::IntoIter<TokenTree>) -> Option<String> {
	let meta_v1 = parse_meta_v1(tokens)?;
	if let None = parse_comma(tokens) {
		panic!("parse struct_layout: invalid format for check argument, expecting `check(PodTrait..)`");
	}
	if meta_v1.ident.to_string() != "check" {
		panic!("parse struct_layout: invalid format for check argument, expecting `check(PodTrait..)`");
	}
	Some(meta_v1.args.stream().to_string())
}
fn parse_layout_end(tokens: &mut vec::IntoIter<TokenTree>) {
	if let None = parse_end(tokens) {
		panic!("parse struct_layout: unexpected additional tokens found")
	}
}

//----------------------------------------------------------------
// Parse struct fields

fn parse_fields(tokens: TokenStream) -> Vec<Field> {
	let tokens: Vec<TokenTree> = tokens.into_iter().collect();
	let mut tokens = tokens.into_iter();
	let mut fields = Vec::new();
	while tokens.len() > 0 {
		fields.push(parse_field(&mut tokens));
	}
	fields
}
fn parse_field(tokens: &mut vec::IntoIter<TokenTree>) -> Field {
	let mut attrs = parse_attrs(tokens);
	let layout = match parse_field_attrs(&mut attrs) {
		Some(layout) => layout,
		None => panic!("parse field: every field must have a `#[field(..)]` attribute"),
	};
	let vis = parse_vis(tokens);
	let name = match parse_ident(tokens) {
		Some(ident) => ident,
		None => panic!("parse field: expecting field identifier not found"),
	};
	if let None = parse_punct(tokens, ':') {
		panic!("parse field: colon must follow field identifier");
	}
	let ty = parse_ty(tokens);
	Field { attrs, layout, vis, name, ty }
}
fn parse_field_attrs(attrs: &mut Vec<Attribute>) -> Option<FieldLayout> {
	let mut result = None;
	attrs.retain(|attr| {
		let tokens: Vec<TokenTree> = attr.meta.stream().into_iter().collect();
		let mut tokens = tokens.into_iter();
		match tokens.as_slice().first() {
			Some(TokenTree::Ident(ident)) => {
				match &*ident.to_string() {
					"field" => {
						let meta_v1 = match parse_meta_v1(&mut tokens) {
							Some(meta_v1) => meta_v1,
							None => panic!("parse field: invalid field attribute syntax, expecting `#[field(..)]`"),
						};
						if let None = parse_end(&mut tokens) {
							panic!("parse field: found extra tokens after field attribute");
						}
						let tokens: Vec<TokenTree> = meta_v1.args.stream().into_iter().collect();
						let mut tokens = tokens.into_iter();
						result = Some(parse_field_layout(&mut tokens));
						false
					},
					"doc" => true,
					s => panic!("parse field: unsupported attribute `{}`", s),
				}
			},
			Some(_) => panic!("parse field: unknown attribute syntax"),
			None => false,
		}
	});
	result
}
fn parse_field_layout(tokens: &mut vec::IntoIter<TokenTree>) -> FieldLayout {
	let offset = match parse_kv(tokens) {
		Some(offset) => offset,
		None => panic!("parse field_layout: invalid format for offset argument, expecting `offset = <usize>`"),
	};
	let offset = match offset.value.to_string().parse::<usize>() {
		Ok(offset) => offset,
		Err(err) => panic!("parse field_layout: error parsing offset argument: {}", err),
	};
	if let None = parse_comma(tokens) {
		panic!("parse field_layout: expecting comma separated list");
	}
	let mut method_get = false;
	let mut method_set = false;
	let mut method_ref = false;
	let mut method_mut = false;
	while tokens.len() > 0 {
		let ident = match parse_ident(tokens) {
			Some(ident) => ident,
			None => panic!("parse field_layout: expecting an identifier"),
		};
		let method = ident.to_string();
		match &*method {
			"get" => method_get = true,
			"set" => method_set = true,
			"ref" => method_ref = true,
			"mut" => method_mut = true,
			_ => panic!("parse field_layout: expecting an identifier of `get`, `set`, `ref` or `mut`"),
		}
		if let None = parse_comma(tokens) {
			panic!("parse field_layout: expecting comma after {}", method);
		}
	}
	// If no methods are specified, enable all of them
	if !method_get && !method_set && !method_ref && !method_mut {
		method_get = true;
		method_set = true;
		method_ref = true;
		method_mut = true;
	}
	FieldLayout { offset, method_get, method_set, method_ref, method_mut }
}

//----------------------------------------------------------------
// Parse structure

fn parse_structure(tokens: TokenStream, layout: ExplicitLayout) -> Structure {
	let tokens: Vec<TokenTree> = tokens.into_iter().collect();
	let mut tokens = tokens.into_iter();
	let mut attrs = parse_attrs(&mut tokens);
	let derived = parse_structure_attrs(&mut attrs);
	let vis = parse_vis(&mut tokens);
	let stru = match parse_keyword(&mut tokens, "struct") {
		Some(ident) => ident,
		None => panic!("parse struct: struct layout is only allowed on structs"),
	};
	let name = match parse_ident(&mut tokens) {
		Some(ident) => ident,
		None => panic!("parse struct: struct name identifier not found"),
	};
	if is_punct(tokens.as_slice(), '<') {
		panic!("parse struct: generic parameters not supported");
	}
	if is_keyword(tokens.as_slice(), "where") {
		panic!("parse struct: where clause not supported");
	}
	let group = match parse_group(&mut tokens, Delimiter::Brace) {
		Some(group) => group,
		None => panic!("parse struct: tuple syntax not supported, struct layout requires {{}} to declare the fields"),
	};
	let fields = parse_fields(group.stream());
	Structure { attrs, derived, layout, vis, stru, name, fields }
}
fn parse_structure_attrs(attrs: &mut Vec<Attribute>) -> Vec<DerivedTrait> {
	let mut result = Vec::new();
	attrs.retain(|attr| {
		let tokens: Vec<TokenTree> = attr.meta.stream().into_iter().collect();
		let mut tokens = tokens.into_iter();
		match tokens.as_slice().first() {
			Some(TokenTree::Ident(ident)) => {
				match &*ident.to_string() {
					"derive" => {
						let meta_v1 = match parse_meta_v1(&mut tokens) {
							Some(meta_v1) => meta_v1,
							None => panic!("parse struct: invalid derive syntax, expecting `#[derive(..)]`"),
						};
						if let None = parse_end(&mut tokens) {
							panic!("parse struct: found extra tokens after derive attribute");
						}
						let tokens: Vec<TokenTree> = meta_v1.args.stream().into_iter().collect();
						let mut tokens = tokens.into_iter();
						while tokens.len() > 0 {
							let ident = match parse_ident(&mut tokens) {
								Some(ident) => ident,
								None => panic!("derive attribute: expecting list of comma separated identifiers"),
							};
							let tr = ident.to_string();
							match &*tr {
								"Copy" => result.push(DerivedTrait::Copy),
								"Clone" => result.push(DerivedTrait::Clone),
								"Debug" => result.push(DerivedTrait::Debug),
								"Default" => result.push(DerivedTrait::Default),
								s => panic!("derive attribute: unsupported trait: `{}`", s),
							}
							if let None = parse_comma(&mut tokens) {
								panic!("derive attribute: expecting comma after {}", tr);
							}
						}
						// Strip the derive attribute as we'll implement it ourselves
						false
					},
					"doc" => true,
					s => panic!("parse struct: unsupported attribute `{}`", s),
				}
			},
			Some(_) => panic!("parse struct: unknown attribute syntax"),
			None => false,
		}
	});
	result
}

//----------------------------------------------------------------

/// Explicit field layout attribute.
#[proc_macro_attribute]
pub fn explicit(attributes: TokenStream, input: TokenStream) -> TokenStream {
	let layout = parse_explicit_layout(attributes);
	let stru = parse_structure(input, layout);
	// Emit the code
	let mut code: Vec<TokenTree> = Vec::new();
	emit_attrs(&mut code, &stru.attrs);
	emit_text(&mut code, &format!("#[repr(C, align({}))]", stru.layout.align));
	emit_vis(&mut code, &stru.vis);
	code.push(TokenTree::Ident(stru.stru.clone()));
	code.push(TokenTree::Ident(stru.name.clone()));
	emit_text(&mut code, &format!("([u8; {}]);", stru.layout.size));
	emit_impl_f(&mut code, &stru.name, |body| {
		for field in &stru.fields {
			emit_field(body, &stru, field);
		}
	});
	emit_derives(&mut code, &stru);
	code.into_iter().collect()
}

//----------------------------------------------------------------
// Emitters

fn emit_punct(code: &mut Vec<TokenTree>, ch: char) {
	code.push(TokenTree::Punct(Punct::new(ch, Spacing::Alone)));
}
fn emit_ident(code: &mut Vec<TokenTree>, name: &str) {
	code.push(TokenTree::Ident(Ident::new(name, Span::call_site())));
}
fn emit_text(code: &mut Vec<TokenTree>, text: &str) {
	let stream: TokenStream = text.parse().unwrap();
	code.extend(stream);
}
fn emit_group_f(code: &mut Vec<TokenTree>, delim: Delimiter, f: impl FnOnce(&mut Vec<TokenTree>)) {
	let mut tokens = Vec::new();
	f(&mut tokens);
	let stream: TokenStream = tokens.into_iter().collect();
	code.push(TokenTree::Group(Group::new(delim, stream)));
}

fn emit_attrs(code: &mut Vec<TokenTree>, attrs: &[Attribute]) {
	for attr in attrs {
		code.push(TokenTree::Punct(attr.punct.clone()));
		code.push(TokenTree::Group(attr.meta.clone()));
	}
}
fn emit_vis(code: &mut Vec<TokenTree>, vis: &Vis) {
	code.extend(vis.0.iter().cloned());
}
fn emit_ty(code: &mut Vec<TokenTree>, ty: &Type) {
	code.extend(ty.0.iter().cloned());
}
fn emit_impl_f(code: &mut Vec<TokenTree>, ident: &Ident, f: impl FnOnce(&mut Vec<TokenTree>)) {
	code.push(TokenTree::Ident(Ident::new("impl", Span::call_site())));
	code.push(TokenTree::Ident(ident.clone()));
	code.push(TokenTree::Group(Group::new(Delimiter::Brace, {
		let mut tokens = Vec::new();
		f(&mut tokens);
		tokens.into_iter().collect()
	})));
}
fn emit_trait_bounds(code: &mut Vec<TokenTree>, stru: &Structure, tr: &str) {
	if stru.fields.len() > 0 {
		emit_ident(code, "where");
		let bound = format!(": {},", tr);
		for field in &stru.fields {
			emit_ty(code, &field.ty);
			emit_text(code, &bound);
		}
	}
}
fn emit_trait_impl_f(code: &mut Vec<TokenTree>, stru: &Structure, tr: &str, f: impl FnOnce(&mut Vec<TokenTree>)) {
	code.push(TokenTree::Ident(Ident::new("impl", Span::call_site())));
	emit_text(code, tr);
	code.push(TokenTree::Ident(Ident::new("for", Span::call_site())));
	code.push(TokenTree::Ident(stru.name.clone()));
	emit_trait_bounds(code, stru, tr);
	code.push(TokenTree::Group(Group::new(Delimiter::Brace, {
		let mut tokens = Vec::new();
		f(&mut tokens);
		tokens.into_iter().collect()
	})));
}

fn emit_derive_copy(code: &mut Vec<TokenTree>, stru: &Structure) {
	emit_trait_impl_f(code, stru, "Copy", |_| {});
}
fn emit_derive_clone(code: &mut Vec<TokenTree>, stru: &Structure) {
	emit_trait_impl_f(code, stru, "Clone", |code| {
		emit_text(code, "fn clone(&self) -> Self { *self }");
	})
}
fn emit_derive_debug(code: &mut Vec<TokenTree>, stru: &Structure) {
	emit_trait_impl_f(code, stru, "::core::fmt::Debug", |code| {
		emit_text(code, "fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result");
		emit_group_f(code, Delimiter::Brace, |code| {
			emit_text(code, &format!("f.debug_struct(\"{}\")", &stru.name));
			for field in &stru.fields {
				if field.layout.method_ref {
					emit_text(code, &format!(".field(\"{0}\", self.{0}_ref())", field.name));
				}
				else if field.layout.method_get {
					emit_text(code, &format!(".field(\"{0}\", &self.{0}())", field.name));
				}
			}
			emit_text(code, ".finish()");
		});
	});
}
fn emit_derive_default(code: &mut Vec<TokenTree>, stru: &Structure) {
	emit_trait_impl_f(code, stru, "Default", |code| {
		emit_text(code, "fn default() -> Self");
		emit_group_f(code, Delimiter::Brace, |code| {
			emit_text(code, "let mut instance: Self = unsafe { ::core::mem::zeroed() };");
			for field in &stru.fields {
				emit_text(code, &format!("instance.set_{}(Default::default());", field.name));
			}
			emit_text(code, "; instance");
		});
	});
}
fn emit_derives(code: &mut Vec<TokenTree>, stru: &Structure) {
	for derive in &stru.derived {
		match derive {
			DerivedTrait::Copy => emit_derive_copy(code, stru),
			DerivedTrait::Clone => emit_derive_clone(code, stru),
			DerivedTrait::Debug => emit_derive_debug(code, stru),
			DerivedTrait::Default => emit_derive_default(code, stru),
		}
	}
}
fn emit_field(code: &mut Vec<TokenTree>, stru: &Structure, field: &Field) {
	if field.layout.method_get {
		emit_field_get(code, stru, field);
	}
	if field.layout.method_set {
		emit_field_set(code, stru, field);
	}
	if field.layout.method_ref {
		emit_field_ref(code, stru, field);
	}
	if field.layout.method_mut {
		emit_field_mut(code, stru, field);
	}
}
fn emit_field_get(code: &mut Vec<TokenTree>, stru: &Structure, field: &Field) {
	emit_attrs(code, &field.attrs);
	emit_vis(code, &field.vis);
	emit_ident(code, "fn");
	code.push(TokenTree::Ident(field.name.clone()));
	emit_text(code, "(&self) -> ");
	emit_ty(code, &field.ty);
	emit_field_check(code, stru, field);
	emit_group_f(code, Delimiter::Brace, |body| {
		emit_text(body, &format!("const FIELD_OFFSET: usize = {};", field.layout.offset));
		emit_text(body, "type FieldT = "); emit_ty(body, &field.ty);
		emit_text(body, "; use ::core::{mem, ptr}; let _: [();
			(FIELD_OFFSET + mem::size_of::<FieldT>() <= mem::size_of::<Self>()) as usize - 1];");
		emit_text(body, "unsafe { ptr::read_unaligned((self as *const _ as *const u8).offset(FIELD_OFFSET as isize) as *const FieldT) }");
	});
}
fn emit_field_set(code: &mut Vec<TokenTree>, stru: &Structure, field: &Field) {
	emit_attrs(code, &field.attrs);
	emit_vis(code, &field.vis);
	emit_ident(code, "fn");
	emit_ident(code, &format!("set_{}", field.name));
	emit_group_f(code, Delimiter::Parenthesis, |params| {
		emit_text(params, "&mut self, value: ");
		emit_ty(params, &field.ty);
	});
	emit_text(code, " -> &mut Self");
	emit_field_check(code, stru, field);
	emit_group_f(code, Delimiter::Brace, |body| {
		emit_text(body, &format!("const FIELD_OFFSET: usize = {};", field.layout.offset));
		emit_text(body, "type FieldT = "); emit_ty(body, &field.ty);
		emit_text(body, "; use ::core::{mem, ptr}; let _: [();
			(FIELD_OFFSET + mem::size_of::<FieldT>() <= mem::size_of::<Self>()) as usize - 1];");
		emit_text(body, "unsafe { ptr::write_unaligned((self as *mut _ as *mut u8).offset(FIELD_OFFSET as isize) as *mut FieldT, value); }");
		emit_ident(body, "self");
	})
}
fn emit_field_ref(code: &mut Vec<TokenTree>, stru: &Structure, field: &Field) {
	emit_attrs(code, &field.attrs);
	emit_vis(code, &field.vis);
	emit_text(code, &format!("fn {}_ref(&self) -> &", field.name));
	emit_ty(code, &field.ty);
	emit_field_check(code, stru, field);
	emit_group_f(code, Delimiter::Brace, |body| {
		emit_text(body, &format!("const FIELD_OFFSET: usize = {};", field.layout.offset));
		emit_text(body, "type FieldT = "); emit_ty(body, &field.ty);
		emit_text(body, "; use ::core::mem; let _: [();
			(FIELD_OFFSET + mem::size_of::<FieldT>() <= mem::size_of::<Self>() &&
			FIELD_OFFSET % mem::align_of::<FieldT>() == 0 &&
			mem::align_of::<FieldT>() % mem::align_of::<FieldT>() == 0) as usize - 1];");
		emit_text(body, "unsafe { &*((self as *const _ as *const u8).offset(FIELD_OFFSET as isize) as *const FieldT) }");
	});
}
fn emit_field_mut(code: &mut Vec<TokenTree>, stru: &Structure, field: &Field) {
	emit_attrs(code, &field.attrs);
	emit_vis(code, &field.vis);
	emit_text(code, &format!("fn {}_mut(&mut self) -> &mut ", field.name));
	emit_ty(code, &field.ty);
	emit_field_check(code, stru, field);
	emit_group_f(code, Delimiter::Brace, |body| {
		emit_text(body, &format!("const FIELD_OFFSET: usize = {};", field.layout.offset));
		emit_text(body, "type FieldT = "); emit_ty(body, &field.ty);
		emit_text(body, "; use ::core::mem; let _: [();
			(FIELD_OFFSET + mem::size_of::<FieldT>() <= mem::size_of::<Self>() &&
			FIELD_OFFSET % mem::align_of::<FieldT>() == 0 &&
			mem::align_of::<FieldT>() % mem::align_of::<FieldT>() == 0) as usize - 1];");
		emit_text(body, "unsafe { &mut *((self as *mut _ as *mut u8).offset(FIELD_OFFSET as isize) as *mut FieldT) }");
	});
}
fn emit_field_check(code: &mut Vec<TokenTree>, stru: &Structure, field: &Field) {
	let check = stru.layout.check.as_ref().map(std::ops::Deref::deref).unwrap_or("Copy + 'static");
	emit_ident(code, "where");
	emit_ty(code, &field.ty);
	emit_punct(code, ':');
	emit_text(code, check);
}

/// The following are incorrect usage of the explicit attribute.
///
/// ```compile_fail
/// #[struct_layout::explicit]
/// struct Foo {}
/// ```
///
/// Missing required arguments.
///
/// ```compile_fail
/// #[struct_layout::explicit(size = 8, align = 4)]
/// enum Foo {}
/// ```
///
/// The explicit attribute only works on struct definitions.
///
/// ```compile_fail
/// #[struct_layout::explicit(size = 8, align = 4)]
/// struct Foo {
/// 	field: i32,
/// }
/// ```
///
/// All fields must have exactly one `#[field]` attribute.
///
/// ```compile_fail
/// #[struct_layout::explicit(size = 8, align = 4)]
/// struct Foo {
/// 	#[field(offset = 1)]
/// 	field: i64,
/// }
/// ```
///
/// Field does not meet alignment requirements.
///
/// ```compile_fail
/// #[struct_layout::explicit(size = 8, align = 4)]
/// struct Foo {
/// 	#[field(offset = 5, get, set)]
/// 	field: i32,
/// }
/// ```
///
/// Field out of bounds.
///
/// ```compile_fail
/// unsafe trait Pod {}
///
/// #[struct_layout::explicit(size = 8, align = 4, check(Pod))]
/// struct Foo {
/// 	#[field(offset = 4)]
/// 	field: i32,
/// }
/// ```
///
/// Field type does not satisfy Pod constraint.
///
/// ```compile_fail
/// #[struct_layout::explicit(size = 8, align = 4)]
/// #[repr(C)]
/// struct Foo {}
/// ```
///
/// ```compile_fail
/// #[struct_layout::explicit(size = 8, align = 4)]
/// struct Foo {
/// 	#[field(offset = 4)]
/// 	#[allow(bad_style)]
/// 	Field: i32,
/// }
/// ```
///
/// Unsupported attributes.
#[allow(dead_code)]
fn compile_fail() {}
