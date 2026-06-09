export function buildPrTitle(issue) {
  const title = issue.title.replace(/\s+/g, " ").trim();
  const conventionalPattern =
    /^(build|chore|ci|docs|feat|fix|perf|refactor|revert|style|test)(\([^)]+\))?!?:\s+\S/i;
  const bugTitle = title.match(/^bug:\s*(.+)$/i);

  let prTitle = title;
  if (bugTitle) {
    prTitle = `fix: ${bugTitle[1].trim()}`;
  } else if (!conventionalPattern.test(title)) {
    const summary = title.replace(/^[a-z]+:\s*/i, "").trim() || `implement issue #${issue.number}`;
    prTitle = `${inferType(issue.labels)}: ${summary}`;
  }

  if (!conventionalPattern.test(prTitle)) {
    prTitle = `chore: implement issue #${issue.number}`;
  }

  return prTitle.length > 120 ? `${prTitle.slice(0, 117).trimEnd()}...` : prTitle;
}

function inferType(labels = []) {
  const names = labels.map((label) => label.name);
  if (names.includes("bug")) return "fix";
  if (names.includes("documentation")) return "docs";
  if (names.includes("enhancement")) return "feat";
  return "chore";
}
