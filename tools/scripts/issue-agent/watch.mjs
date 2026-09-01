import { listOpenIssues } from "./github.mjs";
import { loadState, markSeen, saveState, sleep } from "./utils.mjs";
import { processIssue } from "./process.mjs";

export async function watchIssues(context) {
  const intervalSeconds = Number(context.args.interval ?? 60);
  if (!Number.isFinite(intervalSeconds) || intervalSeconds < 5) {
    throw new Error("--interval must be a number >= 5");
  }

  const state = loadState(context.stateFile);

  if (!state.initialized && !context.args.backfill) {
    const openIssues = listOpenIssues(context.repo);
    for (const issue of openIssues) {
      markSeen(state, issue.number);
    }
    state.initialized = true;
    saveState(context.stateFile, state);
    process.stdout.write(
      `Seeded ${openIssues.length} existing open issue(s). Watching for new issues.\n`,
    );
  }

  state.initialized = true;
  saveState(context.stateFile, state);

  while (true) {
    const openIssues = listOpenIssues(context.repo);
    const pending = openIssues.filter((issue) => !state.seenIssues.includes(issue.number));

    for (const issue of pending) {
      markSeen(state, issue.number);
      saveState(context.stateFile, state);

      try {
        const result = await processIssue({ ...context, issue });
        state.processedIssues[String(issue.number)] = {
          at: new Date().toISOString(),
          branch: result.branch,
          prUrl: result.prUrl ?? null,
          status: result.status,
        };
      } catch (error) {
        state.processedIssues[String(issue.number)] = {
          at: new Date().toISOString(),
          error: error instanceof Error ? error.message : String(error),
          status: "failed",
        };
        process.stderr.write(
          `Issue #${issue.number} failed: ${state.processedIssues[String(issue.number)].error}\n`,
        );
      }

      saveState(context.stateFile, state);
    }

    await sleep(intervalSeconds * 1000);
  }
}
