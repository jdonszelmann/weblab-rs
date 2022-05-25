use crate::{parse_attr, Attr, FindAnnotated, ParseAttrStatus};
use proc_macro::TokenStream;
use quote::quote;
use std::mem;
use syn::fold::Fold;
use syn::ItemMod;

pub fn process_programming_assignment(attributes: &[Attr], item: TokenStream) -> TokenStream {
    let mut attrs = attributes.to_vec();
    let mut non_parsed_attrs = Vec::new();

    let mut module = syn::parse_macro_input!(item as ItemMod);

    let module_attrs = mem::take(&mut module.attrs);
    for i in module_attrs {
        match parse_attr(i) {
            Ok(ParseAttrStatus::Attr(i)) => attrs.extend(i),
            Ok(ParseAttrStatus::Doc(i, a)) => {
                attrs.push(i);
                non_parsed_attrs.push(a);
            }
            Ok(ParseAttrStatus::NotParsed(a)) => {
                non_parsed_attrs.push(a);
            }
            Err(e) => return e,
        }
    }

    let mut reference = FindAnnotated::reference();
    let mut template = FindAnnotated::template();

    let reference_modified = reference.fold_item_mod(module.clone());
    let _ = template.fold_item_mod(module); // for side effects

    let mut title = reference_modified.ident.to_string();
    let mut num_titles = 0;
    for i in &attrs {
        if let Attr::Title(x) = i {
            title = x.clone();
            num_titles += 1;
        }
    }

    if num_titles > 1 {
        return quote! {
            compile_error!("assignment has more than one title");
        }
        .into();
    }

    let spectest = if let Some(i) = reference.test().map(|i| quote! {#(#i)*}.to_string()) {
        i
    } else {
        return quote! {
            compile_error!("assignment has no spectest");
        }
        .into();
    };
    let testtemplate = template
        .test()
        .map(|i| quote! {#(#i)*}.to_string())
        .unwrap_or_else(|| "".to_string());
    let referencesolution =
        if let Some(i) = reference.solution().map(|i| quote! {#(#i)*}.to_string()) {
            i
        } else {
            return quote! {
                compile_error!("assignment has no reference solution");
            }
            .into();
        };
    let solutiontemplate = template
        .solution()
        .map(|i| quote! {#(#i)*}.to_string())
        .unwrap_or_else(|| "".to_string());
    let library = if let Some(i) = template.library().map(|i| quote! {#(#i)*}.to_string()) {
        quote! {
            Some(#i)
        }
    } else {
        quote! {
            None
        }
    };

    let assignment_text = attrs
        .iter()
        .filter_map(|x| {
            if let Attr::Doc(i) = x {
                Some(i.as_str())
            } else {
                None
            }
        })
        .map(|i| i.trim())
        .collect::<Vec<_>>()
        .join("\n");

    quote! {
        pub mod __WEBLAB_ASSIGNMENT_METADATA {
            use weblab::*;

            pub const ASSIGNMENT_INFO: WeblabAssignment = WeblabAssignment::Programming(ProgrammingAssignment {
                title: #title,

                assignment_text: #assignment_text,

                library_visible: false,
                spectest_stdout_visible: false,

                test: #spectest,
                solution: #referencesolution,

                library: #library,
                test_template: #testtemplate,
                solution_template: #solutiontemplate,

                checklist: None,
            });
        }

        #[allow(unused_imports)]
        #[allow(dead_code)]
        #reference_modified
    }.into()
}
