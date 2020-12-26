use std::fs;
pub mod lexer;
pub mod bundler;

fn main() {
    let entry_source = fs::read_to_string("test/fixtures/src/main.js").expect("Unable to read main.js");
    let entry_module: lexer::JavascriptModule = lexer::JavascriptLexer::new(entry_source).tokenize();
    println!("entry_module {:?}", entry_module);

    let bundle: bundler::JavascriptBundle = bundler::bundle(entry_module);

    fs::write("test/fixtures/bundle.js", bundle.content).expect("Unable to write bundle.js");
    println!("bundle.js written");
}
