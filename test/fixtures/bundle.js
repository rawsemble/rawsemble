import { insertModule, createModuleUrl, resolveImportSpecifier } from "/bloom.js";
insertModule("test/fixtures/src/b.js",createModuleUrl(`const b = 're-export';

export { b };
`));
insertModule("test/fixtures/src/e.js",createModuleUrl(`const e = 're-export e-aggregate';

export { e };
`));
insertModule("test/fixtures/src/d.js",createModuleUrl(`const d = 're-export default';

export default d;
`));
insertModule("test/fixtures/src/a-default.js",createModuleUrl(`const A = () => {};

export { A };
`));
insertModule("test/fixtures/src/a.js",createModuleUrl(`export const a = 'a value';

export { b } from '${resolveImportSpecifier("test/fixtures/src/b.js")}';
export * from '${resolveImportSpecifier("test/fixtures/src/e.js")}';
export { default as DModule } from '${resolveImportSpecifier("test/fixtures/src/d.js")}';
export { A as default } from '${resolveImportSpecifier("test/fixtures/src/a-default.js")}';
`));
insertModule("test/fixtures/src/c-default.js",createModuleUrl(`const c = 'default export';

export default c;
`));
insertModule("test/fixtures/src/main.js",createModuleUrl(`import A, { a, b, DModule, e } from '${resolveImportSpecifier("test/fixtures/src/a.js")}'; // test-comment
import c from '${resolveImportSpecifier("test/fixtures/src/c-default.js")}';

/**
 * Make sure this shows up in the console
 */
console.log('hi there', A, a, b, c, DModule, e);
`));
import(resolveImportSpecifier("test/fixtures/src/main.js"));