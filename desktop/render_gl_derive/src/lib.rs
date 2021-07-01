#![recursion_limit="128"]

extern crate proc_macro;
extern crate proc_macro2;

#[macro_use] extern crate syn;
#[macro_use] extern crate quote;

use std::collections::HashSet;
use proc_macro::Ident;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Token;
use syn::{Result, DeriveInput, Fields, Meta, MetaNameValue};


#[proc_macro_derive(VertexAttribPointers, attributes(location, divisor))]
pub fn vertex_attrib_pointers_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast:DeriveInput = parse_macro_input!(input as DeriveInput);

    // Build the impl
    let gen = generate_impl(&ast);

    // Return the generated impl
    proc_macro::TokenStream::from(gen)
}

fn generate_impl(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let ident = &ast.ident;
    let generics = &ast.generics;
    let where_clause = &ast.generics.where_clause;
    let fields_vertex_attrib_pointer
        = generate_vertex_attrib_pointer_calls(&ast.data);

    fn generate_vertex_attrib_pointer_calls(body: &syn::Data) -> Vec<proc_macro2::TokenStream> {
        match &body {
            &syn::Data::Union(_)
            => panic!("VertexAttribPointers can not be implemented for union"),
            &syn::Data::Enum(_)
            => panic!("VertexAttribPointers can not be implemented for enums"),
            &syn::Data::Struct(data) => {
                match data.fields{
                    Fields::Named(ref fields) => fields.named.iter()
                        .map(generate_struct_field_vertex_attrib_pointer_call)
                        .collect(),
                    Fields::Unnamed(_) => panic!("VertexAttribPointers can not be implemented for tuples"),
                    Fields::Unit => panic!("VertexAttribPointers can not be implemented for unit structs"),
                }
            }
        }
    }
    fn generate_struct_field_vertex_attrib_pointer_call(field: &syn::Field) -> proc_macro2::TokenStream  {
        let field_name = match field.ident {
            Some(ref i) => format!("{}", i),
            None => unreachable!("This macro should not be defined for tuples"),
        };
        fn find_attr(field: &syn::Field, attr_name:&str) -> Option<syn::Lit> {
            let location_attr = field.attrs
                .iter()
                .find(|a| a.path.is_ident(attr_name))?;
            let meta = location_attr.parse_meta().unwrap();
            let value = match &meta {
                Meta::NameValue(MetaNameValue { lit: lit @ syn::Lit::Int(_), .. }) => lit,
                _ => panic!("Field {} location attribute value must be an integer literal", field.ident.as_ref().unwrap())
            };
            Some(value.clone())
        }
        let location_value_literal = find_attr(field, "location").unwrap_or_else(|| panic!(
            "Field {:?} is missing #[location = ?] attribute", field_name));
        let divisor_value_literal = find_attr(field, "divisor");

        let field_ty = &field.ty;
        if let Some(divisor_value_literal) = divisor_value_literal{
            quote! {
                let location = #location_value_literal;
                unsafe {
                    #field_ty::vertex_attrib_pointer(gl, stride, location, offset);
                    gl.VertexAttribDivisor(location as gl::types::GLuint, #divisor_value_literal);
                }
                let offset = offset + ::std::mem::size_of::<#field_ty>();
            }
        }else {
            quote! {
                let location = #location_value_literal;
                unsafe {
                    #field_ty::vertex_attrib_pointer(gl, stride, location, offset);
                }
                let offset = offset + ::std::mem::size_of::<#field_ty>();
            }
        }


    }

    quote!{
        impl VertexAttribPointers for #ident #generics #where_clause {
            #[allow(unused_variables)]
            fn vertex_attrib_pointers(gl: &::gl::Gl){
                let offset:usize = 0;
                let stride = ::std::mem::size_of::<Self>(); // byte offset between consecutive attributes
                #(#fields_vertex_attrib_pointer)*
            }
        }
    }



}