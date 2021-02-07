use logos::Logos;

#[derive(Debug, PartialEq)]
pub(crate) struct Token<'a> {
    pub kind: TokenKind,
    pub text: &'a str,
}

pub(crate) struct JavascriptLexer<'a> {
    inner: logos::Lexer<'a, TokenKind>,
}

impl<'a> JavascriptLexer<'a> {
    pub(crate) fn new(source: &'a str) -> Self {
        Self {
            inner: TokenKind::lexer(source)
        }
    }
}

impl<'a> Iterator for JavascriptLexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let kind = self.inner.next()?;
        let text = self.inner.slice();

        Some(Self::Item { kind, text })
    }
}

#[derive(Logos, Debug, Clone, PartialEq)]
pub(crate) enum TokenKind {
    #[regex("[ \n]+")]
    Whitespace,

    #[token("import")]
    Import,

    #[token("export")]
    Export,

    #[token("default")]
    Default,

    #[token("from")]
    From,

    #[token("as")]
    As,

    #[token(",")]
    Comma,

    #[token(";")]
    Semicolon,

    #[token("*")]
    Star,

    #[token("}")]
    RBrace,

    #[token("{")]
    LBrace,

    #[regex("['\"]./[a-zA-Z0-9_-]+.js['\"]")] //TODO improve
    Specifier,

    #[regex("[A-Za-z][A-Za-z0-9]*")]
    Ident,

    #[error]
    Error,
}
