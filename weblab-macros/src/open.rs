use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{Attribute, Ident, Lit, LitStr, Meta, MetaNameValue, Token};

#[derive(Default)]
pub struct DocString {
    pub text: String,
}

impl ToTokens for DocString {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.text.to_tokens(tokens);
    }
}

impl Parse for DocString {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = Attribute::parse_outer(input)?;
        let mut text = String::new();

        for i in attrs {
            let meta = i.parse_meta()?;
            match meta {
                Meta::NameValue(MetaNameValue { path, lit, .. }) if path.is_ident("doc") => {
                    match lit {
                        Lit::Str(val) => {
                            if !text.is_empty() {
                                text.push('\n');
                            }

                            text.push_str(val.value().as_str().trim_start())
                        }
                        _ => {
                            return Err(syn::Error::new(lit.span(), "expected string literal here"))
                        }
                    }
                }
                _ => return Err(syn::Error::new(meta.span(), "expected doc=\"...\" here")),
            }
        }

        Ok(Self { text })
    }
}

pub struct OpenQuestion {
    pub title: String,
    pub question_text: DocString,
    pub answer: DocString,
}

impl Parse for OpenQuestion {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut res = Self {
            title: String::new(),
            question_text: Default::default(),
            answer: Default::default(),
        };

        while !input.is_empty() {
            let field: Ident = input.parse()?;
            let _colon: Token!(:) = input.parse()?;
            match field.to_string().as_str() {
                "title" => res.title = input.parse::<LitStr>()?.value(),
                "question" => res.question_text = input.parse()?,
                "answer" => res.answer = input.parse()?,
                n => {
                    return Err(syn::Error::new(
                        field.span(),
                        format!("unexpected field name {}", n),
                    ))
                }
            }

            if input.peek(Token!(,)) {
                let _comma = input.parse::<Token!(,)>()?;
            }
        }

        Ok(res)
    }
}
