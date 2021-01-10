import { insertModule, createModuleUrl, resolveImportSpecifier } from "/bloom.js";
insertModule("test/fixtures/src/b.js",createModuleUrl(`const b = 're-export';

export { b };
`));
insertModule("test/fixtures/src/d.js",createModuleUrl(`const d = 're-export';

export default d;
`));
insertModule("test/fixtures/src/a.js",createModuleUrl(`export const a = 'a value';
export const other = 'other value';

export { b } from './b.js';
export { default as DModule } from './d.js';
`));
insertModule("test/fixtures/src/c-default.js",createModuleUrl(`const c = 'default export';

export default c;
`));
insertModule("test/fixtures/src/main.js",createModuleUrl(`import { a, b, DModule, other } from '${resolveImportSpecifier("test/fixtures/src/a.js")}';
import c from '${resolveImportSpecifier("test/fixtures/src/c-default.js")}';

console.log('hi there', a, other, b, c, DModule);
`));
import(resolveImportSpecifier("test/fixtures/src/main.js"));