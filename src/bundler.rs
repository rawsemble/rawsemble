use super::lexer;

pub struct JavascriptBundle {
    pub content: String,
}

pub fn bundle(module: lexer::JavascriptModule) -> JavascriptBundle {
  let mut content = String::new();
  println!("Imports!");
  let mut last_index: usize = 0;

  content.push_str("import { insertModule, createModuleUrl, getModuleUrl } from \"/bloom.js\";\n");
  content.push_str("insertModule(\"main.js\",createModuleUrl(`");
  for import in module.imports.iter() {
    content.push_str(module.raw_source.get(last_index..import.specifier_start).unwrap());
    content.push_str("${getModuleUrl(\"");
    content.push_str(import.specifier.as_str());
    content.push_str("\")}");
    last_index = import.specifier_end + 1;
  }

  if last_index < module.raw_source.len() {
    content.push_str(module.raw_source.get(last_index..module.raw_source.len()).unwrap());
  }
  content.push_str("`));\n");
  content.push_str("import(getModuleUrl(\"main.js\"));");

  println!("content:");
  println!("{}", content);

  JavascriptBundle {
    content
  }
}