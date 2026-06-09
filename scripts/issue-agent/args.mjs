export const usage = `Usage:
  node scripts/issue-agent.mjs run --issue <number> [--repo owner/name] [--base main]
  node scripts/issue-agent.mjs watch [--repo owner/name] [--base main] [--interval 60] [--backfill]
  node scripts/issue-agent.mjs list [--repo owner/name]

Options:
  --backfill        Process existing open issues on the first watch run.
  --base <branch>   Base branch for generated pull requests.
  --interval <sec>  Polling interval for watch mode. Default: 60.
  --issue <number>  Issue number for run mode.
  --no-watch-ci     Open the PR without waiting for checks.
  --repo <repo>     GitHub repository in owner/name form.
  --state-file <p>  Local state file. Default: .cache/issue-agent/state.json.
`;

export function parseArgs(rawArgs) {
  const parsed = {};

  for (let index = 0; index < rawArgs.length; index += 1) {
    const arg = rawArgs[index];
    if (!arg.startsWith("--")) {
      continue;
    }

    const [rawKey, rawValue] = arg.slice(2).split("=", 2);
    const key = rawKey.replace(/-([a-z])/g, (_, char) => char.toUpperCase());

    if (rawValue !== undefined) {
      parsed[key] = rawValue;
    } else if (rawArgs[index + 1] && !rawArgs[index + 1].startsWith("--")) {
      parsed[key] = rawArgs[index + 1];
      index += 1;
    } else {
      parsed[key] = true;
    }
  }

  return parsed;
}
