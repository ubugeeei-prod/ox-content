export const packageBuildConcurrencyEnvName = "OX_CONTENT_VP_PACKAGE_BUILD_CONCURRENCY";

export function packageBuildConcurrencyFlag(env: NodeJS.ProcessEnv = process.env): string {
  const value = env[packageBuildConcurrencyEnvName];
  if (!value) {
    return "";
  }

  const parsed = Number.parseInt(value, 10);
  if (!Number.isSafeInteger(parsed) || parsed < 1 || String(parsed) !== value) {
    throw new Error(`${packageBuildConcurrencyEnvName} must be a positive integer`);
  }

  return ` --concurrency-limit ${value}`;
}
