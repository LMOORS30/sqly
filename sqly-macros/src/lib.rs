#![cfg(feature = "postgres")]

mod parse;
mod quote;
mod derive;

use syn::DeriveInput;
use syn::parse_macro_input;
use proc_macro::TokenStream;

#[proc_macro_derive(Table, attributes(sqly))]
pub fn derive_table(tokens: TokenStream) -> TokenStream {
    match derive::table(parse_macro_input!(tokens as DeriveInput)) {
        Err(err) => err.to_compile_error().into(),
        Ok(stream) => stream.into(),
    }
}

#[proc_macro_derive(Delete, attributes(sqly))]
pub fn derive_delete(tokens: TokenStream) -> TokenStream {
    match derive::delete(parse_macro_input!(tokens as DeriveInput)) {
        Err(err) => err.to_compile_error().into(),
        Ok(stream) => stream.into(),
    }
}

#[proc_macro_derive(Insert, attributes(sqly))]
pub fn derive_insert(tokens: TokenStream) -> TokenStream {
    match derive::insert(parse_macro_input!(tokens as DeriveInput)) {
        Err(err) => err.to_compile_error().into(),
        Ok(stream) => stream.into(),
    }
}

#[proc_macro_derive(Select, attributes(sqly))]
pub fn derive_select(tokens: TokenStream) -> TokenStream {
    match derive::select(parse_macro_input!(tokens as DeriveInput)) {
        Err(err) => err.to_compile_error().into(),
        Ok(stream) => stream.into(),
    }
}

#[proc_macro_derive(Update, attributes(sqly))]
pub fn derive_update(tokens: TokenStream) -> TokenStream {
    match derive::update(parse_macro_input!(tokens as DeriveInput)) {
        Err(err) => err.to_compile_error().into(),
        Ok(stream) => stream.into(),
    }
}
