#![allow(unused)]

extern crate proc_macro;

use proc_macro::TokenStream;
use std::vec::Vec;
use quote::{quote};
use syn;

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let struct_desc = StructDescriptor::from(&input).unwrap();
    let output = StructOutput::from(&struct_desc).unwrap();
    output.into()
}

struct StructDescriptor<'a> {
    ident: &'a syn::Ident,
    fields: Vec<Field<'a>>,
}

impl<'a> StructDescriptor<'a> {
    fn from(input: &'a syn::DeriveInput) -> Result<Self, ParseError> {
        match &input.data {
            syn::Data::Struct(data) => Ok(StructDescriptor{
                ident: &input.ident, 
                fields: Self::parse_fields(&data.fields)?,
            }),
            _ => Err(ParseError::new("Only structs are supported yet")),
        }
    }

    fn parse_fields(fields: &'a syn::Fields) -> Result<Vec<Field>, ParseError> {
        match fields {
            syn::Fields::Named(fields_named) => Self::parsed_fields_named(&fields_named),
            _ => Err(ParseError::new("Only named fields are supported yet")),
        }
    }

    fn parsed_fields_named(fields: &'a syn::FieldsNamed) -> Result<Vec<Field>, ParseError> {
        Ok(fields.named.iter().filter_map(|field| {
            match &field.ident {
                Some(ident) => Some(Field {
                    ident: &ident,
                    ty: &field.ty,
                }),
                None => None,
            }
        }).collect())
    }
}

#[derive(Debug)]
struct ParseError {
    message: String,
}

impl ParseError {
    fn new(msg: &str) -> ParseError {
        ParseError {
            message: msg.to_owned(),
        }
    }
}

struct Field<'a> {
    ident: &'a syn::Ident,
    ty: &'a syn::Type,
}

struct StructOutput<'a> {
    struct_ident: &'a syn::Ident,
    builder_ident: syn::Ident,
    builder_setters: Vec<syn::Stmt>,
}

impl<'a> StructOutput<'a> {
    fn from(desc: &'a StructDescriptor) -> Result<Self, ParseError> {
        let mut builder_setters: Vec<syn::Stmt> = vec![];

        for field in &desc.fields {
            let ident = &field.ident;
            let ty = &field.ty;

            builder_setters.push(syn::parse_quote!{
                pub fn #ident(&mut self, value: #ty) -> &mut Self {
                    self
                }
            });
        }

        Ok(Self {
            struct_ident: &desc.ident,
            builder_ident: quote::format_ident!("{}Builder", &desc.ident),
            builder_setters: builder_setters,
        })
    }
}

impl<'a> Into<TokenStream> for StructOutput<'a> {
    fn into(self) -> TokenStream {
        let struct_ident = &self.struct_ident;
        let builder_ident = &self.builder_ident;
        let builder_setters = &self.builder_setters;

        TokenStream::from(quote! {
            impl #struct_ident {
                pub fn builder() -> #builder_ident {
                    #builder_ident::new()
                }
            }

            struct #builder_ident {
            }

            impl #builder_ident {
                pub fn new() -> Self {
                    Self {}
                }

                #(#builder_setters)*
            }
        })
    }
}
