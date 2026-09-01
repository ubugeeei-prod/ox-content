# API Reference

Complete API documentation for the package.

## Functions

### createApp

Creates a new application instance.

```typescript
function createApp(config: Config): App;
```

**Parameters:**

- `config` - Configuration object

**Returns:** `App` instance

**Example:**

```typescript
const app = createApp({
  name: "my-app",
});
```

### defineConfig

Helper function to define configuration with type safety.

```typescript
function defineConfig(config: UserConfig): Config;
```

**Parameters:**

- `config` - User configuration object

**Returns:** Resolved configuration

## Types

### Config

```typescript
interface Config {
  name: string;
  version: string;
  plugins: Plugin[];
  debug: boolean;
}
```

### Plugin

```typescript
interface Plugin {
  name: string;
  setup(app: App): void | Promise<void>;
}
```

### App

```typescript
interface App {
  config: Config;
  start(): Promise<void>;
  stop(): Promise<void>;
  use(plugin: Plugin): void;
}
```

## Constants

| Name           | Value     | Description         |
| -------------- | --------- | ------------------- |
| `VERSION`      | `'1.0.0'` | Package version     |
| `DEFAULT_PORT` | `3000`    | Default server port |
