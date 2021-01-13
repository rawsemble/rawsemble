use std::convert::AsRef;
use strum_macros::AsRefStr;

#[derive(Debug)]
pub struct JavascriptModule {
  pub imports: Vec<JavascriptImport>,
  pub exports: Vec<JavascriptExport>,
  pub raw_source: String,
}

#[derive(Debug)]
enum ImportToken {
  Variables, // represents a block of possible NamedImports or DefaultImports
  From,
  Specifier,
  DefaultImport,
  NamedImport,
  NextNamedImport,
  StatementEnd,
}

#[derive(Debug)]
enum ExportToken {
  Variables,
  From,
  Specifier,
  NamedExport,
  NextNamedExport,
  StatementEnd,
}

#[derive(Debug)]
struct PendingJavascriptImport {
  import: JavascriptImport,
  expected_token: ImportToken,
  token_start: Option<usize>,
  str_char: Option<char>,
}

// TODO: Backtracking to ensure there is a specifier
#[derive(Debug)]
struct PendingJavascriptExport {
  export: JavascriptExport,
  expected_token: ExportToken,
  token_start: Option<usize>,
  str_char: Option<char>,
}

#[derive(Debug)]
pub struct JavascriptImport {
  pub default_name: Option<String>,
  pub default_import: Option<DefaultImport>,
  pub named_imports: Vec<NamedImport>,
  pub specifier: String,
  pub specifier_start: usize,
  pub specifier_end: usize,
}

#[derive(Debug)]
pub struct DefaultImport {
  variable_name: String,
  binding_name: String
}

#[derive(Debug)]
pub struct NamedImport {
  variable_name: String,
  binding_name: String
}

#[derive(Debug)]
pub struct JavascriptExport {
  pub default_name: Option<String>,
  pub named_exports: Vec<NamedExport>,
  pub specifier: String,
  pub specifier_start: usize,
  pub specifier_end: usize,
}

#[derive(Debug)]
pub struct NamedExport {
  variable_name: String,
  binding_name: String // TODO: what is the difference?
}

pub struct JavascriptLexer {
  source: String,
  current_index: usize,
  current_char: char,
  indices_to_skip: usize,
  handler_stack: Vec<Handler>,
  current_handler: Handler,
  pending_import: Option<PendingJavascriptImport>,
  pending_export: Option<PendingJavascriptExport>,
}

#[derive(Clone, Copy, AsRefStr, Debug)]
enum Handler {
  Normal,
  Import,
  Export,
}

#[derive(Clone, Copy)]
enum ImportTokens {
  Variables,
  From,
  Specifier
}

impl JavascriptLexer {
  pub fn new(source: String) -> JavascriptLexer {
    JavascriptLexer {
      source,
      current_index: 0,
      current_char: ' ',
      indices_to_skip: 0,
      handler_stack: Vec::new(),
      current_handler: Handler::Normal,
      pending_import: None,
      pending_export: None
    }
  }

  pub fn parse_module(&mut self) -> JavascriptModule {
    let mut js_module = JavascriptModule {
      imports: Vec::new(),
      exports: Vec::new(),
      raw_source: self.source.clone(),
    };

    self.source = self.source.to_string();
    self.current_char = ' ';
    self.current_index = 0;
    self.indices_to_skip = 0;
    self.handler_stack = Vec::new();
    self.handler_stack.push(Handler::Normal);

    let source = self.source.clone();

    for (i, c) in source.char_indices() {
      if self.indices_to_skip > 0 {
        self.indices_to_skip = self.indices_to_skip - 1;
        continue
      }

      self.current_char = c;
      self.current_index = i;

      self.current_handler = self.handler_stack.pop().expect(format!("JavascriptLexer died - no handler specified at index {}. Last handler was {}", i, self.current_handler.as_ref()).as_str());

      match self.current_handler {
        Handler::Normal => self.handle_normal(&mut js_module),
        Handler::Import => self.handle_import(&mut js_module),
        Handler::Export => self.handle_export(&mut js_module),
      }
    }

    js_module
  }

