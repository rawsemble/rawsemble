use std::fs;
pub mod bundler;
pub mod lexer;
pub mod parser;
use relative_path::{RelativePath, RelativePathBuf};
use std::collections::HashMap;
use std::env::current_dir;

fn main() {
    let mut module_map: HashMap<String, parser::JavascriptModule> = HashMap::new();
    let entry_file = String::from("test/fixtures/src/main.js");
    module_map = traverse_file(entry_file.clone(), module_map);

    let bundle: bundler::JavascriptBundle = bundler::bundle(entry_file, module_map);

    fs::write("test/fixtures/bundle.js", bundle.content).expect("Unable to write bundle.js");
    println!("bundle.js written");
}

fn traverse_file(
    file_path: String,
    mut module_map: HashMap<String, parser::JavascriptModule>,
) -> HashMap<String, parser::JavascriptModule> {
    let source = fs::read_to_string(file_path.clone())
        .expect(format!("Unable to read {}", file_path.clone()).as_str());

    let tokens: Vec<_> = lexer::JavascriptLexer::new(&source).collect();
    let parser = parser::Parser::new(&tokens);
    let module = parser.parse_module(source.clone());

    for import in module.imports.iter() {
        let mut parent_path_buf = RelativePathBuf::from(file_path.as_str());
        // remove filename + extension
        parent_path_buf.pop();
        let mod_path = parent_path_buf.join_normalized(RelativePath::new(&import.specifier));
        module_map = traverse_file(mod_path.to_string(), module_map);
    }

    for export in module.exports.iter() {
        let mut parent_path_buf = RelativePathBuf::from(file_path.as_str());
        parent_path_buf.pop();
        let mod_path = parent_path_buf.join_normalized(RelativePath::new(&export.specifier));
        module_map = traverse_file(mod_path.to_string(), module_map);
    }

    let full_path = RelativePath::new(file_path.as_str())
        .to_path(current_dir().unwrap().as_path())
        .to_str()
        .unwrap()
        .to_string();

    module_map.insert(full_path, module);

    module_map
}
