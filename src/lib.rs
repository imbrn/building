#![allow(unused)]

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use std::vec::Vec;
use syn;

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let descriptor = get_descriptor(&input).unwrap();
    descriptor.to_token_stream().unwrap()
}

fn get_descriptor(input: &syn::DeriveInput) -> Result<Box<dyn Descriptor>, ParseError> {
    match &input.data {
        syn::Data::Struct(data) => Ok(Box::new(StructDescriptor::new(&input.ident, &data))),
        _ => Err(ParseError::new("Only structs are supported yet")),
    }
}

trait Descriptor {
    fn to_token_stream(&self) -> Result<TokenStream, ParseError>;
}

struct StructDescriptor {
    ident: syn::Ident,
    data: syn::DataStruct,
}

impl StructDescriptor {
    fn new(ident: &syn::Ident, data: &syn::DataStruct) -> Self {
        StructDescriptor {
            ident: ident.clone(),
            data: data.clone(),
        }
    }

    fn parse_fields(&self) -> Result<Vec<FieldDescriptor>, ParseError> {
        match &self.data.fields {
            syn::Fields::Named(fields_named) => Ok(Self::parsed_fields_named(&fields_named)),
            _ => Err(ParseError::new("Only named fields are supported yet")),
        }
    }

    fn parsed_fields_named(fields: &syn::FieldsNamed) -> Vec<FieldDescriptor> {
        fields
            .named
            .iter()
            .filter_map(|field| match &field.ident {
                Some(ident) => Some(FieldDescriptor {
                    ident: &ident,
                    ty: &field.ty,
                }),
                None => None,
            })
            .collect()
    }
}

impl Descriptor for StructDescriptor {
    fn to_token_stream(&self) -> Result<TokenStream, ParseError> {
        Ok(StructOutput::from(&self)?.to_token_stream())
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

struct FieldDescriptor<'a> {
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

        for field in &desc.parse_fields()? {
            let ident = &field.ident;
            let ty = &field.ty;

            builder_setters.push(syn::parse_quote! {
                pub fn #ident(&mut self, value: #ty) -> &mut Self {
                    self
                }
            });
        }

        Ok(Self {
            struct_ident: &desc.ident,
            builder_ident: quote::format_ident!("{}Builder", &desc.ident),
            builder_setters,
        })
    }

    fn to_token_stream(&self) -> TokenStream {
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