  fn keep_using_handler(&mut self) {
    self.handler_stack.push(self.current_handler);
  }

  fn queue_handler(&mut self, handler: Handler) {
    self.handler_stack.push(handler);
  }

  fn handle_normal(&mut self, _js_module: &mut JavascriptModule) {
    match self.current_char {
      'i' => {
        match self.source.get(self.current_index..self.current_index + 6).unwrap_or("") {
          "import" => {
            self.queue_handler(Handler::Import);
            self.indices_to_skip = 5;
            self.pending_import = Some(PendingJavascriptImport {
              expected_token: ImportToken::Variables,
              import: JavascriptImport {
                default_name: None,
                named_imports: Vec::new(),
                default_import: None,
                specifier: String::new(),
                specifier_start: 0,
                specifier_end: 0,
              },
              token_start: None,
              str_char: None,
            });
          },
          _ => {
            self.keep_using_handler();
          }
        }
      },
      'e' => {
        match self.source.get(self.current_index..self.current_index + 6).unwrap_or("") {
          "export" => {
            self.queue_handler(Handler::Export);
            self.indices_to_skip = 5;
            self.pending_export = Some(PendingJavascriptExport {
              expected_token: ExportToken::Variables,
              export: JavascriptExport {
                default_name: None,
                named_exports: Vec::new(),
                specifier: String::new(),
                specifier_start: 0,
                specifier_end: 0,
              },
              token_start: None,
              str_char: None,
            });
          },
          _ => {
            self.keep_using_handler();
          }
        }
      },
      _ => {
        self.keep_using_handler();
      }
    }
  }

