---
title: Code Play example
description: Minimal Vite site that runs documentation samples on demand.
---

# Code Play example

This site enables **JavaScript**, **TypeScript**, **Rust**, **Go**, and
**Python**. Mark a fence with `play`. **Typecheck** shows for TypeScript during
`vite dev`, or on a published page if you set `endpoints.typecheck`. Rust and
Go use official playground adapters; Python uses a Piston-compatible endpoint
when `OX_CODE_PLAY_PYTHON_ENDPOINT` is set.

See the [plain page](./plain.md) for a route with no Code Play runtime.

```ts play typecheck play-title="Strict TypeScript" play-target=ESNext
const message: string = "hello from examples/code-play";
console.log(message);
console.warn("stderr from console.warn");
```

```ts play typecheck play-title="Loose TypeScript" play-strict=false play-compact
const label = "per-sample config";
console.log(label);
```

```js play play-title="JavaScript sum"
console.log(2 + 40);
```

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

```python play play-title="Python remote executor"
print("hello from Python Code Play")
```

See [@ox-content/code-play](https://github.com/ubugeeei-prod/ox-content/blob/main/docs/content/packages/code-play.md)
for the full language list and the headless API.
