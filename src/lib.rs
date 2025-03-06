#![deny(missing_docs)]
//! # Short Default
//! Avoid writing tedious [Default] implementations by using a simple [`default!`](crate::default!)
//! macro instead.
//!
//!
//! ## Usage
//! ```
//! use short_default::default;
//!
//! // Define a new struct with default values
//! default! {
//!     struct Config {
//!         version: (u64, u64, u64) = (0, 1, 0),
//!         // This default value will be inferred via
//!         // authors: Default::default(),
//!         authors: Vec<String>,
//!     }
//! }
//!
//! // Use the default implementation
//! let config = Config::default();
//!
//! assert_eq!(config.version, (0, 1, 0));
//! assert_eq!(config.authors.len(), 0);
//! ```
//!
//! ## Side-by-Side Comparison
//!
//! <div style="display: grid; grid-template-columns: 1fr 1fr; column-gap: 2em;">
//! <div>
//!
//! ### Short Default
//! ```
//! # use std::fs::File;
//! use short_default::default;
//! default! {
//!     struct SomeOptions {
//!         foo: i32 = 10,
//!         bar: f32 = 5.0,
//!         baz: String = "bazzz".to_string(),
//!         qux: Option<bool> = Some(false),
//!         corge: Vec<i32>,
//!     }
//! }
//! ```
//!
//! </div>
//! <div>
//!
//! ## Traditional Approach
//!
//! ```
//! # use std::fs::File;
//! struct SomeOptions {
//!     foo: i32,
//!     bar: f32,
//!     baz: String,
//!     qux: Option<bool>,
//!     corge: Vec<i32>,
//! }
//! impl Default for SomeOptions {
//!     fn default() -> Self {
//!         Self {
//!             foo: 10,
//!             bar: 5.0,
//!             baz: "bazz".to_string(),
//!             qux: Some(false),
//!             corge: Vec::default(),
//!         }
//!     }
//! }
//! ```
//!
//! </div>
//! </div>
//!
//! ## Supported Features
//! The definition of the struct preceding the equal sign for each field is parsed conventionally
//! and identically returned.
//! This means that any regular syntax which such as field attributes, generics, etc. works as
//! well.

use proc_macro::TokenStream;

struct DefaultValue {
    #[allow(unused)]
    equal_sign: syn::Token![=],
    value: syn::Expr,
}

// #[derive(Parse)]
struct CustomField {
    field: syn::Field,
    default_value: Option<DefaultValue>,
}

impl syn::parse::Parse for DefaultValue {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let equal_sign: syn::Token![=] = input.parse()?;
        let value: syn::Expr = input.parse()?;
        Ok(Self { equal_sign, value })
    }
}

impl CustomField {
    fn parse_named(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let field = syn::Field::parse_named(input)?;
        let default_value = if !input.peek(syn::Token![,]) {
            Some(input.parse()?)
        } else {
            None
        };
        Ok(CustomField {
            field,
            default_value,
        })
    }

    fn parse_unnamed(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let field = syn::Field::parse_unnamed(input)?;
        let default_value = if !input.peek(syn::Token![,]) {
            Some(input.parse()?)
        } else {
            None
        };
        Ok(CustomField {
            field,
            default_value,
        })
    }
}

struct CustomFieldsNamed {
    brace_token: syn::token::Brace,
    named: syn::punctuated::Punctuated<CustomField, syn::Token![,]>,
}

impl CustomFieldsNamed {
    fn to_formatted_defaults(&self) -> Vec<proc_macro2::TokenStream> {
        self.named
            .iter()
            .map(
                |CustomField {
                     field,
                     default_value,
                 }| {
                    let ty = &field.ty;
                    let ident = &field.ident;
                    let value = match &default_value {
                        Some(DefaultValue {
                            equal_sign: _,
                            value,
                        }) => quote::quote!(#value),
                        None => quote::quote!(<#ty as core::default::Default>::default()),
                    };
                    quote::quote!(#ident: #value)
                },
            )
            .collect()
    }
}

impl syn::parse::Parse for CustomFieldsNamed {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            brace_token: syn::braced!(content in input),
            named: content.parse_terminated(CustomField::parse_named, syn::Token![,])?,
        })
    }
}

struct CustomFieldsUnnamed {
    paren_token: syn::token::Paren,
    unnamed: syn::punctuated::Punctuated<CustomField, syn::Token![,]>,
}

impl CustomFieldsUnnamed {
    fn to_formatted_defaults(&self) -> Vec<proc_macro2::TokenStream> {
        self.unnamed
            .iter()
            .map(|field| {
                let ty = &field.field.ty;
                let value = match &field.default_value {
                    Some(DefaultValue {
                        equal_sign: _,
                        value,
                    }) => quote::quote!(#value),
                    None => quote::quote!(<#ty as core::default::Default>::default()),
                };
                quote::quote!(#value)
            })
            .collect()
    }
}

impl syn::parse::Parse for CustomFieldsUnnamed {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            paren_token: syn::parenthesized!(content in input),
            unnamed: content.parse_terminated(CustomField::parse_unnamed, syn::Token![,])?,
        })
    }
}

