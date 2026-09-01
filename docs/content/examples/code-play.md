---
title: Code Play
description: Opt-in on-demand sample execution with stdio, stderr, config, provenance, and timing viewers.
---

# Code Play

This page uses `@ox-content/code-play` with **JavaScript**, **TypeScript**,
**Rust**, and **Go** enabled. Other languages stay ordinary fences until a site
opts them in. The standalone `examples/code-play` app uses the same Rust and Go
playground adapters, and renders Python with an explicit remote executor when
`OX_CODE_PLAY_PYTHON_ENDPOINT` is set.
Routes without a `play` fence stay ordinary docs pages and do not load
`ox-code-play.js`.

A copy-paste Vite app lives at
[`examples/code-play`](https://github.com/ubugeeei-prod/ox-content/tree/main/examples/code-play)
in the repository.

## Enable the plugin

```ts
import { oxContent } from "@ox-content/vite-plugin";
import { codePlay } from "@ox-content/code-play";

export default {
  plugins: [
    oxContent({ highlight: true }),
    codePlay({
      languages: {
        javascript: true,
        typescript: { execute: true, typecheck: true },
        rust: true,
        go: true,
        python: { endpoint: "https://piston.example/api/v2/piston" },
      },
      ui: "default",
      viewers: { config: true, stdio: true, stderr: true, provenance: true, timing: true },
    }),
  ],
};
```

## Live TypeScript sample

The fence below is marked `play`. Use **Run** to execute it. **Typecheck**
appears during `vite dev` (the `/__ox-code-play/typecheck` proxy) or when
the site sets a reachable `endpoints.typecheck`. Published pages still run
TypeScript by stripping types into the sandbox iframe. The stdio, stderr,
config, provenance, and timing tabs are the same objects the headless API
returns. `console.warn` lands in `run.stderr`.

```ts play typecheck play-title="Strict TypeScript" play-target=ESNext
const message: string = "hello from Code Play";
console.log(message);
console.warn("this warning is a stderr chunk");
```

## Per-sample config

`play-<config-key>=...` overrides the language config for one sample. This
sample intentionally disables strict TypeScript checking while keeping the
page-level defaults strict.

```ts play typecheck play-title="Loose TypeScript" play-strict=false play-compact
const label = "works without an explicit type annotation";
console.log(label.toUpperCase());
```

## Live JavaScript sample

```js play play-title="JavaScript sum"
function add(left, right) {
  return left + right;
}

console.log(add(2, 40));
```

## Live Rust sample

This sample uses the official Rust playground adapter. It slugifies headings
from a small Markdown document and asserts the result before printing the
navigation targets.

```rust play typecheck play-title="Rust heading slugs" play-mode=release play-edition=2024
#[derive(Debug, PartialEq, Eq)]
struct Heading {
    level: usize,
    text: String,
    slug: String,
}

fn collect_headings(markdown: &str) -> Vec<Heading> {
    markdown
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            let level = trimmed.chars().take_while(|&ch| ch == '#').count();
            if level == 0
                || level > 6
                || !trimmed
                    .as_bytes()
                    .get(level)
                    .is_some_and(|byte| byte.is_ascii_whitespace())
            {
                return None;
            }
            let text = trimmed[level..].trim();
            Some(Heading {
                level,
                text: text.to_string(),
                slug: slugify(text),
            })
        })
        .collect()
}

fn slugify(text: &str) -> String {
    let mut slug = String::new();
    let mut pending_dash = false;

    for ch in text.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            if pending_dash && !slug.is_empty() {
                slug.push('-');
            }
            slug.push(ch);
            pending_dash = false;
        } else if ch.is_whitespace() || matches!(ch, '-' | '_' | ':' | '/') {
            pending_dash = true;
        }
    }

    if slug.is_empty() {
        "section".to_string()
    } else {
        slug
    }
}

fn main() {
    let markdown = "# Code Play\n\n## Rust Runner\n\n### Stdio & Timing";
    let headings = collect_headings(markdown);
    assert_eq!(
        headings.iter().map(|heading| heading.slug.as_str()).collect::<Vec<_>>(),
        ["code-play", "rust-runner", "stdio-timing"]
    );

    for heading in &headings {
        println!("h{} {} -> #{}", heading.level, heading.text, heading.slug);
    }
}
```

## Live Go sample

The Go sample uses the Go playground adapter with vet enabled. It counts fenced
code blocks by language, sorts the result, and prints a small summary.

````go play typecheck play-title="Go fence summary"
package main

import (
	"fmt"
	"sort"
	"strings"
)

type Fence struct {
	Language string
	Lines    int
}

func collectFences(markdown string) []Fence {
	var fences []Fence
	inFence := false
	current := Fence{Language: "text"}

	for _, line := range strings.Split(markdown, "\n") {
		if strings.HasPrefix(line, "```") {
			if inFence {
				fences = append(fences, current)
				inFence = false
				current = Fence{Language: "text"}
				continue
			}
			language := strings.TrimSpace(strings.TrimPrefix(line, "```"))
			if language == "" {
				language = "text"
			}
			current = Fence{Language: language}
			inFence = true
			continue
		}
		if inFence {
			current.Lines++
		}
	}

	return fences
}

func main() {
	markdown := strings.Join([]string{
		"# Samples",
		"",
		"```go",
		`fmt.Println("ok")`,
		"```",
		"",
		"```rust",
		`println!("ok");`,
		"```",
	}, "\n")

	fences := collectFences(markdown)
	sort.Slice(fences, func(i, j int) bool {
		return fences[i].Language < fences[j].Language
	})

	if len(fences) != 2 {
		panic("expected two fenced code blocks")
	}

	for _, fence := range fences {
		fmt.Printf("%s: %d line(s)\n", fence.Language, fence.Lines)
	}
}
````

## Typecheck failure

During `vite dev`, **Typecheck** should fail on this sample. On a published
page the button is omitted unless `endpoints.typecheck` is set. **Run** still
executes after types are stripped, so execute and type-check stay separate.

```ts play typecheck play-title="Typecheck failure"
const n: number = "not a number";
console.log(n);
```

## Runtime error

`throw` becomes a diagnostic and a stderr chunk. The stderr tab opens when the
run produces stderr or an error diagnostic.

```js play play-title="Runtime error"
console.log("before");
throw new Error("boom from the example");
```

## Headless usage

```ts
import { createCodePlay } from "@ox-content/code-play";

const play = createCodePlay({ languages: { typescript: true } });
const session = play.createSession({
  language: "ts",
  code: "const n: number = 1;",
});

const result = await session.run();
result.stdio;
result.stdout;
result.stderr;
result.provenance.compile;
result.provenance.execute;
result.timing.phases;
```

`RunActionState` helpers model idle, running, result, error, and offline states
for custom UIs. Transport/CORS failures return `status: "offline"`.
`ui: "compact"` hides the tab list and keeps stdio plus stderr. `ui: "headless"`
renders no chrome — use `createCodePlay()` from your own UI.

## Remote languages

Rust and Go use typed playground adapters. During `vite dev`, their browser
payloads use the Vite dev proxy by default; production builds embed
`endpoints.rust` and `endpoints.go`. Python uses the generic remote adapter and
needs a Piston-compatible `languages.python.endpoint`.

````md
```python play play-title="Python via Piston"
print("ok")
```
````

If Python is enabled without an endpoint, **Run** reports
`status: "unsupported"` and explains that a configured HTTP executor is
required. Transport and CORS failures report `status: "offline"`.

See [@ox-content/code-play](../packages/code-play.md) and the
[roadmap](../code-play-roadmap.md).
