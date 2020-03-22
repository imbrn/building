#![allow(unused)]

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashMap;
use std::vec::Vec;
use syn;

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let descriptor = get_descriptor(&input).unwrap();
    descriptor.to_token_stream().unwrap()
}

fn get_descriptor(input: &syn::DeriveInput) -> Result<Box<dyn Descriptor>, TokenizingError> {
    match &input.data {
        syn::Data::Struct(data) => Ok(Box::new(StructDescriptor::new(&input.ident, &data))),
        _ => Err(TokenizingError::new("Only structs are supported yet")),
    }
}

trait Descriptor {
    fn to_token_stream(&self) -> Result<TokenStream, TokenizingError>;
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

    fn parse_fields<'a>(&'a self) -> Result<Vec<BoxedFieldDescriptor>, TokenizingError> {
        match &self.data.fields {
            syn::Fields::Named(fields_named) => Ok(fields_named
                .named
                .iter()
                .filter_map(|field| match &field.ident {
                    Some(ident) => resolve_field_descriptor(&ident, &field).ok(),
                    None => None,
                })
                .collect()),
            _ => Err(TokenizingError::new("Only named fields are supported yet")),
        }
    }
}

impl Descriptor for StructDescriptor {
    fn to_token_stream(&self) -> Result<TokenStream, TokenizingError> {
        let fields = self.parse_fields()?;

        let struct_ident = &self.ident;
        let builder_ident = quote::format_ident!("{}Builder", &struct_ident);

        let mut struct_fields_init: Vec<Box<dyn quote::ToTokens>> = vec![];
        let mut builder_fields_decl: Vec<Box<dyn quote::ToTokens>> = vec![];
        let mut builder_fields_init: Vec<Box<dyn quote::ToTokens>> = vec![];
        let mut builder_setters: Vec<Box<dyn quote::ToTokens>> = vec![];

        for field in &fields {
            struct_fields_init.push(field.struct_field_init()?);
            builder_fields_decl.push(field.builder_field_decl()?);
            builder_fields_init.push(field.builder_field_init()?);
            builder_setters.push(field.builder_setter()?);
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

fn resolve_field_descriptor<'a>(
    ident: &'a syn::Ident,
    field: &'a syn::Field,
) -> Result<BoxedFieldDescriptor<'a>, String> {
    let angle_bracketed_type = get_angle_bracketed_type(&field.ty);

    match angle_bracketed_type {
        Some((bracket_ident, angle_bracketed_type)) => {
            if let Some(main_type) = get_nth_angle_bracketed_type(0, &angle_bracketed_type) {
                if VectorFieldDescriptor::processes(&bracket_ident, &main_type) {
                    return Ok(Box::new(VectorFieldDescriptor::new(
                        &ident,
                        &main_type,
                        &field.attrs,
                    )));
                }
                if OptionFieldDescriptor::processes(&bracket_ident, &main_type) {
                    return Ok(Box::new(OptionFieldDescriptor::new(&ident, &main_type)));
                }
            }
        }
        None => {}
    }

    Ok(Box::new(BasicFieldDescriptor::new(&ident, &field.ty)))
}

fn get_angle_bracketed_type<'a>(
    ty: &'a syn::Type,
) -> Option<(&'a syn::Ident, &'a syn::AngleBracketedGenericArguments)> {
    match ty {
        syn::Type::Path(type_path) => match type_path.path.segments.first() {
            Some(segment) => match &segment.arguments {
                syn::PathArguments::AngleBracketed(args) => Some((&segment.ident, args)),
                _ => None,
            },
            _ => None,
        },
        _ => None,
    }
}

fn get_nth_angle_bracketed_type(
    nth: usize,
    bracketed_args: &syn::AngleBracketedGenericArguments,
) -> Option<&syn::Type> {
    match bracketed_args.args.first() {
        Some(arg) => match arg {
            syn::GenericArgument::Type(ty) => Some(ty),
            _ => None,
        },
        _ => None,
    }
}

type BoxedFieldDescriptor<'a> = Box<dyn FieldDescriptor + 'a>;
type BoxedToTokens = Box<dyn quote::ToTokens>;

trait FieldDescriptor {
    fn struct_field_init(&self) -> Result<BoxedToTokens, TokenizingError>;
    fn builder_field_decl(&self) -> Result<BoxedToTokens, TokenizingError>;
    fn builder_field_init(&self) -> Result<BoxedToTokens, TokenizingError>;
    fn builder_setter(&self) -> Result<BoxedToTokens, TokenizingError>;
}

struct BasicFieldDescriptor<'a> {
    ident: &'a syn::Ident,
    ty: &'a syn::Type,
}

impl<'a> BasicFieldDescriptor<'a> {
    fn new(ident: &'a syn::Ident, ty: &'a syn::Type) -> Self {
        Self { ident, ty }
    }
}

