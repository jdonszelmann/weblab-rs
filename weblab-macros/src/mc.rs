use crate::open::DocString;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitInt, LitStr, Token};

pub struct McAnswer {
    pub text: String,
    pub correct: bool,
}

pub struct McQuestion {
    pub title: String,
    pub question_text: DocString,
    pub options: Vec<McAnswer>,
    pub num_answers_expected: usize,
    pub randomize: bool,
    pub explanation: DocString,
}

impl Parse for McQuestion {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut res = Self {
            title: String::new(),
            question_text: Default::default(),
            options: vec![],
            num_answers_expected: 1,
            randomize: false,
            explanation: Default::default(),
        };

        while !input.is_empty() {
            let field: Ident = input.parse()?;

            match field.to_string().as_str() {
                "option" => {
                    let text = input.parse::<LitStr>()?.value();
                    let mut correct = false;
                    if input.peek(Ident) {
                        let correct_ident = input.parse::<Ident>()?;
                        if correct_ident != "correct" {
                            return Err(syn::Error::new(
                                correct_ident.span(),
                                "expected either the ident `correct` or nothing here",
                            ));
                        }
                        correct = true;
                    }

                    let _comma = input.parse::<Token!(,)>()?;

                    res.options.push(McAnswer { text, correct })
                }
                "explanation" => {
                    let _colon: Token!(:) = input.parse()?;
                    res.explanation = input.parse()?
                }
                "randomize" => {
                    res.randomize = true;
                }
                "multiple" => {
                    res.num_answers_expected = 0;
                }
                "title" => {
                    let _colon: Token!(:) = input.parse()?;
                    res.title = input.parse::<LitStr>()?.value()
                }
                "expect" => {
                    res.num_answers_expected = input.parse::<LitInt>()?.base10_parse()?;

                    let answers = input.parse::<Ident>()?;
                    if answers != "answers" {
                        return Err(syn::Error::new(
                            field.span(),
                            "expected the word answers here",
                        ));
                    }
                }
                "question" => {
                    let _colon: Token!(:) = input.parse()?;
                    res.question_text = input.parse()?
                }
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
