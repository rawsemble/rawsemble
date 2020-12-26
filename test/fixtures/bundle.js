import { insertModule, createModuleUrl, getModuleUrl } from "/bloom.js";
insertModule("main.js",createModuleUrl(`import{ a } from '${getModuleUrl("./a.js")}';

console.log('hi there', a);`));
import(getModuleUrl("main.js"));