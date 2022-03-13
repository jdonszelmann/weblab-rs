use crate::attr::{parse_attr, parse_attr_stream, Attr, ParseAttrStatus};
use crate::Attr::{Solution, SolutionTemplate};
use proc_macro::{Span, TokenStream};
use proc_macro2::Span as Span2;
use quote::{quote, quote_spanned, ToTokens};
use std::mem;
use syn::__private::TokenStream2;
use syn::fold::{fold_item, Fold};
use syn::spanned::Spanned;
use syn::{
    Item, ItemConst, ItemEnum, ItemExternCrate, ItemFn, ItemForeignMod, ItemImpl, ItemMacro,
    ItemMacro2, ItemMod, ItemStatic, ItemStruct, ItemTrait, ItemTraitAlias, ItemType, ItemUnion,
    ItemUse,
};

mod attr;

#[proc_macro]
pub fn open_question(_item: TokenStream) -> TokenStream {
    "".parse().unwrap()
}

#[proc_macro]
pub fn mc_question(_item: TokenStream) -> TokenStream {
    "".parse().unwrap()
}

fn process_programming_assignment(attributes: &[Attr], item: TokenStream) -> TokenStream {
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

    let res = reference.fold_item_mod(module.clone());
    let _ = template.fold_item_mod(module); // for side effects

    let mut title = res.ident.to_string();
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

    let description: String = attrs
        .iter()
        .filter_map(|x| {
            if let Attr::Doc(i) = x {
                Some(i.as_str())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    quote! {
        pub mod __WEBLAB_ASSIGNMENT_METADATA {
            use weblab::*;

            pub const ASSIGNMENT_INFO: WeblabAssignment = WeblabAssignment::Programming(ProgrammingAssignment {
                title: #title,

                description: #description,

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
        #res
    }.into()
}

enum Status {
    Certain(ItemMod),
    Maybe(ItemMod),
    Unkown,
}

enum FindAnnotated {
    Template {
        solution: Status,
        test: Status,
        library: Option<ItemMod>,
    },
    Reference {
        solution: Option<ItemMod>,
        test: Option<ItemMod>,
    },
}

impl FindAnnotated {
    pub fn test(&self) -> Option<Vec<Item>> {
        match self {
            FindAnnotated::Template { test, .. } => match test {
                Status::Certain(i) => i.content.clone().map(|(_, x)| x),
                Status::Maybe(i) => i.content.clone().map(|(_, x)| x),
                Status::Unkown => None,
            },
            FindAnnotated::Reference { test, .. } => {
                test.clone().and_then(|i| i.content).map(|(_, x)| x)
            }
        }
    }

    pub fn solution(&self) -> Option<Vec<Item>> {
        match self {
            FindAnnotated::Template { solution, .. } => match solution {
                Status::Certain(i) => i.content.clone().map(|(_, x)| x),
                Status::Maybe(i) => i.content.clone().map(|(_, x)| x),
                Status::Unkown => None,
            },
            FindAnnotated::Reference { solution, .. } => {
                solution.clone().and_then(|i| i.content).map(|(_, x)| x)
            }
        }
    }

    pub fn library(&self) -> Option<Vec<Item>> {
        match self {
            FindAnnotated::Template { library, .. } => {
                library.clone().and_then(|i| i.content).map(|(_, x)| x)
            }
            FindAnnotated::Reference { .. } => None,
        }
    }

    pub fn reference() -> Self {
        Self::Reference {
            solution: None,
            test: None,
        }
    }

    pub fn template() -> Self {
        Self::Template {
            solution: Status::Unkown,
            test: Status::Unkown,
            library: None,
        }
    }

    pub fn is_reference(&self) -> bool {
        match self {
            FindAnnotated::Template { .. } => false,
            FindAnnotated::Reference { .. } => true,
        }
    }

    pub fn is_template(&self) -> bool {
        !self.is_reference()
    }
}

impl Fold for FindAnnotated {
    fn fold_item(&mut self, mut i: Item) -> Item {
        let attrs = match &mut i {
            Item::Const(ItemConst { attrs, .. })
            | Item::Enum(ItemEnum { attrs, .. })
            | Item::ExternCrate(ItemExternCrate { attrs, .. })
            | Item::Fn(ItemFn { attrs, .. })
            | Item::ForeignMod(ItemForeignMod { attrs, .. })
            | Item::Impl(ItemImpl { attrs, .. })
            | Item::Macro(ItemMacro { attrs, .. })
            | Item::Macro2(ItemMacro2 { attrs, .. })
            | Item::Mod(ItemMod { attrs, .. })
            | Item::Static(ItemStatic { attrs, .. })
            | Item::Struct(ItemStruct { attrs, .. })
            | Item::Trait(ItemTrait { attrs, .. })
            | Item::TraitAlias(ItemTraitAlias { attrs, .. })
            | Item::Type(ItemType { attrs, .. })
            | Item::Union(ItemUnion { attrs, .. })
            | Item::Use(ItemUse { attrs, .. }) => {
                let mut parsed_attrs = Vec::new();
                let mut non_parsed_attrs = Vec::new();

                for i in attrs.drain(..) {
                    match parse_attr(i) {
                        Ok(ParseAttrStatus::Attr(i)) => parsed_attrs.extend(i),
                        Ok(ParseAttrStatus::Doc(i, a)) => {
                            parsed_attrs.push(i);
                            non_parsed_attrs.push(a);
                        }
                        Ok(ParseAttrStatus::NotParsed(a)) => {
                            non_parsed_attrs.push(a);
                        }
                        Err(e) => return Item::Verbatim(e.into()),
                    }
                }

                *attrs = non_parsed_attrs;
                parsed_attrs
            }
            Item::Verbatim(_ts) => {
                vec![]
            }
            _ => return Item::Verbatim(r#"compile_error!("not implemented")"#.to_token_stream()),
        };

        if let Item::Macro(ItemMacro { mac, .. }) = &i {
            if let Some(i) = mac.path.get_ident() {
                if i == "template_only" && self.is_reference() {
                    return Item::Verbatim(TokenStream2::new());
                }
                if i == "solution_only" && self.is_template() {
                    return Item::Verbatim(TokenStream2::new());
                }
            }
        }

        if let Item::Mod(ref i) = i {
            match self {
                FindAnnotated::Template {
                    solution,
                    test,
                    library,
                } => {
                    if attrs.contains(&SolutionTemplate) {
                        match solution {
                            Status::Certain(_) => {
                                return Item::Verbatim(quote_spanned! {
                                    i.span() =>
                                    compile_error!("multiple solution template blocks defined")
                                })
                            }
                            Status::Maybe(_) | Status::Unkown => {
                                *solution = Status::Certain(i.clone());
                            }
                        }
                    } else if attrs.contains(&Solution) {
                        match solution {
                            Status::Certain(_) | Status::Maybe(_) => {
                                return Item::Verbatim(quote_spanned! {
                                    i.span() =>
                                    compile_error!("multiple solution template blocks defined")
                                })
                            }
                            Status::Unkown => {
                                *solution = Status::Maybe(i.clone());
                            }
                        }
                    } else if attrs.contains(&Attr::TestTemplate) {
                        match test {
                            Status::Certain(_) => {
                                return Item::Verbatim(quote_spanned! {
                                    i.span() =>
                                    compile_error!("multiple test template blocks defined")
                                })
                            }
                            Status::Maybe(_) | Status::Unkown => {
                                *test = Status::Certain(i.clone());
                            }
                        }
                    } else if attrs.contains(&Attr::Test) {
                        match test {
                            Status::Certain(_) | Status::Maybe(_) => {
                                return Item::Verbatim(quote_spanned! {
                                    i.span() =>
                                    compile_error!("multiple test template blocks defined")
                                })
                            }
                            Status::Unkown => {
                                *test = Status::Maybe(i.clone());
                            }
                        }
                    } else if attrs.contains(&Attr::Library) {
                        if library.is_none() {
                            *library = Some(i.clone());
                        } else {
                            return Item::Verbatim(quote_spanned! {
                                i.span() =>
                                compile_error!("multiple library blocks defined")
                            });
                        }
                    }
                }
                FindAnnotated::Reference { solution, test } => {
                    if attrs.contains(&Attr::Solution) {
                        if solution.is_none() {
                            *solution = Some(i.clone());
                        } else {
                            return Item::Verbatim(quote_spanned! {
                                i.span() =>
                                compile_error!("multiple reference solution blocks defined")
                            });
                        }
                    } else if attrs.contains(&Attr::Test) {
                        if test.is_none() {
                            *test = Some(i.clone());
                        } else {
                            return Item::Verbatim(quote_spanned! {
                                i.span() =>
                                compile_error!("multiple spec test blocks defined")
                            });
                        }
                    }
                }
            }
        }

        fold_item(self, i)
    }
}

#[proc_macro_attribute]
pub fn weblab(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = match parse_attr_stream(attr) {
        Ok(i) => i,
        Err(e) => return e,
    };

    let res = if let Some(Attr::ProgrammingAssignment) = attr.first() {
        process_programming_assignment(&attr[1..], item)
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
