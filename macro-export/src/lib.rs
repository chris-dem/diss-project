use core::panic;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, parse_macro_input};

/// A procedural macro that adds a `describe` function to enum types
/// Usage: #[derive(EnumDescribe)]
#[proc_macro_derive(EnumCycle)]
pub fn enum_cycle(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let enum_name = &input.ident;

    // Ensure we're working with an enum
    let enum_data = match &input.data {
        Data::Enum(data) => data,
        _ => panic!("EnumDescribe can only be used with enums"),
    };
    let next_avail = enum_data.variants.iter().cycle().skip(1);

    // Generate match arms for each variant
    let match_arms =
        enum_data
            .variants
            .iter()
            .zip(next_avail)
            .map(|(current_variant, next_variant)| {
                let current_ident = &current_variant.ident;
                let next_ident = &next_variant.ident;
                quote! {
                    #enum_name::#current_ident => #enum_name::#next_ident,
                }
            });

    let expanded = quote! {
        impl EnumCycle for #enum_name {
            fn toggle(&self) -> Self {
                match self {
                    #(#match_arms)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
