---
title: Code Groups
description: Opt-in VitePress-style ::: code-group fences that reuse the existing no-JS tab widget.
---

# Code Groups

Adjacent code samples can share the existing accessible tab UI without
hand-writing `<tabs>` markup. The feature is opt-in: omitted or `false`
leaves `::: code-group` on the normal Markdown or custom-container path.

| Option       | Type                           | Default |
| ------------ | ------------------------------ | ------- |
| `codeGroups` | `boolean` / `CodeGroupOptions` | `false` |

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      codeGroups: true,
    }),
  ],
};
```

Passing `{}` also enables the defaults. The rewrite is static: it reuses the
no-JS `ox-tabs` widget and `<noscript>` details fallback. No extra client
JavaScript is shipped.

## Authoring

### Before (`<tabs>`)

```html
<tabs>
  <tab label="config.js">
    <pre><code>export default {}</code></pre>
  </tab>
  <tab label="config.ts">
    <pre><code>export default {}</code></pre>
  </tab>
</tabs>
```

### After (`::: code-group`)

::: code-group

```js [config.js]
export default {};
```

```ts [config.ts]
export default {};
```

:::

````md
::: code-group

```js [config.js]
export default {};
```

```ts [config.ts]
export default {};
```

:::
````

Tab titles come from ` ```ts [label] ` or fence meta such as
`title="config.ts"`. When neither is present, the language identifier is
used (`js`, `ts`). A fence with no language falls back to `Tab 1`,
`Tab 2`, and so on.

An unclosed `::: code-group` stays literal and does not consume the rest of
the file. Unknown or malformed group metadata degrades to ordinary fences
and emits a transform warning. Groups written inside fenced or indented
code are left alone.

If custom containers are also enabled, `code-group` is handled by this
feature, not as an unknown container type.

Existing `<tabs>` and `<pm>` blocks are unchanged.

## Related

- [Code Blocks](./code-blocks.md)
- [Embeds](./embeds.md#tabs)
- [Custom Containers](./containers.md)
- [Built-in Features overview](../built-in-features.md)
