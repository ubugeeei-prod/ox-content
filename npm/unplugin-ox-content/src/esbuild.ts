/**
 * esbuild plugin export for @ox-content/unplugin
 */

import oxContentUnplugin from "./index";

export default oxContentUnplugin.esbuild;
export { oxContentUnplugin as unplugin };
export * from "./index";
