---
title: Credits
description: Community credits and contribution summary for Ox Content.
---

# Credits

Ox Content is maintained by [ubugeeei](https://github.com/ubugeeei).

This page records community contributions that shaped the project.

## Community Credits

### kazupon

Special thanks to [kazupon](https://github.com/kazupon) for substantial
community contributions around JSDoc support and documentation quality.

Contribution summary:

- Helped shape JSDoc support as a first-class API documentation workflow.
- Contributed to the API docs generation pipeline used by Ox Content's own
  documentation.
- Improved documentation quality around generated API docs and user-facing
  docs.

### ryoppippi

Special thanks to [ryoppippi](https://github.com/ryoppippi) for production
migration feedback on Markdown attributes and rich social embed parity.

Contribution summary:

- Reported the inline link and transformed image attribute target regression
  found during the ryoppippi.com Ox Content migration.
- Helped validate the expected Twitter/X full-card visual contract through
  sveltweet.
- Requested self-hosted web font acquisition for the built-in theme during the
  ryoppippi.com migration.
- Requested self-hosted Iconify CSS for used icons during the ryoppippi.com
  migration.
- Reported the need for host-only redirect output without redundant HTML
  fallback pages during the ryoppippi.com migration.
- Reported that closing Vite middleware servers could repeat production SSG
  output writes during the ryoppippi.com migration.
- The opt-in `<NotByAI />` authorship badge was requested and first shipped
  in production during the ryoppippi.com Ox Content migration.
- Reported that generated feed item URLs missed `ssg.routePrefix` while page
  output and Markdown companions were mounted under the prefix.
- Requested programmatic feed item sources for the ryoppippi.com JSON-backed
  media feed migration.
- Reported prose typography stylesheet interactions with rich magic-link
  avatars during the ryoppippi.com migration.
- Reported the custom-host Markdown table migration that led to the
  browser-only table helper entrypoint and isolated table stylesheet.
- Requested class-based dark-mode support for Twitter/X full-card styles.
- Reported Twitter/X full-card action control and replies-placement geometry
  mismatches against sveltweet production cards.
- Requested the official Twitter/X Copy link client API and verified the
  downstream migration without a site-specific listener.

## Third-party attribution

### react-tweet and sveltweet

The opt-in Twitter/X `appearance: "full"` card is static HTML and CSS. Its
visual contract — layout, color tokens, and control icons — follows
[react-tweet](https://github.com/vercel/react-tweet) (MIT, Copyright (c) 2023
Luis Alvarez) and [sveltweet](https://github.com/ryoppippi/sveltweet) (MIT,
Copyright (c) 2024 ryoppippi). Ox Content does not depend on those packages at
runtime.

The MIT copyright notice and permission notice for both projects are
reproduced in `crates/ox_content_ssg/src/plugins/social-tweet-full.css`.

X, Twitter, and related marks are trademarks of their respective owners.

### Not By AI badge artwork

The opt-in `<NotByAI />` badge vendors the official light and dark “Written
by Human, Not By AI” SVGs from [Not By AI](https://notbyai.fyi). The copies
are sanitized at vendoring time and inlined as static HTML. Ox Content does
not load scripts or assets from notbyai.fyi at runtime.

Not By AI and related marks are trademarks of their respective owners. See
the [Not By AI usage guidelines](https://notbyai.fyi) for eligibility and
commercial-use terms.
