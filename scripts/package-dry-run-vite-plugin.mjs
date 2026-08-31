const publicValues = ["resolveSelfHostedAssetManifest", "writeSelfHostedAssets"];
const publicTypes = [
  "OxContentAssetManifest",
  "OxContentAssetPreload",
  "SelfHostedAssetOptions",
  "WriteSelfHostedAssetsInput",
  "WriteSelfHostedAssetsResult",
];
const virtualModules = ["virtual:ox-content/assets.css", "virtual:ox-content/asset-manifest"];

export function checkVitePluginDeclarations({ pkg, tarball, failures, readPackedFile }) {
  for (const extension of ["mts", "cts"]) {
    const declaration = readPackedFile(tarball, `dist/index.d.${extension}`);

    for (const name of [...publicValues, ...publicTypes]) {
      if (!new RegExp(`\\b${name}\\b`).test(declaration)) {
        failures.push(`${pkg.name} index.d.${extension} is missing ${name}`);
      }
    }

    for (const moduleId of virtualModules) {
      if (!declaration.includes(`declare module "${moduleId}"`)) {
        failures.push(`${pkg.name} index.d.${extension} is missing ${moduleId}`);
      }
    }
  }
}
