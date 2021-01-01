let registry = {};
export function insertModule(specifier, url) {
    registry[specifier] = url;
}
export function createModuleUrl(strings, ...imports) {
    const source = [];
    for (let i = 0; i < strings.length; i++) {
        source.push(strings[i]);
        if (i + 1 < imports.length) {
            source.push(resolveImportSpecifier(imports[i]));
        }
    }
    return sourceToModuleUrl(source);
}
export function resolveImportSpecifier(specifier) {
    if (registry[specifier]) {
        return registry[specifier];
    }
    else {
        throw Error(`No module has been created with specifier '${specifier}'`);
    }
}
export function insertProxyModule(specifier, exportNames) {
    const source = exportNames.map((exportName) => exportName === "default"
        ? `let defaultVal; export { defaultVal as default }; export function setdefault(val) { defaultVal = val };`
        : `export let ${exportName} = null; export function set${exportName}(val) {${exportName} = val;}`);
    insertModule(specifier, sourceToModuleUrl(source));
}
export function resetRegistry() {
    for (let specifier in registry) {
        const url = registry[specifier];
        if (url.startsWith("blob:")) {
            URL.revokeObjectURL(url);
        }
    }
    registry = {};
}
function sourceToModuleUrl(source) {
    return URL.createObjectURL(new Blob(source, { type: "application/javascript" }));
}
//# sourceMappingURL=bloom.js.map