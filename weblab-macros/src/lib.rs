use crate::attr::{parse_attr, parse_attr_stream, Attr, ParseAttrStatus};
use crate::inline_question_list::InlineQuestionList;
use crate::Attr::{Solution, SolutionTemplate};
use fold_programming_input::FindAnnotated;
use mc::McQuestion;
use open::OpenQuestion;
use proc_macro::{Span, TokenStream};
use proc_macro2::Span as Span2;
use quote::{format_ident, quote};
use syn::parse_macro_input;

mod attr;
mod fold_programming_input;
mod inline_question_list;
mod mc;
mod open;
mod programming;

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
    "std",
    "core",
    "alloc",
    "test",
]
.as_slice();

#[proc_macro]
pub fn inline_question_list(item: TokenStream) -> TokenStream {
    let InlineQuestionList {
        title,
        question_text,
        questions,
    } = parse_macro_input!(item as InlineQuestionList);

    let assignment_names = questions
        .iter()
        .enumerate()
        .map(|(num, _)| format_ident!("__INLINE_ASSIGNMENT_{num}"))
        .collect::<Vec<_>>();

    quote! {
        pub mod __WEBLAB_ASSIGNMENT_METADATA {
            use weblab::*;

            pub const ASSIGNMENT_INFO: WeblabAssignment = WeblabAssignment::InlineQuestionList(InlineQuestionList {
                title: #title,
                assignment_text: #question_text,
                assignments: &[#(
                    {
                        use super::*;
                        use #assignment_names as weblab_module;

                        weblab_module::__WEBLAB_ASSIGNMENT_METADATA::ASSIGNMENT_INFO
                    }
                ),*]
            });
        }

        #(
            pub mod #assignment_names {
                use weblab::{mc_question, open_question};
                #questions;
            }
        )*
    }.into()
}

#[proc_macro]
pub fn open_question(item: TokenStream) -> TokenStream {
    let OpenQuestion {
        title,
        question_text,
        answer,
    } = parse_macro_input!(item as OpenQuestion);

    if title.is_empty() {
        return quote! {compile_error!("expected title");}.into();
    }
    if question_text.text.is_empty() {
        return quote! {compile_error!("expected question");}.into();
    }

    quote! {
        pub mod __WEBLAB_ASSIGNMENT_METADATA {
            use weblab::*;

            pub const ASSIGNMENT_INFO: WeblabAssignment = WeblabAssignment::Open(OpenQuestion {
                title: #title,

                assignment_text: #question_text,
                expected_answer: #answer,

                checklist: None,
            });
        }
    }
    .into()
}

#[proc_macro]
pub fn mc_question(item: TokenStream) -> TokenStream {
    let McQuestion {
        title,
        question_text,
        options,
        num_answers_expected,
        randomize,
    } = parse_macro_input!(item as McQuestion);

    let answers: Vec<_> = options.iter().map(|i| i.text.clone()).collect();
    let corrects: Vec<_> = options.iter().map(|i| i.correct).collect();

    if question_text.text.is_empty() {
        return quote! {compile_error!("expected question text");}.into();
    }
    if title.is_empty() {
        return quote! {compile_error!("expected title");}.into();
    }
    if answers.is_empty() {
        return quote! {compile_error!("expected at least one option (using `option \"text\"`)");}
            .into();
    }
    if !corrects.iter().any(|i| *i) {
        return quote!{compile_error!("expected at least one option marked as correct (using `option \"text\" correct`)");}.into();
    }
    if num_answers_expected > answers.len() {
        let text = format!("you marked this question as requiring {num_answers_expected} but there are only {} options", answers.len());
        return quote! {compile_error!(#text);}.into();
    }

    let style = if num_answers_expected == 0 {
        quote! {MCStyle::AllThatApply}
    } else {
        quote! {MCStyle::NumCorrect(#num_answers_expected)}
    };

    quote! {
        pub mod __WEBLAB_ASSIGNMENT_METADATA {
            use weblab::*;

            pub const ASSIGNMENT_INFO: WeblabAssignment = WeblabAssignment::MultipleChoice(MCQuestion {
                title: #title,

                assignment_text: #question_text,

                options: &[#(
                    MCOption {
                        text: #answers,
                        is_correct: #corrects
                    }
                ),*],
                randomize: #randomize,
                style: #style,
            });
        }
    }.into()
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
