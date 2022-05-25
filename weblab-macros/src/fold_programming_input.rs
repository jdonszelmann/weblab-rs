use syn::fold::{Fold, fold_item};
use syn::{Item, ItemConst, ItemEnum, ItemExternCrate, ItemFn, ItemForeignMod, ItemImpl, ItemMacro, ItemMacro2, ItemMod, ItemStatic, ItemStruct, ItemTrait, ItemTraitAlias, ItemType, ItemUnion, ItemUse, Macro, MacroDelimiter, UseGroup, UseName, UsePath, UseRename, UseTree};
use syn::spanned::Spanned;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote_spanned, ToTokens};
use crate::{ALLOWED_CRATES, Attr, parse_attr, ParseAttrStatus, Solution, SolutionTemplate};

pub enum FindAnnotated {
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

        let ident = i
            .path
            .segments
            .last()
            .expect("no segments in path")
            .ident
            .to_string();

        let msg = format!("use braces in {}", ident);

        if !matches!(i.delimiter, MacroDelimiter::Brace(_)) && TARGETS.contains(&ident.as_str()) {
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

pub enum Status {
    Certain(ItemMod),
    Maybe(ItemMod),
    Unkown,
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


