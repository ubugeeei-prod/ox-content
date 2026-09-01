# Getting Started

Learn how to set up and use this package.

## Installation

You can install via npm, yarn, or pnpm:

```bash
# npm
npm install my-package

# yarn
yarn add my-package

# pnpm
pnpm add my-package
```

## Basic Usage

Import and use the main function:

```typescript
import { createApp, defineConfig } from "my-package";

const config = defineConfig({
  name: "my-app",
  version: "1.0.0",
  plugins: [],
});

const app = createApp(config);
app.start();
```

## Configuration

| Option    | Type       | Default   | Description       |
| --------- | ---------- | --------- | ----------------- |
| `name`    | `string`   | required  | Application name  |
| `version` | `string`   | `'0.0.0'` | Version string    |
| `plugins` | `Plugin[]` | `[]`      | List of plugins   |
| `debug`   | `boolean`  | `false`   | Enable debug mode |

## Next Steps

- Read the [API Reference](./api.md)
- Check out [Examples](./examples.md)
