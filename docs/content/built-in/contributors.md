---
title: Git Contributors
description: Opt-in unique git authors rendered under each article.
---

# Git Contributors

When `ssg.contributors` is enabled, each article lists the unique git authors
of its source file. Names are escaped. The feature is off unless you turn it
on. Existing sites stay unchanged.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      ssg: {
        contributors: true,
      },
    }),
  ],
};
```

`false` or omitted keeps the list off. `true` enables names only. An object
enables the feature and can set `ignore` and `avatars`.

```ts
oxContent({
  ssg: {
    contributors: {
      ignore: ["dependabot[bot]", "ci@example.com"],
      avatars: true,
    },
  },
});
```

| Option             | Type                                | Default |
| ------------------ | ----------------------------------- | ------- |
| `ssg.contributors` | `boolean` / `{ ignore?, avatars? }` | `false` |
| `ignore`           | `string[]`                          | `[]`    |
| `avatars`          | `boolean`                           | `false` |

Authors come from `git log --format=%an%x09%ae` on the source file. Duplicates
are merged by email when an email is present, otherwise by name. Comparison is
case-insensitive. The first name seen for that key is the one that is rendered.
Commit counts are available from the native helper but are not shown in the
default theme.

The ignore list matches a full author name or a full email, case-insensitive.
Partial strings do not match. Ignored authors are dropped before HTML is
generated.

## Avatars

Names only is the default. Set `avatars: true` to load a Gravatar image when
the git author email is present. The image URL is
`https://www.gravatar.com/avatar/{md5(email)}`. The raw email is never written
into HTML, `mailto:` links, or `href`.

Turning avatars on hashes the git author email and requests
`gravatar.com`. Sites that must not publish author-email hashes should leave
`avatars` off.

No GitHub profile URL is invented from an email. A `https://github.com/{login}`
link is not emitted unless a later option supplies an explicit login. This
version does not accept that mapping.

Unsafe avatar URLs (`javascript:`, `data:`, protocol-relative `//`, `http:`)
are omitted. Only `https:` avatar URLs reach the markup.

## Missing `.git`

A published npm tarball, a CI checkout without history, or a path that is not
inside a git work tree yields an empty list. The build does not fail. The page
emits no contributor markup and no warning is required at runtime. This page
is the documented behavior.

The native helper `getGitContributors(filePath, root?)` returns `[]` when
`root` is omitted, git is missing, `git log` fails, or the file has no
commits. Results are cached per file path and `HEAD` commit for the life of
the process.

## Markup

The list is rendered under the article, after last-updated when that option is
also on. The container uses `.contributors`. Each author is a `.contributor`
item with an optional `.contributor-avatar` image and a `.contributor-name`
span. Hostile names are escaped (`<`, `"`, and the other HTML specials).

Bare mode still receives the same `contributors` array on page data. The
default bare template does not emit the list; a custom `ssg.render` theme can
read `page.contributors`.

## Related

- [Site Generation](./site-generation.md)
- [Last updated](./site-generation.md)
- [Built-in Features overview](../built-in-features.md)
