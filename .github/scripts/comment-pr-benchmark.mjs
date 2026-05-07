#!/usr/bin/env node

import { readFileSync } from "node:fs";

const COMMENT_MARKER = "<!-- ox-content-benchmark-report -->";

const token = requiredEnv("GITHUB_TOKEN");
const [owner, repo] = requiredEnv("GITHUB_REPOSITORY").split("/");
const event = JSON.parse(readFileSync(requiredEnv("GITHUB_EVENT_PATH"), "utf8"));
const issueNumber = event.pull_request?.number ?? event.number;

if (!owner || !repo) {
  throw new Error("GITHUB_REPOSITORY must be in owner/repo format");
}
if (!issueNumber) {
  throw new Error("Pull request number was not found in the event payload");
}

const body = readFileSync(requiredEnv("COMMENT_PATH"), "utf8");
const comments = await listComments(owner, repo, issueNumber);
const previous = comments.find((comment) => {
  return comment.user?.type === "Bot" && comment.body?.includes(COMMENT_MARKER);
});

if (previous) {
  await request(`/repos/${owner}/${repo}/issues/comments/${previous.id}`, {
    method: "PATCH",
    body: JSON.stringify({ body }),
  });
} else {
  await request(`/repos/${owner}/${repo}/issues/${issueNumber}/comments`, {
    method: "POST",
    body: JSON.stringify({ body }),
  });
}

/**
 * @param {string} owner
 * @param {string} repo
 * @param {number} issueNumber
 * @returns {Promise<unknown[]>}
 */
async function listComments(owner, repo, issueNumber) {
  const comments = [];

  for (let page = 1; ; page++) {
    const pageComments = await request(
      `/repos/${owner}/${repo}/issues/${issueNumber}/comments?per_page=100&page=${page}`,
    );
    comments.push(...pageComments);
    if (pageComments.length < 100) {
      return comments;
    }
  }
}

/**
 * @param {string} path
 * @param {RequestInit} init
 * @returns {Promise<any>}
 */
async function request(path, init = {}) {
  const response = await fetch(`https://api.github.com${path}`, {
    ...init,
    headers: {
      Accept: "application/vnd.github+json",
      Authorization: `Bearer ${token}`,
      "Content-Type": "application/json",
      "X-GitHub-Api-Version": "2022-11-28",
    },
  });

  if (!response.ok) {
    const responseBody = await response.text();
    throw new Error(`GitHub API request failed: ${response.status} ${responseBody}`);
  }

  if (response.status === 204) {
    return null;
  }

  return response.json();
}

/**
 * @param {string} name
 * @returns {string}
 */
function requiredEnv(name) {
  const value = process.env[name];
  if (!value) {
    throw new Error(`${name} is required`);
  }

  return value;
}
