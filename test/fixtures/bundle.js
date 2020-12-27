import { insertModule, createModuleUrl, getModuleUrl } from "/bloom.js";
insertModule("main.js",createModuleUrl(`import { a, other } from '${getModuleUrl("./a.js")}';

console.log('hi there', a, other);`));
import(getModuleUrl("main.js"));