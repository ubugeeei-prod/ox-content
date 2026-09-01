import * as fs from "fs";
import * as path from "path";

export function verifyPublishWorkflow(options: {
  root: string;
  workflowRel: string;
  cargoPackages: string[];
  npmPackages: string[];
}): void {
  const workflowPath = path.join(options.root, options.workflowRel);
  const workflow = fs.readFileSync(workflowPath, "utf-8");
  const missingCargo = options.cargoPackages.filter((pkg) => !hasCargoPublishTarget(workflow, pkg));
  const npmDirs = options.npmPackages.filter(
    (pkg) =>
      pkg.startsWith("npm/") && !pkg.startsWith("npm/theme") && pkg !== "npm/vscode-ox-content",
  );
  const missingNpm = npmDirs.filter((dir) => !workflow.includes(`working-directory: ${dir}`));
  if (missingCargo.length || missingNpm.length) {
    throw new Error(
      `Missing publish targets in ${options.workflowRel}: cargo=${missingCargo.join(",") || "-"} npm=${missingNpm.join(",") || "-"}`,
    );
  }
  console.log(`  Verified crates.io publish targets: ${options.cargoPackages.join(", ")}`);
  console.log(`  Verified npm publish targets: ${npmDirs.join(", ")}`);
}

function hasCargoPublishTarget(workflow: string, pkg: string): boolean {
  const escapedPkg = pkg.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const packageRef = `(?:"${escapedPkg}"|'${escapedPkg}'|${escapedPkg})`;
  return (
    new RegExp(`\\bcargo\\s+publish\\b[^\\n]*(?:-p|--package)\\s+${packageRef}(?=\\s|$)`).test(
      workflow,
    ) || new RegExp(`\\bpublish_crate\\s+${packageRef}(?=\\s|$)`).test(workflow)
  );
}
