use crate::attr::{Attr, parse_attr, parse_attr_stream, ParseAttrStatus};
use crate::Attr::{Solution, SolutionTemplate};
use proc_macro::{Span, TokenStream};
use proc_macro2::Span as Span2;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned, ToTokens};
use std::mem;
use syn::fold::{Fold, fold_item};
use syn::spanned::Spanned;
use syn::UseTree;
use fold_programming_input::FindAnnotated;

mod attr;
mod mc;
mod open;
mod programming;
mod fold_programming_input;

const ALLOWED_CRATES: &[&str] = [
    "serde",
    "lazy_static",
    "async-trait",
    "futures",
    "tokio",
    "log",
    "pretty_env_logger",
    "rand",
    "regex",
    "serde_json",
    "itertools",
    "parking_lot",
    "petgraph",
    "quickcheck",
    "quickcheck_macros",
]
.as_slice();

#[proc_macro]
pub fn open_question(_item: TokenStream) -> TokenStream {
    "".parse().unwrap()
}

#[proc_macro]
pub fn mc_question(_item: TokenStream) -> TokenStream {
    "".parse().unwrap()
}


#[proc_macro_attribute]
pub fn weblab(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = match parse_attr_stream(attr) {
        Ok(i) => i,
        Err(e) => return e,
    };

    let res = if let Some(Attr::ProgrammingAssignment) = attr.first() {
        programming::process_programming_assignment(&attr[1..], item)
    } else {
        return syn::Error::new(
            Span::call_site().into(),
            "#[weblab(programming_assignment)] always needs to be the first attribute \
            on a module containing the solution, test and library. Other attributes, \
            #[weblab(...)] attributes and doc comments need to be below it or inside the \
            module that's annotated with #[weblab(programming_assignment)]",
        )
        .to_compile_error()
        .into();
    };

    res
}
