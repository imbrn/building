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
            syn::Fields::Named(fields_named) => Ok(Self::parse_fields_named(&fields_named)),
            _ => Err(ParseError::new("Only named fields are supported yet")),
        }
    }

    fn parse_fields_named(fields: &syn::FieldsNamed) -> Vec<FieldDescriptor> {
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
        let fields = self.parse_fields()?;

        let struct_ident = &self.ident;
        let builder_ident = quote::format_ident!("{}Builder", &struct_ident);

        let mut struct_fields_init: Vec<Box<dyn quote::ToTokens>> = vec![];
        let mut builder_fields_decl: Vec<Box<dyn quote::ToTokens>> = vec![];
        let mut builder_fields_init: Vec<Box<dyn quote::ToTokens>> = vec![];
        let mut builder_setters: Vec<Box<dyn quote::ToTokens>> = vec![];

        for field in &fields {
            struct_fields_init.push(Box::new(field.struct_field_init()));
            builder_fields_decl.push(Box::new(field.builder_field_decl()));
            builder_fields_init.push(Box::new(field.builder_field_init()));
            builder_setters.push(Box::new(field.builder_setter()));
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

struct FieldDescriptor<'a> {
    ident: &'a syn::Ident,
    ty: &'a syn::Type,
}

impl<'a> FieldDescriptor<'a> {
    fn struct_field_init(&self) -> impl quote::ToTokens {
        let ident = &self.ident;
        let ty = &self.ty;
        
        if let Some(_) = self.optional_type() {
            quote!{ #ident: builder.#ident.clone() }
        } else {
            quote!{ #ident: builder.#ident.clone().ok_or("Failed to build field".to_owned())? }
        }
    }

    fn builder_field_decl(&self) -> impl quote::ToTokens {
        let ident = self.ident;
        let ty = self.get_type();
        quote!{ pub #ident: std::option::Option<#ty> }
    }

    fn builder_field_init(&self) -> impl quote::ToTokens {
        let ident = self.ident;
        quote!{ #ident: std::option::Option::None }
    }

    fn builder_setter(&self) -> impl quote::ToTokens {
        let ident = self.ident;
        let ty = self.get_type();

        quote! {
            pub fn #ident(&mut self, value: #ty) -> &mut Self {
                self.#ident = std::option::Option::Some(value);
                self
            }
        }
    }

    fn get_type(&self) -> syn::Type {
        let optional_type = self.optional_type();

        match self.optional_type() {
            Some(optional_type) => optional_type,
            _ => self.ty.clone(),
        }
    }

    fn optional_type(&self) -> Option<syn::Type> {
        match self.ty {
            syn::Type::Path(path) => Self::get_path_optional_type(path),
            _ => None,
        }
    }

    fn get_path_optional_type(type_path: &syn::TypePath) -> Option<syn::Type> {
        if let Some(segment) = type_path.path.segments.first() {
            if segment.ident == "Option" {
                return Self::get_optional_type_from_path_arguments(&segment.arguments);
            }
        }
        None
    }

    fn get_optional_type_from_path_arguments(arguments: &syn::PathArguments) -> Option<syn::Type> {
        if let syn::PathArguments::AngleBracketed(arguments) = arguments {
            return Self::get_optional_angle_bracketed_type(&arguments);
        }
        None
    }

    fn get_optional_angle_bracketed_type(arguments: &syn::AngleBracketedGenericArguments) -> Option<syn::Type> {
        if let Some(argument) = arguments.args.first() {
            if let syn::GenericArgument::Type(ty) = argument {
                return Some(ty.clone());
            }
        }
        None
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
