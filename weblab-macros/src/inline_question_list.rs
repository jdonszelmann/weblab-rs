use proc_macro2::TokenStream;
use quote::{quote, TokenStreamExt, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{Macro, Ident, LitStr, Token};
use crate::open::DocString;

pub struct MacroWrapper(pub Macro);
impl Parse for MacroWrapper {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self(input.parse()?))
    }
}
impl ToTokens for MacroWrapper {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Macro{ path, bang_token, delimiter: _, tokens : m_tokens} = &self.0;

        tokens.append_all(quote! {#path #bang_token (#m_tokens)});
    }
}

pub struct InlineQuestionList {
    pub title: String,
    pub question_text: DocString,
    pub questions: Vec<MacroWrapper>,
}

impl Parse for InlineQuestionList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut res = Self {
            title: String::new(),
            question_text: Default::default(),
            questions: Vec::new(),
        };

        while !input.is_empty() {
            // we found a macro!
            if input.peek2(Token!(!)) {
                let m = input.parse()?;
                res.questions.push(m);
                continue
            }

            let field: Ident = input.parse()?;

            match field.to_string().as_str() {
                "title" => {
                    let _colon: Token!(:) = input.parse()?;
                    res.title = input.parse::<LitStr>()?.value()
                },
                "question" => {
                    let _colon: Token!(:) = input.parse()?;
                    res.question_text = input.parse()?
                },
                n => return Err(syn::Error::new(field.span(), format!("unexpected field name {}", n)))
            }

            if input.peek(Token!(,)) {
                let _comma = input.parse::<Token!(,)>()?;
            }
        }

        Ok(res)
    }
}
