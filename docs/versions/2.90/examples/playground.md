# Playground Example

Interactive web playground for testing Markdown parsing.

## Setup

```bash
# Start docs and playground together
vp run dev

# Or run only the playground
vp run playground
```

## Features

- **Live Preview**: Edit Markdown and see the rendered result immediately
- **HTML Output**: Inspect the generated HTML in a dedicated pane
- **Pseudo AST Viewer**: Inspect a browser-friendly AST-style structure
- **Minimal UI**: A simple split view with source on the left and output on the right
- **Copy Actions**: Copy the current source or rendered output with one click

## Architecture

The playground currently uses a lightweight browser-side parser shim:

```ts
const result = parseMarkdown(input);

preview.innerHTML = result.html;
htmlPane.textContent = result.html;
astPane.textContent = result.ast;
```

This keeps the demo easy to run in a plain Vite app while still making the
output inspectable. Use native bindings when you need parser-accurate behavior.

## Running

```bash
# Start docs and playground together
vp run dev

# Start only the playground
vp run playground

# Build for production
cd examples/playground
vp build

# Preview production build
vp preview
```
