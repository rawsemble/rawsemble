use super::lexer;
use std::collections::HashMap;
use std::rc::Rc;
use relative_path::{RelativePath, RelativePathBuf};
use std::env::current_dir;

pub struct JavascriptBundle {
    pub content: String,
}

pub fn bundle(entry_module: String, module_map: HashMap<String, lexer::JavascriptModule>) -> JavascriptBundle {
  let mut content = String::new();
  content.push_str("import { insertModule, createModuleUrl, resolveImportSpecifier } from \"/bloom.js\";\n");

  traverse_module(&entry_module, &mut content, Rc::new(module_map));

  content.push_str("import(resolveImportSpecifier(\"");
  content.push_str(entry_module.as_str());
  content.push_str("\"));");

  println!("{}", content);

  JavascriptBundle {
    content
  }
}

fn traverse_module(file_path: &String, content: &mut String, module_map: Rc<HashMap<String, lexer::JavascriptModule>>) {
  let full_path = RelativePath::new(file_path).to_path(current_dir().unwrap().as_path());
  let module = module_map.get(full_path.to_str().unwrap()).expect(format!("File not found in module_map {}", full_path.to_str().unwrap()).as_str());

  let mut parent_path_buf = RelativePathBuf::from(file_path.as_str());
  parent_path_buf.pop();
  for import in module.imports.iter() {
    let mod_path = parent_path_buf.join_normalized(RelativePath::new(&import.specifier));
    traverse_module(&mod_path.to_string(), content, Rc::clone(&module_map));
  }

  content.push_str("insertModule(\"");
  content.push_str(file_path.as_str());
  content.push_str("\",createModuleUrl(`");

  let mut last_index: usize = 0;
  for import in module.imports.iter() {
    let mod_path = parent_path_buf.join_normalized(RelativePath::new(&import.specifier));
    content.push_str(module.raw_source.get(last_index..import.specifier_start).unwrap());
    content.push_str("${resolveImportSpecifier(\"");
    content.push_str(&mod_path.to_string().as_str());
    content.push_str("\")}");
    last_index = import.specifier_end + 1;
  }

  if last_index < module.raw_source.len() {
    content.push_str(module.raw_source.get(last_index..module.raw_source.len()).unwrap());
  }
  content.push_str("`));\n");
}