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

    fn evaluate_struct_field_init(field: &FieldDescriptor) -> impl quote::ToTokens {
        let ident = &field.ident;
        let ty = &field.ty;

        quote!{
            #ident: builder.#ident.clone().ok_or("Failed to build field #ident".to_owned())?
        }
    }

    fn evaluate_builder_field_decl(field: &FieldDescriptor) -> impl quote::ToTokens {
        let ident = &field.ident;
        let ty = &field.ty;

        quote!{
            pub #ident: std::option::Option<#ty>
        }
    }

    fn evaluate_builder_field_init(field: &FieldDescriptor) -> impl quote::ToTokens {
        let ident = &field.ident;

        quote!{
            #ident: std::option::Option::None
        }
    }

    fn evaluate_builder_setter(field: &FieldDescriptor) -> impl quote::ToTokens {
        let ident = &field.ident;
        let ty = &field.ty;

        quote! {
            pub fn #ident(&mut self, value: #ty) -> &mut Self {
                self.#ident = std::option::Option::Some(value);
                self
            }
        }
    }
}

impl Descriptor for StructDescriptor {
    fn to_token_stream(&self) -> Result<TokenStream, ParseError> {
        let fields = self.parse_fields()?;

        let struct_ident = &self.ident;
        let builder_ident = quote::format_ident!("{}Builder", &struct_ident);

        let mut struct_fields_init: Vec<Box<dyn quote::ToTokens>> = vec![];
        let mut builder_fields_decl: Vec<Box<dyn quote::ToTokens>> = vec![];
        let mut builder_fields_init: Vec<Box<dyn quote::ToTokens>> = vec![];
        let mut builder_setters: Vec<Box<dyn quote::ToTokens>> = vec![];

        for field in &fields {
            struct_fields_init.push(Box::new(Self::evaluate_struct_field_init(&field)));
            builder_fields_decl.push(Box::new(Self::evaluate_builder_field_decl(&field)));
            builder_fields_init.push(Box::new(Self::evaluate_builder_field_init(&field)));
            builder_setters.push(Box::new(Self::evaluate_builder_setter(&field)));
        }

        Ok(TokenStream::from(quote! {
            impl #struct_ident {
                pub fn builder() -> #builder_ident {
                    #builder_ident::new()
                }

                fn from_builder(builder: &#builder_ident) -> std::result::Result<#struct_ident, String> {
                    std::result::Result::Ok(#struct_ident{
                        #(#struct_fields_init),*
                    })
                }
            }

            pub struct #builder_ident {
                #(#builder_fields_decl),*
            }

            impl #builder_ident {
                fn new() -> Self {
                    Self {
                        #(#builder_fields_init),*
                    }
                }

                #(#builder_setters)*

                pub fn build(&self) -> std::result::Result<#struct_ident, String> {
                    #struct_ident::from_builder(&self)
                }
            }
        }))
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
