use crate::Span2;
use proc_macro::{Span, TokenStream};
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::{Attribute, AttributeArgs, Lit, Meta, MetaNameValue, NestedMeta, Token};

pub enum ToAttrError {
    Message(String),
    Spanned(Span2, String),
}

impl ToAttrError {
    fn into_token_stream(self) -> TokenStream {
        match self {
            ToAttrError::Spanned(s, m) => syn::Error::new(s, m).to_compile_error().into(),
            ToAttrError::Message(s) => syn::Error::new(Span::call_site().into(), s)
                .to_compile_error()
                .into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Attr {
    ProgrammingAssignment,
    Solution,
    SolutionTemplate,
    Test,
    TestTemplate,
    Library,

    Main,

    Title(String),
    Doc(String),
}

impl TryFrom<NestedMeta> for Attr {
    type Error = ToAttrError;

    fn try_from(value: NestedMeta) -> Result<Self, Self::Error> {
        match value {
            NestedMeta::Meta(m) => return m.try_into(),
            NestedMeta::Lit(_) => {}
        }

        Err(ToAttrError::Message(
            "expected one of 'programming_assignment', 'solution', 'solution_template', 'test', \
            'test_template', 'library', 'title=\"...\"' or 'description=\"...\"'"
                .to_string(),
        ))
    }
}
impl TryFrom<Meta> for Attr {
    type Error = ToAttrError;

    fn try_from(value: Meta) -> Result<Self, Self::Error> {
        match value {
            Meta::Path(path) => {
                if let Some(i) = path.get_ident() {
                    match i.to_string().as_str() {
                        "programming_assignment" => return Ok(Self::ProgrammingAssignment),
                        "main" => return Ok(Self::Main),
                        "solution" => return Ok(Self::Solution),
                        "solution_template" => return Ok(Self::SolutionTemplate),
                        "test" => return Ok(Self::Test),
                        "test_template" => return Ok(Self::TestTemplate),
                        "library" => return Ok(Self::Library),
                        _ => {}
                    }
                }
            }
            Meta::List(_) => {}
            Meta::NameValue(MetaNameValue {
                path,
                eq_token: _,
                lit,
            }) => {
                if let Some(i) = path.get_ident() {
                    match i.to_string().as_str() {
                        "title" => {
                            if let Lit::Str(s) = lit {
                                return Ok(Self::Title(s.value()));
                            } else {
                                return Err(ToAttrError::Spanned(
                                    lit.span(),
                                    "expected string".to_string(),
                                ));
                            }
                        }
                        "description" | "doc" => {
                            if let Lit::Str(s) = lit {
                                return Ok(Self::Doc(s.value()));
                            } else {
                                return Err(ToAttrError::Spanned(
                                    lit.span(),
                                    "expected string".to_string(),
                                ));
                            }
                        }
                        _ => {
                            return Err(ToAttrError::Spanned(
                                i.span(),
                                "expected 'title' or 'description'".to_string(),
                            ))
                        }
                    }
                }
            }
        }

        Err(ToAttrError::Message(
            "expected one of 'programming_assignment', 'solution', 'solution_template', 'test', \
            'test_template', 'library', 'main', 'title=\"...\"' or 'description=\"...\"'"
                .to_string(),
        ))
    }
}

pub enum ParseAttrStatus {
    NotParsed(Attribute),
    Doc(Attr, Attribute),
    Attr(Vec<Attr>),
}

pub fn parse_attr(attr: Attribute) -> Result<ParseAttrStatus, TokenStream> {
    struct Args {
        metas: Vec<NestedMeta>,
    }

    impl Parse for Args {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            let parsed =
                syn::punctuated::Punctuated::<NestedMeta, Token![,]>::parse_separated_nonempty(
                    input,
                )?;
            Ok(Self {
                metas: parsed.into_iter().collect(),
            })
        }
    }

    match attr.path.get_ident() {
        Some(i) if *i == "doc" => {
            let meta = attr
                .parse_meta()
                .map_err(|e| e.to_compile_error().to_token_stream())?;

            let parsed_attr: Attr = meta
                .try_into()
                .map_err(|e: ToAttrError| e.into_token_stream())?;

            Ok(ParseAttrStatus::Doc(parsed_attr, attr.clone()))
        }
        Some(i) if *i == "weblab" => {
            let args = attr
                .parse_args::<Args>()
                .map_err(|e| e.to_compile_error().to_token_stream())?;

            let attrs = args
                .metas
                .into_iter()
                .map(Attr::try_from)
                .collect::<Result<_, _>>()
                .map_err(|e| e.into_token_stream())?;

            Ok(ParseAttrStatus::Attr(attrs))
        }
        _ => Ok(ParseAttrStatus::NotParsed(attr)),
    }
}

pub fn parse_attr_stream(attr: TokenStream) -> Result<Vec<Attr>, TokenStream> {
    syn::parse_macro_input::parse::<AttributeArgs>(attr)
        .map_err(|e| -> TokenStream { e.to_compile_error().into() })?
        .into_iter()
        .map(Attr::try_from)
        .collect::<Result<_, _>>()
        .map_err(|e| e.into_token_stream())
}
