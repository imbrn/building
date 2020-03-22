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
                Some(ident) => Some(FieldDescriptor::parse(&ident, &field)),
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
    field: &'a syn::Field,
    angle_bracketed_type: Option<(&'a syn::Ident, &'a syn::AngleBracketedGenericArguments)>,
    config: HashMap<String, String>,
}

impl<'a> FieldDescriptor<'a> {
    fn parse(ident: &'a syn::Ident, field: &'a syn::Field) -> Self {
        Self {
            ident: &ident,
            field,
            angle_bracketed_type: Self::parse_angle_bracketed_type(&field),
            config: Self::parse_config(&field),
        }
    }

    fn parse_angle_bracketed_type(
        field: &'a syn::Field,
    ) -> Option<(&'a syn::Ident, &'a syn::AngleBracketedGenericArguments)> {
        match &field.ty {
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

    fn parse_config(field: &'a syn::Field) -> HashMap<String, String> {
        let mut config = HashMap::new();

        &field
            .attrs
            .iter()
            .filter(|attr_ref| {
                attr_ref
                    .path
                    .get_ident()
                    .map_or(false, |ident| ident == "builder")
            })
            .filter_map(|attr_ref| attr_ref.parse_meta().ok())
            .filter_map(|meta| match meta {
                syn::Meta::List(meta_list) => Some(meta_list.nested),
                _ => None,
            })
            .for_each(|nested_data| {
                nested_data.iter().for_each(|nested_data| {
                    if let syn::NestedMeta::Meta(nested_data) = nested_data {
                        match nested_data {
                            syn::Meta::NameValue(name_value) => match &name_value.lit {
                                syn::Lit::Str(lit_str) => {
                                    config.insert(
                                        name_value.path.get_ident().unwrap().to_string(),
                                        lit_str.value(),
                                    );
                                }
                                _ => {}
                            },
                            _ => {}
                        }
                    }
                });
            });

        config
    }

    fn struct_field_init(&self) -> impl quote::ToTokens {
        let ident = &self.ident;

        if self.is_optional_type() || self.is_vector_type() {
            quote! { #ident: builder.#ident.clone() }
        } else {
            quote! { #ident: builder.#ident.clone().ok_or("Failed to build field".to_owned())? }
        }
    }

    fn builder_field_decl(&self) -> impl quote::ToTokens {
        let ident = self.ident;
        let ty = self.get_main_type();

        if self.is_optional_type() {
            quote! { pub #ident: std::option::Option<#ty> }
        } else if self.is_vector_type() {
            quote! { pub #ident: std::vec::Vec<#ty> }
        } else {
            quote! { pub #ident: std::option::Option<#ty> }
        }
    }

    fn builder_field_init(&self) -> impl quote::ToTokens {
        let ident = self.ident;

        if self.is_vector_type() {
            quote! { #ident: std::vec::Vec::new() }
        } else {
            quote! { #ident: std::option::Option::None }
        }
    }

    fn builder_setter(&self) -> impl quote::ToTokens {
        let ident = self.ident;
        let each_setter = self.config.get("each");
        let ty = self.get_main_type();

        let each_setter = if each_setter.is_some() && self.is_vector_type() {
            let each_setter = quote::format_ident!("{}", each_setter.unwrap());
            quote!{
                pub fn #each_setter(&mut self, value: #ty) -> &mut Self {
                    self.#ident.push(value);
                    self
                }
            }
        } else {
            quote!{}
        };

        let normal_setter = if self.is_vector_type() {
            quote! {
                pub fn #ident(&mut self, value: std::vec::Vec<#ty>) -> &mut Self {
                    self.#ident = value;
                    self
                }
            }
        } else {
            quote! {
                pub fn #ident(&mut self, value: #ty) -> &mut Self {
                    self.#ident = std::option::Option::Some(value);
                    self
                }
            }
        };

        quote! {
            #each_setter
            #normal_setter
        }
    }

    fn is_optional_type(&self) -> bool {
        match &self.angle_bracketed_type {
            Some((ident, _)) => ident == &"Option",
            _ => false,
        }
    }

    fn is_vector_type(&self) -> bool {
        match &self.angle_bracketed_type {
            Some((ident, _)) => ident == &"Vec",
            _ => false,
        }
    }

    fn get_main_type(&'a self) -> &'a syn::Type {
        match &self.angle_bracketed_type {
            Some((_, bracketed_args)) => {
                Self::get_nth_angle_bracketed_type(0, &bracketed_args).unwrap_or(&self.field.ty)
            }
            _ => &self.field.ty,
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
