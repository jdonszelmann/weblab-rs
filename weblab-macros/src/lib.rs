use crate::attr::{parse_attr, parse_attr_stream, Attr, ParseAttrStatus};
use crate::Attr::{Solution, SolutionTemplate};
use proc_macro::{Span, TokenStream};
use proc_macro2::{Ident, Span as Span2};
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned, ToTokens};
use std::mem;
use syn::fold::{fold_item, Fold};
use syn::spanned::Spanned;
use syn::{Abi, AngleBracketedGenericArguments, Arm, Attribute, AttrStyle, BareFnArg, Binding, BinOp, Block, BoundLifetimes, ConstParam, Constraint, Data, DataEnum, DataStruct, DataUnion, DeriveInput, Expr, ExprArray, ExprAssign, ExprAssignOp, ExprAsync, ExprAwait, ExprBinary, ExprBlock, ExprBox, ExprBreak, ExprCall, ExprCast, ExprClosure, ExprContinue, ExprField, ExprForLoop, ExprGroup, ExprIf, ExprIndex, ExprLet, ExprLit, ExprLoop, ExprMacro, ExprMatch, ExprMethodCall, ExprParen, ExprPath, ExprRange, ExprReference, ExprRepeat, ExprReturn, ExprStruct, ExprTry, ExprTryBlock, ExprTuple, ExprType, ExprUnary, ExprUnsafe, ExprWhile, ExprYield, Field, FieldPat, Fields, FieldsNamed, FieldsUnnamed, FieldValue, File, FnArg, ForeignItem, ForeignItemFn, ForeignItemMacro, ForeignItemStatic, ForeignItemType, GenericArgument, GenericMethodArgument, GenericParam, Generics, ImplItem, ImplItemConst, ImplItemMacro, ImplItemMethod, ImplItemType, Index, Item, ItemConst, ItemEnum, ItemExternCrate, ItemFn, ItemForeignMod, ItemImpl, ItemMacro, ItemMacro2, ItemMod, ItemStatic, ItemStruct, ItemTrait, ItemTraitAlias, ItemType, ItemUnion, ItemUse, Label, Lifetime, LifetimeDef, Lit, LitBool, LitByte, LitByteStr, LitChar, LitFloat, LitInt, LitStr, Local, Macro, MacroDelimiter, Member, Meta, MetaList, MetaNameValue, MethodTurbofish, NestedMeta, ParenthesizedGenericArguments, Pat, PatBox, Path, PathArguments, PathSegment, PatIdent, PatLit, PatMacro, PatOr, PatPath, PatRange, PatReference, PatRest, PatSlice, PatStruct, PatTuple, PatTupleStruct, PatType, PatWild, PredicateEq, PredicateLifetime, PredicateType, QSelf, RangeLimits, Receiver, ReturnType, Signature, Stmt, TraitBound, TraitBoundModifier, TraitItem, TraitItemConst, TraitItemMacro, TraitItemMethod, TraitItemType, Type, TypeArray, TypeBareFn, TypeGroup, TypeImplTrait, TypeInfer, TypeMacro, TypeNever, TypeParam, TypeParamBound, TypeParen, TypePath, TypePtr, TypeReference, TypeSlice, TypeTraitObject, TypeTuple, UnOp, UseGlob, UseGroup, UseName, UsePath, UseRename, UseTree, Variadic, Variant, VisCrate, Visibility, VisPublic, VisRestricted, WhereClause, WherePredicate};

mod attr;

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

fn parse_only_contents(t: &TokenStream2) -> Item {
    // let res: Block = syn::parse2(quote! {"{ #tt }"})?;
    // Ok(Item)
    // TODO: parse as block so inner macros get expanded
    Item::Verbatim(t.to_token_stream())
}

fn should_drop(t: &UseTree) -> Result<bool, String> {
    match t {
        UseTree::Path(UsePath { ident, .. })
        | UseTree::Name(UseName { ident })
        | UseTree::Rename(UseRename { ident, .. }) => {
            if ident == "weblab" {
                return Ok(true);
            }

            if ALLOWED_CRATES.contains(&ident.to_string().as_str()) {
                Ok(false)
            } else if ident == "crate" {
                Err("crate-relative imports break on weblab since weblab's generated project structure will be different to this one. Use relative imports (with super)".to_string())
            } else if ident == "super" {
                Ok(false)
            } else {
                Err(format!(
                    "{ident} cannot be imported in weblab and is therefore forbidden"
                ))
            }
        }
        UseTree::Glob(_) => Ok(false),
        UseTree::Group(UseGroup { items, .. }) => {
            let res = items
                .iter()
                .map(should_drop)
                .collect::<Result<Vec<_>, _>>()?;
            if res.iter().all(|i| *i) {
                Ok(true)
            } else if res.iter().all(|i| !*i) {
                Ok(false)
            } else {
                Err("can't filter out only parts of this `use` statement. Some parts are not supposed to be shown to students on weblab.".to_string())
            }
        }
    }
}

struct DropUse;

impl Fold for DropUse {
    fn fold_item(&mut self, item: Item) -> Item {
        if let Item::Use(ItemUse { tree, .. }) = &item {
            match should_drop(tree) {
                Ok(true) => {
                    return Item::Verbatim(TokenStream2::new());
                }
                Ok(false) => { /* do nothing */ }
                _ => unreachable!("all errors should have been filtered out by FindAnnotated"),
            }
        }

        fold_item(self, item)
    }
}

impl Fold for FindAnnotated {
    fn fold_macro(&mut self, mut i: Macro) -> Macro {
        const TARGETS: &[&str] = &["template_only", "solution_only"];


        let ident = i.path.segments.last().expect("no segments in path").ident.to_string();

        let msg = format!("use braces in {}", ident);

        if !matches!(i.delimiter, MacroDelimiter::Brace(_)) &&
            TARGETS.contains(&ident.as_str()) {
            i.tokens = quote_spanned! {
                i.span() =>
                compile_error!(#msg);
                todo!()
            }
        }

        i
    }
    
    fn fold_item(&mut self, mut item: Item) -> Item {
        let attrs = match &mut item {
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

        if let Item::Macro(ItemMacro { mac, .. }) = &item {
            if let Some(ident) = mac.path.get_ident() {
                if ident == "template_only" {
                    if self.is_reference() {
                        return Item::Verbatim(TokenStream2::new());
                    } else {
                        item = parse_only_contents(&mac.tokens);
                    }
                } else if ident == "solution_only" {
                    if self.is_template() {
                        return Item::Verbatim(TokenStream2::new());
                    } else {
                        item = parse_only_contents(&mac.tokens);
                    }
                }
            }
        }

        if let Item::Use(ItemUse { tree, .. }) = &item {
            if let Err(e) = should_drop(tree) {
                return Item::Verbatim(quote_spanned! {
                    item.span() =>
                    compile_error!(#e);
                });
            }
        }

        let folded = fold_item(self, item);
        let without_use = DropUse.fold_item(folded.clone());

        if let Item::Mod(ref i) = without_use {
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
                                });
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
                                });
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
                                });
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
                                });
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

        folded
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


#[proc_macro_attribute]
pub fn test(attr: TokenStream, item: TokenStream) -> TokenStream {
    quote! {}.into()
}
