import { insertModule, createModuleUrl, resolveImportSpecifier } from "/bloom.js";
insertModule("test/fixtures/src/a.js",createModuleUrl(`export const a = 'a value';
export const other = 'other value';`));
insertModule("test/fixtures/src/main.js",createModuleUrl(`import { a, other } from '${resolveImportSpecifier("test/fixtures/src/a.js")}';

console.log('hi there', a, other);`));
import(resolveImportSpecifier("test/fixtures/src/main.js"));