---
title: Code Play
description: stdio、stderr、config、provenance、timing ビューアー付きのオンデマンドサンプル実行です。
---

# Code Play

このページは `@ox-content/code-play` で **JavaScript**、**TypeScript**、**Rust**、
**Go** を有効にします。それ以外の言語は、サイト側で opt-in するまで通常の
code fence のままです。Code Play がないページは通常の docs ページのままで、
`ox-code-play.js` を読み込みません。

## プラグインを有効にする

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
      },
      ui: "default",
      viewers: { config: true, stdio: true, stderr: true, provenance: true, timing: true },
    }),
  ],
};
```

## TypeScript サンプル

```ts play typecheck play-title="Strict TypeScript" play-target=ESNext
const message: string = "hello from Code Play";
console.log(message);
console.warn("this warning is a stderr chunk");
```

## サンプルごとの config

`play-<config-key>=...` は、そのサンプルだけ言語 config を上書きします。

```ts play typecheck play-title="Loose TypeScript" play-strict=false play-compact
const label = "works without an explicit type annotation";
console.log(label.toUpperCase());
```

## Rust サンプル

公式 Rust playground adapter で動きます。小さな Markdown 文字列から見出しを集め、
slug を検証してからナビゲーション先を出力します。

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

## Go サンプル

Go playground adapter で vet も有効にして動きます。fenced code block を言語ごとに
集計し、ソートした結果を出力します。

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

## 実行時エラー

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

`RunActionState` ヘルパーは idle、running、result、error、offline を表します。
transport / CORS の失敗は `status: "offline"` です。

詳しくは [@ox-content/code-play](/packages/code-play.md) と
[ロードマップ](/code-play-roadmap.md) を見てください。