enum CustomFields {
    Named(CustomFieldsNamed),
    Unnamed(CustomFieldsUnnamed),
    Unit,
}

struct Parsed {
    attrs: Vec<syn::Attribute>,
    vis: syn::Visibility,
    struct_token: syn::Token![struct],
    ident: syn::Ident,
    generics: syn::Generics,
    fields: CustomFields,
    semi_token: Option<syn::Token![;]>,
}

impl Parsed {
    fn into_item_struct(self) -> syn::ItemStruct {
        let Self {
            attrs,
            vis,
            struct_token,
            ident,
            generics,
            fields,
            semi_token,
        } = self;
        let fields = match fields {
            CustomFields::Named(CustomFieldsNamed { brace_token, named }) => {
                syn::Fields::Named(syn::FieldsNamed {
                    brace_token,
                    named: syn::punctuated::Punctuated::from_iter(
                        named.into_iter().map(|x| x.field),
                    ),
                })
            }
            CustomFields::Unnamed(CustomFieldsUnnamed {
                paren_token,
                unnamed,
            }) => syn::Fields::Unnamed(syn::FieldsUnnamed {
                paren_token,
                unnamed: syn::punctuated::Punctuated::from_iter(
                    unnamed.into_iter().map(|x| x.field),
                ),
            }),
            CustomFields::Unit => syn::Fields::Unit,
        };
        syn::ItemStruct {
            attrs,
            vis,
            struct_token,
            ident,
            generics,
            fields,
            semi_token,
        }
    }

    fn impl_default(&self) -> proc_macro2::TokenStream {
        #[allow(unused)]
        let Self {
            attrs,
            vis,
            struct_token,
            ident,
            generics,
            fields,
            semi_token,
        } = &self;
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        let fields = match fields {
            CustomFields::Named(cfn) => {
                let entries = cfn.to_formatted_defaults();
                quote::quote!(
                    Self {
                        #(#entries),*
                    }
                )
            }
            CustomFields::Unnamed(cfu) => {
                let entries = cfu.to_formatted_defaults();
                quote::quote!(
                    Self (#(#entries),*)
                )
            }
            CustomFields::Unit => quote::quote!(Self),
        };
        quote::quote!(
            impl #impl_generics core::default::Default for #ident #ty_generics #where_clause {
                fn default() -> Self { #fields }
            }
        )
    }
}

fn data_struct(
    input: syn::parse::ParseStream,
) -> syn::Result<(
    Option<syn::WhereClause>,
    CustomFields,
    Option<syn::Token![;]>,
)> {
    let mut lookahead = input.lookahead1();
    let mut where_clause = None;
    if lookahead.peek(syn::Token![where]) {
        where_clause = Some(input.parse()?);
        lookahead = input.lookahead1();
    }
    if where_clause.is_none() && lookahead.peek(syn::token::Paren) {
        let fields = input.parse()?;
        lookahead = input.lookahead1();
        if lookahead.peek(syn::Token![where]) {
            where_clause = Some(input.parse()?);
            lookahead = input.lookahead1();
        }
        if lookahead.peek(syn::Token![;]) {
            let semi = input.parse()?;
            Ok((where_clause, CustomFields::Unnamed(fields), Some(semi)))
        } else {
            Err(lookahead.error())
        }
    } else if lookahead.peek(syn::token::Brace) {
        let fields = input.parse()?;
        Ok((where_clause, CustomFields::Named(fields), None))
    } else if lookahead.peek(syn::Token![;]) {
        let semi = input.parse()?;
        Ok((where_clause, CustomFields::Unit, Some(semi)))
    } else {
        Err(lookahead.error())
    }
}

impl syn::parse::Parse for Parsed {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let vis = input.parse::<syn::Visibility>()?;
        let struct_token = input.parse::<syn::Token![struct]>()?;
        let ident = input.parse::<syn::Ident>()?;
        let generics = input.parse::<syn::Generics>()?;
        let (where_clause, fields, semi_token) = data_struct(input)?;
        Ok(Parsed {
            attrs,
            vis,
            struct_token,
            ident,
            generics: syn::Generics {
                where_clause,
                ..generics
            },
            fields,
            semi_token,
        })
    }
}

/// See the [crate-level](crate) documentation
#[proc_macro]
pub fn default(tokenstream: TokenStream) -> TokenStream {
    let parsed: Parsed = syn::parse_macro_input!(tokenstream);
    let default_impl = parsed.impl_default();
    let item_struct = parsed.into_item_struct();
    quote::quote!(
        #item_struct
        const _: () = {
            #default_impl
        };
    )
    .into()
}
