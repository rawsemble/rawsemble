use crate::lexer::{Token, TokenKind};

#[derive(Debug)]
pub struct JavascriptModule {
    pub imports: Vec<JavascriptImport>,
    pub exports: Vec<JavascriptExport>,
    pub raw_source: String
}

#[derive(Debug, PartialEq)]
pub struct JavascriptImport {
    default_import: Option<DefaultImport>,
    named_imports: Vec<NamedImport>,
    pub specifier: String,
    pub specifier_start: usize,
    pub specifier_end: usize,
}

impl JavascriptImport {
    fn new() -> Self {
        Self {
            named_imports: Vec::new(),
            default_import: None,
            specifier: String::from(""),
            specifier_start: 0,
            specifier_end: 0,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct JavascriptExport {
    default_export: Option<DefaultExport>,
    named_exports: Vec<NamedExport>,
    pub specifier: String,
    pub specifier_start: usize,
    pub specifier_end: usize,
}

impl JavascriptExport {
    fn new() -> Self {
        Self {
            named_exports: Vec::new(),
            default_export: None,
            specifier: String::from(""),
            specifier_start: 0,
            specifier_end: 0,
        }
    }
}

#[derive(Debug)]
struct PendingImport {
    import: JavascriptImport,
}

#[derive(Debug, PartialEq)]
struct DefaultImport {
    variable_name: String,
    binding_name: String,
}

#[derive(Debug, PartialEq)]
struct NamedImport<> {
    variable_name: String,
    binding_name: String,
}

#[derive(Debug, PartialEq)]
enum NextToken {
  Specifier,
  DefaultImport,
  DefaultExport,
  NamedImport,
  NamedExport,
  StatementEnd,
  Continue,
}

#[derive(Debug)]
struct PendingExport {
    export: JavascriptExport,
}

#[derive(Debug, PartialEq)]
struct DefaultExport {
    variable_name: String,
    binding_name: String,
}

#[derive(Debug, PartialEq)]
struct NamedExport<> {
    variable_name: String,
    binding_name: String,
}

pub(crate) struct Parser<'l, 'a> {
    tokens: &'l [Token<'a>],
    pending_import: Option<PendingImport>,
    pending_export: Option<PendingExport>,
    next_token: NextToken,
    cursor: usize
}

impl<'l, 'a> Parser<'l, 'a> {
    pub fn new(tokens: &'l [Token<'a>]) -> Self {
        Self {
            tokens,
            pending_import: None,
            pending_export: None,
            next_token: NextToken::DefaultImport,
            cursor: 0,
        }
    }

    pub fn parse_module(mut self, raw_source: String) -> JavascriptModule {
        let mut js_module = JavascriptModule {
            imports: Vec::new(),
            exports: Vec::new(),
            raw_source
        };

        for Token { kind, text } in self.tokens {
            match kind {
                TokenKind::Import => {
                    self.pending_import = Some(PendingImport {
                        import: JavascriptImport::new(),
                    });
                    self.next_token = NextToken::DefaultImport;
                },
                TokenKind::Export => {
                    self.pending_export = Some(PendingExport {
                        export: JavascriptExport::new(),
                    });
                    self.next_token = NextToken::DefaultExport;
                },
                TokenKind::From => {
                    if self.pending_import_or_export() {
                        self.next_token = NextToken::Specifier;
                    }
                },
                TokenKind::Ident => {
                    if self.pending_import.is_some() {
                        let mut pending_import = self.pending_import.as_mut().unwrap();

                        match self.next_token {
                            NextToken::NamedImport => {
                                pending_import.import.named_imports.push(NamedImport {
                                    variable_name: text.to_string(),
                                    binding_name: text.to_string()
                                });
                            },
                            NextToken::DefaultImport => {
                                pending_import.import.default_import = Some(DefaultImport {
                                    variable_name: text.to_string(),
                                    binding_name: text.to_string()
                                });
                            },
                            _ => {}
                        }
                    } else if self.pending_export.is_some() {
                        let mut pending_export = self.pending_export.as_mut().unwrap();

                        match self.next_token {
                            NextToken::NamedExport => {
                                pending_export.export.named_exports.push(NamedExport {
                                    variable_name: text.to_string(),
                                    binding_name: text.to_string()
                                });
                            },
                            NextToken::DefaultExport => {
                                pending_export.export.default_export = Some(DefaultExport {
                                    variable_name: text.to_string(),
                                    binding_name: text.to_string()
                                });
                            },
                            _ => {}
                        }
                    }
                },
                TokenKind::LBrace | TokenKind::Comma => {
                    if self.pending_import.is_some() {
                        self.next_token = NextToken::NamedImport;
                    } else if self.pending_export.is_some() {
                        self.next_token = NextToken::NamedExport;
                    }
                },
                TokenKind::RBrace => {
                    if self.pending_import_or_export() {
                        self.next_token = NextToken::Specifier;
                    }
                },
                TokenKind::Default => {},
                TokenKind::As => {},
                TokenKind::Star => {},
                TokenKind::Error => {},
                TokenKind::Specifier => {
                    if self.pending_import.is_some() {
                        let mut pending_import = self.pending_import.as_mut().unwrap();
                        pending_import.import.specifier = text[1..text.len()-1].to_string();
                        pending_import.import.specifier_start = self.cursor;
                        pending_import.import.specifier_end = self.cursor + text.len();
                        self.next_token = NextToken::StatementEnd;
                    } else if self.pending_export.is_some() {
                        let mut pending_export = self.pending_export.as_mut().unwrap();
                        pending_export.export.specifier = text[1..text.len()-1].to_string();
                        pending_export.export.specifier_start = self.cursor;
                        pending_export.export.specifier_end = self.cursor + text.len();
                        self.next_token = NextToken::StatementEnd;
                    }
                },
                TokenKind::Semicolon | TokenKind::Whitespace => {
                    if self.next_token == NextToken::StatementEnd {
                        if self.pending_import.is_some() {
                            js_module.imports.push(self.pending_import.take().unwrap().import);
                        } else if self.pending_export.is_some() {
                            js_module.exports.push(self.pending_export.take().unwrap().export);
                        }
                        // we are done with this import or export.  Search for more
                        self.next_token = NextToken::Continue;
                    }
                },
            }

            self.cursor += text.len();
        }

        js_module
    }

    fn pending_import_or_export(&self) -> bool {
        self.pending_import.is_some() || self.pending_export.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::JavascriptLexer;

    #[test]
    fn parses_single_import() {
        let source = "
import A from './a.js';
";
        let tokens: Vec<_> = JavascriptLexer::new(&source).collect();
        let parser = Parser::new(&tokens);
        let module = parser.parse_module(source.to_string());
        assert_eq!(module.imports.len(), 1);
        assert_eq!(module.exports.len(), 0);
        assert_eq!(module.imports[0].default_import, Some(
            DefaultImport { variable_name: String::from("A"), binding_name: String::from("A") })
        );
        assert_eq!(module.imports[0].named_imports, []);
        assert_eq!(module.imports[0].specifier, "./a.js");
        assert_eq!(module.imports[0].specifier_start, 15);
        assert_eq!(module.imports[0].specifier_end, 23);
        assert_eq!(module.raw_source, source.to_string());
    }

    #[test]
    fn parses_single_import_with_dbl_quotes() {
        let source = "
import A from \"./a.js\";
";
        let tokens: Vec<_> = JavascriptLexer::new(&source).collect();
        let parser = Parser::new(&tokens);
        let module = parser.parse_module(source.to_string());
        assert_eq!(module.imports.len(), 1);
        assert_eq!(module.exports.len(), 0);
        assert_eq!(module.imports[0].default_import, Some(
            DefaultImport { variable_name: String::from("A"), binding_name: String::from("A") })
        );
        assert_eq!(module.imports[0].named_imports, []);
        assert_eq!(module.imports[0].specifier, "./a.js");
        assert_eq!(module.imports[0].specifier_start, 15);
        assert_eq!(module.imports[0].specifier_end, 23);
        assert_eq!(module.raw_source, source.to_string());
    }


    #[test]
    fn parses_without_ending_semicolon() {
        let source = "
import A from './a.js'
";
        let tokens: Vec<_> = JavascriptLexer::new(&source).collect();
        let parser = Parser::new(&tokens);
        let module = parser.parse_module(source.to_string());
        assert_eq!(module.imports[0].default_import, Some(
            DefaultImport { variable_name: String::from("A"), binding_name: String::from("A") })
        );
        assert_eq!(module.imports[0].named_imports, []);
        assert_eq!(module.imports[0].specifier, "./a.js");
        assert_eq!(module.imports[0].specifier_start, 15);
        assert_eq!(module.imports[0].specifier_end, 23);
    }

    #[test]
    fn parses_multiple_imports() {
        let source = "
import A, { a, b } from './a.js';
import C, { d, e } from './c.js';
";
        let tokens: Vec<_> = JavascriptLexer::new(&source).collect();
        let parser = Parser::new(&tokens);
        let module = parser.parse_module(source.to_string());
        assert_eq!(module.imports.len(), 2);
        assert_eq!(module.imports[0].default_import, Some(
            DefaultImport { variable_name: String::from("A"), binding_name: String::from("A") })
        );
        assert_eq!(module.imports[0].named_imports.len(), 2);
        assert_eq!(module.imports[0].named_imports, [
            NamedImport { variable_name: String::from("a"), binding_name: String::from("a") },
            NamedImport { variable_name: String::from("b"), binding_name: String::from("b") }
        ]);
        assert_eq!(module.imports[0].specifier, "./a.js");
        assert_eq!(module.imports[1].named_imports.len(), 2);
        assert_eq!(module.imports[1].named_imports, [
            NamedImport { variable_name: String::from("d"), binding_name: String::from("d") },
            NamedImport { variable_name: String::from("e"), binding_name: String::from("e") }
        ]);
    }

    #[test]
    fn parses_named_import() {
        let source = "
import { a } from './a.js';
";
        let tokens: Vec<_> = JavascriptLexer::new(&source).collect();
        let parser = Parser::new(&tokens);
        let module = parser.parse_module(source.to_string());
        assert_eq!(module.imports.len(), 1);
        assert_eq!(module.imports[0].default_import, None);
        assert_eq!(module.imports[0].named_imports, [NamedImport { variable_name: String::from("a"), binding_name: String::from("a") }]);
        assert_eq!(module.imports[0].specifier, "./a.js");
    }

    #[test]
    fn parses_multiple_named_imports() {
        let source = "
import { a, b } from './a.js';
";
        let tokens: Vec<_> = JavascriptLexer::new(&source).collect();
        let parser = Parser::new(&tokens);
        let module = parser.parse_module(source.to_string());
        assert_eq!(module.imports.len(), 1);
        assert_eq!(module.imports[0].default_import, None);
        assert_eq!(module.imports[0].named_imports, [
            NamedImport { variable_name: String::from("a"), binding_name: String::from("a") },
            NamedImport { variable_name: String::from("b"), binding_name: String::from("b") }
        ]);
        assert_eq!(module.imports[0].specifier, "./a.js");
    }

    #[test]
    fn parses_combo_imports() {
        let source = "
import A, { a, b } from './a.js';
";
        let tokens: Vec<_> = JavascriptLexer::new(&source).collect();
        let parser = Parser::new(&tokens);
        let module = parser.parse_module(source.to_string());
        assert_eq!(module.imports.len(), 1);
        assert_eq!(module.imports[0].default_import, Some(
            DefaultImport { variable_name: String::from("A"), binding_name: String::from("A") })
        );
        assert_eq!(module.imports[0].named_imports, [
            NamedImport { variable_name: String::from("a"), binding_name: String::from("a") },
            NamedImport { variable_name: String::from("b"), binding_name: String::from("b") }
        ]);
        assert_eq!(module.imports[0].specifier, "./a.js");
    }

    #[test]
    fn parses_single_export() {
        let source = "
export A from './a.js';
";
        let tokens: Vec<_> = JavascriptLexer::new(&source).collect();
        let parser = Parser::new(&tokens);
        let module = parser.parse_module(source.to_string());
        assert_eq!(module.imports.len(), 0);
        assert_eq!(module.exports.len(), 1);
        assert_eq!(module.exports[0].default_export, Some(
            DefaultExport { variable_name: String::from("A"), binding_name: String::from("A") })
        );
        assert_eq!(module.exports[0].named_exports, []);
        assert_eq!(module.exports[0].specifier, "./a.js");
        assert_eq!(module.exports[0].specifier_start, 15);
        assert_eq!(module.exports[0].specifier_end, 23);
        assert_eq!(module.raw_source, source.to_string());
    }

    #[test]
    fn parses_named_exports() {
        let source = "
export { d, E } from './d.js';
";
        let tokens: Vec<_> = JavascriptLexer::new(&source).collect();
        let parser = Parser::new(&tokens);
        let module = parser.parse_module(source.to_string());
        assert_eq!(module.imports.len(), 0);
        assert_eq!(module.exports.len(), 1);
        assert_eq!(module.exports[0].default_export, None);
        assert_eq!(module.exports[0].named_exports, [
            NamedExport { variable_name: String::from("d"), binding_name: String::from("d") },
            NamedExport { variable_name: String::from("E"), binding_name: String::from("E") }
        ]);
        assert_eq!(module.exports[0].specifier, "./d.js");
        assert_eq!(module.exports[0].specifier_start, 22);
        assert_eq!(module.exports[0].specifier_end, 30);
        assert_eq!(module.raw_source, source.to_string());
    }

    #[test]
    fn parses_as_default_export() {
        let source = "
export { b as default } from './b.js';
";
        let tokens: Vec<_> = JavascriptLexer::new(&source).collect();
        let parser = Parser::new(&tokens);
        let module = parser.parse_module(source.to_string());
        assert_eq!(module.imports.len(), 0);
        assert_eq!(module.exports.len(), 1);
        assert_eq!(module.exports[0].default_export, None);
        assert_eq!(module.exports[0].named_exports, [
            NamedExport { variable_name: String::from("b"), binding_name: String::from("b") },
        ]);
        assert_eq!(module.exports[0].specifier, "./b.js");
        assert_eq!(module.exports[0].specifier_start, 30);
        assert_eq!(module.exports[0].specifier_end, 38);
        assert_eq!(module.raw_source, source.to_string());
    }

    #[test]
    fn parses_default_as_named_export() {
        let source = "
export { default as b } from './b.js';
";
        let tokens: Vec<_> = JavascriptLexer::new(&source).collect();
        let parser = Parser::new(&tokens);
        let module = parser.parse_module(source.to_string());
        assert_eq!(module.imports.len(), 0);
        assert_eq!(module.exports.len(), 1);
        assert_eq!(module.exports[0].default_export, None);
        assert_eq!(module.exports[0].named_exports, [
            NamedExport { variable_name: String::from("b"), binding_name: String::from("b") },
        ]);
        assert_eq!(module.exports[0].specifier, "./b.js");
        assert_eq!(module.exports[0].specifier_start, 30);
        assert_eq!(module.exports[0].specifier_end, 38);
        assert_eq!(module.raw_source, source.to_string());
    }

    #[test]
    fn parses_splat_export() {
        let source = "
export * from './b.js';
";
        let tokens: Vec<_> = JavascriptLexer::new(&source).collect();
        let parser = Parser::new(&tokens);
        let module = parser.parse_module(source.to_string());
        assert_eq!(module.imports.len(), 0);
        assert_eq!(module.exports.len(), 1);
        assert_eq!(module.exports[0].default_export, None);
        assert_eq!(module.exports[0].named_exports, []);
        assert_eq!(module.exports[0].specifier, "./b.js");
        assert_eq!(module.exports[0].specifier_start, 15);
        assert_eq!(module.exports[0].specifier_end, 23);
        assert_eq!(module.raw_source, source.to_string());
    }

    #[test]
    fn parses_multiple_exports() {
        let source = "
export { d, E } from './d.js';
export A from './a.js';
";
        let tokens: Vec<_> = JavascriptLexer::new(&source).collect();
        let parser = Parser::new(&tokens);
        let module = parser.parse_module(source.to_string());
        assert_eq!(module.imports.len(), 0);
        assert_eq!(module.exports.len(), 2);
        assert_eq!(module.exports[0].default_export, None);
        assert_eq!(module.exports[0].named_exports, [
            NamedExport { variable_name: String::from("d"), binding_name: String::from("d") },
            NamedExport { variable_name: String::from("E"), binding_name: String::from("E") }
        ]);
        assert_eq!(module.exports[0].specifier, "./d.js");
        assert_eq!(module.exports[0].specifier_start, 22);
        assert_eq!(module.exports[0].specifier_end, 30);
        assert_eq!(module.exports[1].default_export, Some(
            DefaultExport { variable_name: String::from("A"), binding_name: String::from("A") })
        );
        assert_eq!(module.exports[1].named_exports, []);
        assert_eq!(module.exports[1].specifier, "./a.js");
        assert_eq!(module.exports[1].specifier_start, 46);
        assert_eq!(module.exports[1].specifier_end, 54);
        assert_eq!(module.raw_source, source.to_string());
    }

    #[test]
    fn parses_multiple_exports_2() {
        let source = "
export { default as DModule } from './d.js';
export { A as default } from './a-default.js';
";
        let tokens: Vec<_> = JavascriptLexer::new(&source).collect();
        let parser = Parser::new(&tokens);
        let module = parser.parse_module(source.to_string());
        assert_eq!(module.imports.len(), 0);
        assert_eq!(module.exports.len(), 2);
        assert_eq!(module.exports[0].default_export, None);
        assert_eq!(module.exports[0].named_exports, [
            NamedExport { variable_name: String::from("DModule"), binding_name: String::from("DModule") },
        ]);
        assert_eq!(module.exports[0].specifier, "./d.js");
        assert_eq!(module.exports[0].specifier_start, 36);
        assert_eq!(module.exports[0].specifier_end, 44);
        assert_eq!(module.exports[1].default_export, None);
        assert_eq!(module.exports[1].named_exports, [
            NamedExport { variable_name: String::from("A"), binding_name: String::from("A") },
        ]);
        assert_eq!(module.exports[1].specifier, "./a-default.js");
        assert_eq!(module.exports[1].specifier_start, 75);
        assert_eq!(module.exports[1].specifier_end, 91);
        assert_eq!(module.raw_source, source.to_string());
    }

    #[test]
    fn does_not_parse_named_export() {
        let source = "
const e = 'e';

export { e };
";
        let tokens: Vec<_> = JavascriptLexer::new(&source).collect();
        let parser = Parser::new(&tokens);
        let module = parser.parse_module(source.to_string());
        assert_eq!(module.imports.len(), 0);
        assert_eq!(module.exports.len(), 0);
        assert_eq!(module.raw_source, source.to_string());
    }

    #[test]
    fn does_not_parse_default_export() {
        let source = "
class C {}
export default C;
";
        let tokens: Vec<_> = JavascriptLexer::new(&source).collect();
        let parser = Parser::new(&tokens);
        let module = parser.parse_module(source.to_string());
        assert_eq!(module.imports.len(), 0);
        assert_eq!(module.exports.len(), 0);
        assert_eq!(module.raw_source, source.to_string());
    }

    #[test]
    fn parses_imports_and_exports() {
        let source = "
import A from './a.js';
export E from './e.js';
";
        let tokens: Vec<_> = JavascriptLexer::new(&source).collect();
        let parser = Parser::new(&tokens);
        let module = parser.parse_module(source.to_string());
        assert_eq!(module.imports.len(), 1);
        assert_eq!(module.exports.len(), 1);
        assert_eq!(module.imports[0].default_import, Some(
            DefaultImport { variable_name: String::from("A"), binding_name: String::from("A") })
        );
        assert_eq!(module.imports[0].named_imports, []);
        assert_eq!(module.exports[0].default_export, Some(
            DefaultExport { variable_name: String::from("E"), binding_name: String::from("E") })
        );
        assert_eq!(module.exports[0].named_exports, []);
        assert_eq!(module.exports[0].specifier, "./e.js");
        assert_eq!(module.exports[0].specifier_start, 39);
        assert_eq!(module.exports[0].specifier_end, 47);
        assert_eq!(module.raw_source, source.to_string());
    }

    #[test]
    fn parses_file() {
        let source = "
import A from './a.js';
function() {};
let z = 'bar';
export E from './e.js';
";
        let tokens: Vec<_> = JavascriptLexer::new(&source).collect();
        let parser = Parser::new(&tokens);
        let module = parser.parse_module(source.to_string());

        assert_eq!(module.imports.len(), 1);
        assert_eq!(module.exports.len(), 1);
        assert_eq!(module.imports[0].default_import, Some(
            DefaultImport { variable_name: String::from("A"), binding_name: String::from("A") })
        );
        assert_eq!(module.imports[0].named_imports, []);
        assert_eq!(module.imports[0].specifier, "./a.js");
        assert_eq!(module.imports[0].specifier_start, 15);
        assert_eq!(module.imports[0].specifier_end, 23);

        assert_eq!(module.exports[0].default_export, Some(
            DefaultExport { variable_name: String::from("E"), binding_name: String::from("E") })
        );
        assert_eq!(module.exports[0].named_exports, []);
        assert_eq!(module.exports[0].specifier, "./e.js");
        assert_eq!(module.exports[0].specifier_start, 69);
        assert_eq!(module.exports[0].specifier_end, 77);
        assert_eq!(module.raw_source, source.to_string());
    }

    #[test]
    fn handles_comments() {
        let source = "
import A from './a.js'; // test-comment
/* test-comment */
import B from './b.js';
import C from './c.js'; /** test-comment */
";
        let tokens: Vec<_> = JavascriptLexer::new(&source).collect();
        let parser = Parser::new(&tokens);
        let module = parser.parse_module(source.to_string());
        assert_eq!(module.imports.len(), 3);
        assert_eq!(module.exports.len(), 0);
        assert_eq!(module.imports[0].default_import, Some(
            DefaultImport { variable_name: String::from("A"), binding_name: String::from("A") })
        );
        assert_eq!(module.imports[0].named_imports, []);
        assert_eq!(module.imports[0].specifier, "./a.js");
        assert_eq!(module.imports[0].specifier_start, 15);
        assert_eq!(module.imports[0].specifier_end, 23);
        assert_eq!(module.raw_source, source.to_string());

        assert_eq!(module.imports[1].default_import, Some(
            DefaultImport { variable_name: String::from("B"), binding_name: String::from("B") })
        );
        assert_eq!(module.imports[1].named_imports, []);
        assert_eq!(module.imports[1].specifier, "./b.js");
        assert_eq!(module.imports[1].specifier_start, 74);
        assert_eq!(module.imports[1].specifier_end, 82);
        assert_eq!(module.raw_source, source.to_string());

        assert_eq!(module.imports[2].default_import, Some(
            DefaultImport { variable_name: String::from("C"), binding_name: String::from("C") })
        );
        assert_eq!(module.imports[2].named_imports, []);
        assert_eq!(module.imports[2].specifier, "./c.js");
        assert_eq!(module.imports[2].specifier_start, 98);
        assert_eq!(module.imports[2].specifier_end, 106);
        assert_eq!(module.raw_source, source.to_string());
    }

}
