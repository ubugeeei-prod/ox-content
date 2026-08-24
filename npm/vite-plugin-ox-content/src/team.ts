/**
 * Opt-in team / members page helpers.
 *
 * Resolution lives here. Member cards are rendered in Rust
 * (`ox_content_ssg::render_team_page`) when a page has `layout: team`.
 */

import type { ResolvedTeamOptions, TeamMember, TeamOptions } from "./types";

/**
 * Resolves `ssg.team` with defaults.
 *
 * `false` / omitted stays off. `true` enables an empty member list.
 * An object enables the feature and keeps the members the site set.
 */
export function resolveTeamOptions(value: boolean | TeamOptions | undefined): ResolvedTeamOptions {
  if (!value) {
    return { enabled: false, members: [] };
  }
  if (value === true) {
    return { enabled: true, members: [] };
  }
  return {
    enabled: true,
    members: normalizeMembers(value.members),
  };
}

function normalizeMembers(members: TeamMember[] | undefined): TeamMember[] {
  if (!Array.isArray(members)) {
    return [];
  }
  return members.flatMap((member) => {
    if (!member || typeof member.name !== "string") {
      return [];
    }
    const links = Array.isArray(member.links)
      ? member.links.flatMap((link) => {
          if (!link || typeof link.label !== "string" || typeof link.href !== "string") {
            return [];
          }
          return [{ label: link.label, href: link.href }];
        })
      : undefined;
    return [
      {
        name: member.name,
        role: typeof member.role === "string" ? member.role : undefined,
        avatar: typeof member.avatar === "string" ? member.avatar : undefined,
        links,
      },
    ];
  });
}