impl<'a> FieldDescriptor for BasicFieldDescriptor<'a> {
    fn struct_field_init(&self) -> Result<BoxedToTokens, TokenizingError> {
        let ident = &self.ident;
        Ok(Box::new(
            quote! { #ident: builder.#ident.clone().ok_or("Failed to build field".to_owned())? },
        ))
    }

    fn builder_field_decl(&self) -> Result<BoxedToTokens, TokenizingError> {
        let ident = &self.ident;
        let ty = &self.ty;
        Ok(Box::new(quote! { pub #ident: std::option::Option<#ty> }))
    }

    fn builder_field_init(&self) -> Result<BoxedToTokens, TokenizingError> {
        let ident = &self.ident;
        Ok(Box::new(quote! { #ident: std::option::Option::None }))
    }

    fn builder_setter(&self) -> Result<BoxedToTokens, TokenizingError> {
        let ident = &self.ident;
        let ty = &self.ty;
        Ok(Box::new(quote! {
            pub fn #ident(&mut self, value: #ty) -> &mut Self {
                self.#ident = std::option::Option::Some(value);
                self
            }
        }))
    }
}

struct OptionFieldDescriptor<'a> {
    ident: &'a syn::Ident,
    optional_type: &'a syn::Type,
}

impl<'a> OptionFieldDescriptor<'a> {
    fn processes(ident: &syn::Ident, main_type: &syn::Type) -> bool {
        ident == &"Option"
    }

    fn new(ident: &'a syn::Ident, optional_type: &'a syn::Type) -> Self {
        Self {
            ident,
            optional_type,
        }
    }
}

impl<'a> FieldDescriptor for OptionFieldDescriptor<'a> {
    fn struct_field_init(&self) -> Result<BoxedToTokens, TokenizingError> {
        let ident = &self.ident;
        Ok(Box::new(quote! { #ident: builder.#ident.clone() }))
    }

    fn builder_field_decl(&self) -> Result<BoxedToTokens, TokenizingError> {
        let ident = &self.ident;
        let ty = &self.optional_type;
        Ok(Box::new(quote! { pub #ident: std::option::Option<#ty> }))
    }

    fn builder_field_init(&self) -> Result<BoxedToTokens, TokenizingError> {
        let ident = &self.ident;
        Ok(Box::new(quote! { #ident: std::option::Option::None }))
    }

    fn builder_setter(&self) -> Result<BoxedToTokens, TokenizingError> {
        let ident = &self.ident;
        let ty = &self.optional_type;
        Ok(Box::new(quote! {
            pub fn #ident(&mut self, value: #ty) -> &mut Self {
                self.#ident = std::option::Option::Some(value);
                self
            }
        }))
    }
}

struct VectorFieldDescriptor<'a> {
    ident: &'a syn::Ident,
    item_type: &'a syn::Type,
    each_ident: Option<String>,
}

impl<'a> VectorFieldDescriptor<'a> {
    fn processes(ident: &syn::Ident, main_type: &syn::Type) -> bool {
        ident == &"Vec"
    }

    fn new(
        ident: &'a syn::Ident,
        item_type: &'a syn::Type,
        attrs: &'a Vec<syn::Attribute>,
    ) -> Self {
        Self {
            ident,
            item_type,
            each_ident: Self::resolve_each_config(&attrs),
        }
    }

    fn resolve_each_config(attrs: &'a Vec<syn::Attribute>) -> Option<String> {
        let expected_attr_ident = quote::format_ident!("builder");

        for attr in attrs.iter() {
            if let Some(config) = attr.parse_args::<AttrConfig>().ok() {
                if attr.path.get_ident() == Some(&expected_attr_ident) && config.ident == &"each" {
                    return Some(config.lit.value().to_string());
                }
            }
        }

        None
    }
}

struct AttrConfig {
    ident: syn::Ident,
    sep: syn::Token!(=),
    lit: syn::LitStr,
}

impl syn::parse::Parse for AttrConfig {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            ident: input.parse()?,
            sep: input.parse()?,
            lit: input.parse()?,
        })
    }
}

impl<'a> FieldDescriptor for VectorFieldDescriptor<'a> {
    fn struct_field_init(&self) -> Result<BoxedToTokens, TokenizingError> {
        let ident = &self.ident;
        Ok(Box::new(quote! { #ident: builder.#ident.clone() }))
    }

    fn builder_field_decl(&self) -> Result<BoxedToTokens, TokenizingError> {
        let ident = &self.ident;
        let ty = &self.item_type;
        Ok(Box::new(quote! { pub #ident: std::vec::Vec<#ty> }))
    }

    fn builder_field_init(&self) -> Result<BoxedToTokens, TokenizingError> {
        let ident = &self.ident;
        Ok(Box::new(quote! { #ident: std::vec::Vec::new() }))
    }

    fn builder_setter(&self) -> Result<BoxedToTokens, TokenizingError> {
        let ident = &self.ident;
        let ty = &self.item_type;
        let each_ident = &self.each_ident;

        let each_setter = if let Some(each_ident) = each_ident {
            let each_ident = quote::format_ident!("{}", each_ident);
            quote! {
                pub fn #each_ident(&mut self, value: #ty) -> &mut Self {
                    self.#ident.push(value);
                    self
                }
            }
        } else {
            quote! {}
        };

        let normal_setter = quote! {
            pub fn #ident(&mut self, value: std::vec::Vec<#ty>) -> &mut Self {
                self.#ident = value;
                self
            }
        };

        Ok(Box::new(quote! {
            #each_setter
            #normal_setter
        }))
    }
}

#[derive(Debug)]
struct TokenizingError {
    message: String,
}

impl TokenizingError {
    fn new(msg: &str) -> TokenizingError {
        TokenizingError {
            message: msg.to_owned(),
        }
    }
}
