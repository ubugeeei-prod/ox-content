---
title: Unified Bridge Demo
badge: mdast bridge
stage: mdast -> remark -> html
---

# Existing unified plugins still run

This page starts as plain Markdown and is then processed by the Ox Content unified bridge.

## What the bridge changes

- The custom mdast plugin appends a badge to the first heading.
- An existing remark plugin reads `vfile.data.matter` and appends a summary paragraph.
- A final ox-content HTML plugin wraps the output in an `<article>` and prepends reading time.

## Why this matters

You can keep the native Ox Content parser, but still run the existing unified ecosystem at the mdast stage.

### Signals to look for in the rendered result

- The top heading should include `[mdast bridge]`.
- The final paragraph should mention the frontmatter title and stage.
- The table of contents should stay aligned with the transformed heading text.
