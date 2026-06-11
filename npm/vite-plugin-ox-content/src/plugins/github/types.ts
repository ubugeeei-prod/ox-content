export interface GitHubRepoData {
  name: string;
  full_name: string;
  description: string | null;
  html_url: string;
  stargazers_count: number;
  forks_count: number;
  language: string | null;
  owner: {
    login: string;
    avatar_url: string;
  };
}

export interface GitHubLineRange {
  start: number;
  end: number;
}

export interface GitHubSourceRef {
  repo: string;
  ref: string;
  path: string;
  permalink: string;
  lines?: GitHubLineRange;
}

export interface GitHubSourceData {
  repo: string;
  ref: string;
  path: string;
  permalink: string;
  content: string;
  size: number;
  html_url: string;
  language: string | null;
}

export interface GitHubOptions {
  /**
   * GitHub API token used for higher rate limits and private repository access.
   * @default ''
   */
  token?: string;

  /**
   * Cache fetched repository and source data in memory for the current process.
   * @default true
   */
  cache?: boolean;

  /**
   * Cache TTL in milliseconds.
   * @default 3600000
   */
  cacheTTL?: number;

  /**
   * Maximum source file size to inline in bytes.
   * @default 200000
   */
  maxSourceBytes?: number;

  /**
   * Maximum source lines to inline when no line range is specified.
   * @default 120
   */
  maxSourceLines?: number;
}

export const defaultOptions: Required<GitHubOptions> = {
  token: "",
  cache: true,
  cacheTTL: 3600000,
  maxSourceBytes: 200000,
  maxSourceLines: 120,
};