  fn handle_import(&mut self, js_module: &mut JavascriptModule) {
    let mut pending_import = self.pending_import.as_mut().unwrap();

    match pending_import.expected_token {
      ImportToken::Variables => {
        match self.current_char {
          '{' => {
            pending_import.expected_token = ImportToken::NamedImport;
          },
          '\u{0041}'..='\u{2FA1D}' => {
            pending_import.expected_token = ImportToken::DefaultImport;
            pending_import.token_start = Some(self.current_index);
          }
          _ => {
              // is_whitespace
          }
        }

        self.keep_using_handler();
      },
      ImportToken::DefaultImport => {
        match self.current_char {
          '\u{0030}'..='\u{E01EF}' => {}, // still parsing the DefaultImport
          _ => {
            let identifier = String::from(self.source.get(pending_import.token_start.unwrap()..self.current_index).unwrap());
            let default_import = DefaultImport {
              variable_name: identifier.clone(),
              binding_name: identifier,
            };
            pending_import.import.default_import = Some(default_import);
            let next_token;
            match self.current_char {
              ',' => {
                // import A, { ... } from
                // TODO: None?
                pending_import.token_start = None;
                next_token = ImportToken::Variables;
              },
              _ => {
                // whitespace
                next_token = ImportToken::From;
                pending_import.token_start = None;
              }
            }
            pending_import.expected_token = next_token;
          }
        }
        self.keep_using_handler();
      },
      ImportToken::NamedImport => {
        if pending_import.token_start.is_none() {
          match self.current_char {
            c if c.is_whitespace() => {},
            '\u{0041}'..='\u{2FA1D}' => {
              pending_import.token_start = Some(self.current_index);
            },
            _ => {
              panic!(format!("Invalid character '{}' at index {} - expected identifier start", self.current_char, self.current_index));
            }
          }
        } else {
          match self.current_char {
            '\u{0030}'..='\u{E01EF}' => {
               // still parsing the NamedImport
            },
            _ => {
              let identifier = String::from(self.source.get(pending_import.token_start.unwrap()..self.current_index).unwrap());
              let named_import = NamedImport {
                variable_name: identifier.clone(),
                binding_name: identifier,
              };
              pending_import.import.named_imports.push(named_import);
              let next_token;
              match self.current_char {
                '}' => {
                  next_token = ImportToken::From;
                },
                ',' => {
                  next_token = ImportToken::NamedImport;
                },
                _ => {
                  // whitespace
                  next_token = ImportToken::NextNamedImport;
                }
              }
              pending_import.expected_token = next_token;
              pending_import.token_start = None;
            }
          }
        }
        self.keep_using_handler();
      },
      ImportToken::NextNamedImport => {
        match self.current_char {
          '}' => {
            pending_import.expected_token = ImportToken::From;
          },
          ',' => {
            pending_import.expected_token = ImportToken::NamedImport;
          },
          c if c.is_whitespace() => {},
          _ => {
            panic!(format!("Invalid character '{}' at index {} - expected ',' or '}}'", self.current_char, self.current_index));
          }
        }
        self.keep_using_handler();
      },
      ImportToken::From => {
        match self.current_char {
          c if c.is_whitespace() => {},
          'f' => {
            match self.source.get(self.current_index..self.current_index+4).unwrap_or("f") {
              "from" => {
                self.indices_to_skip = 3;
                pending_import.expected_token = ImportToken::Specifier;
              },
              _ => {
                panic!(format!("Invalid character '{}' at index {} - expected keyword 'from'", self.current_char, self.current_index));
              }
            }
          },
          _ => {
            panic!(format!("Invalid character '{}' at index {} - expected keyword 'from'", self.current_char, self.current_index));
          }
        }
        self.keep_using_handler();
      },
      ImportToken::Specifier => {
        if pending_import.token_start.is_none() {
          match self.current_char {
            c if c.is_whitespace() => {},
            '\'' | '"' => {
              pending_import.token_start = Some(self.current_index + 1);
              pending_import.str_char = Some(self.current_char);
              pending_import.expected_token = ImportToken::Specifier;
              pending_import.import.specifier_start = self.current_index + 1;
            },
            _ => {
              panic!(format!("Invalid character '{}' at index {} - expected string start ' or \"", self.current_char, self.current_index));
            }
          }
        } else {
          match self.current_char {
            c if c == pending_import.str_char.unwrap() => {
              pending_import.import.specifier = String::from(self.source.get(pending_import.token_start.unwrap()..self.current_index).unwrap());
              pending_import.expected_token = ImportToken::StatementEnd;
              pending_import.import.specifier_end = self.current_index - 1;
            },
            _ => {}
          }
        }
        self.keep_using_handler();
      },
      ImportToken::StatementEnd => {
        match self.current_char {
          c if c == ' ' => {},
          ';' | '\n' | '\r' => {
            js_module.imports.push(self.pending_import.take().unwrap().import);
            self.queue_handler(Handler::Normal);
          },
          _ => {
            panic!(format!("Invalid character '{}' at index {} - expected statement end", self.current_char, self.current_index));
          }
        }
      },
    }
  }

