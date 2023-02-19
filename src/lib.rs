#[macro_use]
extern crate quote;

#[macro_use]
extern crate syn;

extern crate proc_macro;

use std::collections::HashMap;

use once_cell::unsync::Lazy;
use proc_macro::TokenStream;
use syn::{DeriveInput, parse::Parser};

static mut _FIELD_DEFS: Lazy<HashMap<String, Vec<String>>> = Lazy::new(|| HashMap::new());

#[proc_macro_attribute]
pub fn extendable(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as DeriveInput);
    let original_ident = ast.ident;


    ast.ident = syn::parse_str::<syn::Ident>(&format!("{original_ident}_")).unwrap();

    let fields = match &ast.data {
        syn::Data::Struct(syn::DataStruct { fields: syn::Fields::Named(fields), .. }) => &fields.named,
        _ => panic!("extendable can only be implemented on structs with named fields"),
    };

    let func_definitions = fields.iter().map(|field| { 
        let ty = &field.ty;
        let ident = &field.ident.clone().unwrap();

        let setter_ident = syn::Ident::new(&format!("set_{ident}"), ident.span());
        let getter_ident = syn::Ident::new(&format!("get_{ident}"), ident.span());

        quote! {
            fn #setter_ident (&mut self, #ident: #ty);
            fn #getter_ident (&self) -> #ty;
        }
    });

    let field_defs: Vec<String> = fields.iter().map(|field| {
        let ident = &field.ident.clone().unwrap();
        let ty = &field.ty;
        
        // let field_ident = syn::Ident::new(&format!("_{original_ident}_{ident}_"), ident.span()); 
        quote! {
            #ident: #ty
        }.to_string()
    }).collect();

    unsafe { _FIELD_DEFS.insert(original_ident.clone().to_string(), field_defs); }

    quote! {
        #ast
        pub trait #original_ident {
            #(#func_definitions)*
        }
    }.into()
}

#[proc_macro_attribute]
pub fn extend(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as DeriveInput);
    let ident_arg = args.to_string();
    let extend_ident = syn::parse_str::<syn::Ident>(&ident_arg).unwrap();
    let field_defs: &Vec<String> = match unsafe { _FIELD_DEFS.get(&extend_ident.clone().to_string()) } {
        Some(f) => f,
        None => panic!("extend can only be used with structs that have implemented the extendable attribute {:?}", unsafe {_FIELD_DEFS.keys() }),
    };

    match &mut ast.data {
        syn::Data::Struct(ref mut struct_data) => {
            match &mut struct_data.fields {
                syn::Fields::Named(fields) => {
                    for fd in field_defs {
                        if let Ok(field_def) = syn::Field::parse_named.parse_str(fd) {
                            fields.named.push(field_def);
                        }
                        else {
                            panic!("Failed to parse field definition: {fd}")
                        }
                    }
                },
                _ => {()}
            }
        },
        _ => panic!("extend must be attributed to a struct"),
    }

    quote! {
        #ast
    }.into()
}