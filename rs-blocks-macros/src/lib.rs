//! Exposes a simple proc macro for automatically getting the name of a block.

use proc_macro::TokenStream;

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