  fn handle_export(&mut self, js_module: &mut JavascriptModule) {
    let mut pending_export = self.pending_export.as_mut().unwrap();

    match pending_export.expected_token {
      ExportToken::Variables => {
        match self.current_char {
          '{' => {
            pending_export.expected_token = ExportToken::NamedExport;
          },
          '*' => {
            pending_export.expected_token = ExportToken::From;
          },
          _ => {}
        }

        self.keep_using_handler();
      },
      ExportToken::NamedExport => {
        if pending_export.token_start.is_none() {
          match self.current_char {
            c if c.is_whitespace() => {},
            '\u{0041}'..='\u{2FA1D}' => {
              pending_export.token_start = Some(self.current_index);
            },
            _ => {
              panic!(format!("Invalid character '{}' at index {} - expected identifier start", self.current_char, self.current_index));
            }
          }
        } else {
          match self.current_char {
            '\u{0030}'..='\u{E01EF}' => {},
            _ => {
              let identifier = String::from(self.source.get(pending_export.token_start.unwrap()..self.current_index).unwrap());
              if identifier == "default" {
                  // TODO: this can be improved.  "default as" w/ backtracking?
                  // reset since next identifier will alias default "export { default as ... }"
                  pending_export.token_start = None;
                  pending_export.expected_token = ExportToken::NextNamedExport;
              } else {
                  let named_export = NamedExport {
                    variable_name: identifier.clone(),
                    binding_name: identifier,
                  };
                  pending_export.export.named_exports.push(named_export);
                  let next_token;
                  match self.current_char {
                    '}' => {
                      next_token = ExportToken::From;
                    },
                    ',' => {
                      next_token = ExportToken::NamedExport;
                    },
                    _ => {
                      next_token = ExportToken::NextNamedExport;
                    }
                  }
                  pending_export.expected_token = next_token;
                  pending_export.token_start = None;
              }
            }
          }
        }
        self.keep_using_handler();
      },
      ExportToken::NextNamedExport => {
        match self.current_char {
          '}' => {
            pending_export.expected_token = ExportToken::From;
          },
          ',' => {
            pending_export.expected_token = ExportToken::NamedExport;
          },
          'a' => {
            // re-export: `export { default as b } from './b';`
            self.indices_to_skip = 1;
            pending_export.expected_token = ExportToken::NamedExport;
          },
          c if c.is_whitespace() => {
            self.keep_using_handler();
          },
          _ => {
            panic!(format!("Invalid character '{}' at index {} - expected ',' or keyword 'as' or '}}'", self.current_char, self.current_index));
          }
        }
        self.keep_using_handler();
      },
      ExportToken::From => {
        match self.current_char {
          c if c.is_whitespace() => {},
          'f' => {
            match self.source.get(self.current_index..self.current_index+4).unwrap_or("f") {
              "from" => {
                self.indices_to_skip = 3;
                pending_export.expected_token = ExportToken::Specifier;
              },
              _ => {
                panic!(format!("Invalid character '{}' at index {} - expected keyword 'from'", self.current_char, self.current_index));
              }
            }
          },
          ';' => {
            // export { b };
            // This ended up not being an export with a specifier
            self.pending_export = None;
            self.queue_handler(Handler::Normal);
            return;
          }
          _ => {
            panic!(format!("Invalid character '{}' at index {} - expected keyword 'from' or statement end ';'", self.current_char, self.current_index));
          }
        }
        self.keep_using_handler();
      },
      ExportToken::Specifier => {
        if pending_export.token_start.is_none() {
          match self.current_char {
            c if c.is_whitespace() => {},
            '\'' | '"' => {
              pending_export.token_start = Some(self.current_index + 1);
              pending_export.str_char = Some(self.current_char);
              pending_export.expected_token = ExportToken::Specifier;
              pending_export.export.specifier_start = self.current_index + 1;
            },
            _ => {
              panic!(format!("Invalid character '{}' at index {} - expected string start ' or \"", self.current_char, self.current_index));
            }
          }
        } else {
          match self.current_char {
            c if c == pending_export.str_char.unwrap() => {
              pending_export.export.specifier = String::from(self.source.get(pending_export.token_start.unwrap()..self.current_index).unwrap());
              pending_export.expected_token = ExportToken::StatementEnd;
              pending_export.export.specifier_end = self.current_index - 1;
            },
            _ => {}
          }
        }
        self.keep_using_handler();
      },
      ExportToken::StatementEnd => {
        match self.current_char {
          c if c == ' ' => {},
          ';' | '\n' | '\r' => {
            js_module.exports.push(self.pending_export.take().unwrap().export);
            self.queue_handler(Handler::Normal);
          },
          _ => {
            panic!(format!("Invalid character '{}' at index {} - expected statement end", self.current_char, self.current_index));
          }
        }
      },
    }
  }
}
