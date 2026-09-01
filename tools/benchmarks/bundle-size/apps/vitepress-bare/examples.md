# Examples

Real-world examples of using this package.

## Basic Example

A simple "Hello World" application:

```typescript
import { createApp } from "my-package";

const app = createApp({
  name: "hello-world",
});

app.start().then(() => {
  console.log("App started!");
});
```

## With Plugins

Using plugins to extend functionality:

```typescript
import { createApp } from "my-package";
import { loggerPlugin } from "my-package/plugins";

const app = createApp({
  name: "with-plugins",
  plugins: [loggerPlugin()],
});

app.start();
```

## Advanced Configuration

Full configuration example:

```typescript
import { createApp, defineConfig } from "my-package";

const config = defineConfig({
  name: "advanced-app",
  version: "2.0.0",
  debug: process.env.NODE_ENV === "development",
  plugins: [
    {
      name: "custom-plugin",
      setup(app) {
        console.log(`Setting up ${app.config.name}`);
      },
    },
  ],
});

const app = createApp(config);

// Graceful shutdown
process.on("SIGTERM", () => {
  app.stop();
});

app.start();
```

## Task List

- [x] Basic setup
- [x] Add plugins
- [ ] Deploy to production
- [ ] Write tests

## Notes

> **Important:** Always handle errors properly in production.

1. First, install dependencies
2. Then, configure the app
3. Finally, start the server
