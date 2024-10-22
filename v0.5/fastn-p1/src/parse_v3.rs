pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

pub struct Lexer<'input> {
    token_stream: logos::SpannedIter<'input, fastn_p1::Sourced<fastn_p1::Token>>,
}

impl<'input> Lexer<'input> {
    #[allow(dead_code)]
    pub fn new(input: &'input str) -> Self {
        use logos::Logos;
        // the Token::lexer() method is provided by the Logos trait
        Self {
            token_stream: fastn_p1::Token::lexer(input).spanned(),
        }
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned<fastn_p1::Token, usize, fastn_p1::LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.token_stream.next().map(|(token, span)| {
            Ok((
                span.start,
                fastn_p1::Sourced {
                    from: span.start,
                    to: span.end,
                    value: token.unwrap(),
                },
                span.end,
            ))
        })
    }
}
