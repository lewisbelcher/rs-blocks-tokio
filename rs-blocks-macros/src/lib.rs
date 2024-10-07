//! Exposes a simple proc macro for automatically getting the name of a block.

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse::Parser;
use syn::{punctuated::Punctuated, Token};

#[proc_macro_derive(NoMarkup)]
pub fn derive_no_markup(input: TokenStream) -> TokenStream {
	let ast = syn::parse_macro_input!(input as syn::DeriveInput);
	let name = &ast.ident;
	let gen = quote::quote! {
		impl GetMarkup for #name {}
	};
	gen.into()
}

#[proc_macro_derive(PangoMarkup)]
pub fn derive_pango_markup(input: TokenStream) -> TokenStream {
	let ast = syn::parse_macro_input!(input as syn::DeriveInput);
	let name = &ast.ident;
	let gen = quote::quote! {
		impl GetMarkup for #name {
			fn get_markup() -> Option<&'static str> {
				Some("pango")
			}
		}
	};
	gen.into()
}

#[proc_macro_derive(GetName)]
pub fn derive_get_name(input: TokenStream) -> TokenStream {
	let ast = syn::parse_macro_input!(input as syn::DeriveInput);
	let name = &ast.ident;
	let gen = quote::quote! {
		impl GetName for #name {
			fn get_name() -> &'static str {
				stringify!(#name)
			}
		}
	};
	gen.into()
}

#[proc_macro_derive(IntoSerialized)]
pub fn derive_into_serialized(input: TokenStream) -> TokenStream {
	let ast = syn::parse_macro_input!(input as syn::DeriveInput);
	let name = &ast.ident;
	let gen = quote::quote! {
		impl IntoSerialized for #name {}
	};
	gen.into()
}

/// Add common block fields to a struct
///
/// Available fields are `alpha` and `period`. A serde default will be used which uses the function
/// `default_{name}`. Any function in scope that matches this pattern will be used to provide the
/// default value.
///
/// Example:
///
/// ```
/// use rs_blocks_macros::with_fields;
/// use serde::Deserialize;
/// use serde_json;
///
/// #[with_fields(alpha)]
/// #[derive(Deserialize)]
/// struct A {
/// 	name: String
/// }
///
/// fn default_alpha() -> f32 {
/// 	0.1
/// }
///
/// let a: A = serde_json::from_str("{ \"name\": \"hello\" }").unwrap();
/// assert_eq!(a.alpha, 0.1);
/// ```
#[proc_macro_attribute]
pub fn with_fields(attr: TokenStream, item: TokenStream) -> TokenStream {
	// Can't use syn::parse_macro_input directly because there are multiple strategies for parsing a
	// punctuated sequence (empty allowed, trailing punctuation allowed etc) and we need to specify
	// which is okay
	let names = Punctuated::<syn::Ident, Token![,]>::parse_separated_nonempty
		.parse(attr)
		.expect("failed to parse attribute");
	let mut item_struct = syn::parse_macro_input!(item as syn::ItemStruct);
	// TODO: the serde default functions are implicitly used from the enclosing scope where this
	// function is called. We should make this more explicit by optionally passing an argument to the
	// attribute call like `#[with_fields(alpha, period(default=1000))]`
	if let syn::Fields::Named(ref mut fields) = item_struct.fields {
		for name in names.into_iter() {
			let token_stream = match name.to_string().as_ref() {
				"alpha" => {
					quote::quote! {
						#[serde(default = "default_alpha")]
						alpha: f32
					}
				}
				"period" => {
					quote::quote! {
						#[serde(default = "default_period")]
						period: u64
					}
				}
				attr => unimplemented!("unrecognised attribute '{}'", attr),
			};
			fields
				.named
				.push(syn::Field::parse_named.parse(token_stream.into()).unwrap());
		}
	} else {
		unimplemented!("cannot use `with_fields` on struct without named fields")
	}
	item_struct.into_token_stream().into()
}
