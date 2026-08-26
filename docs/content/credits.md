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
