# Code Annotations

```ts annotate="highlight:1,4;warning:2;error:3"
export function loadUser(input: string) {
  if (!input) console.warn("missing payload");
  throw new Error("missing id");
}
```

```ts {1,3} [config.ts]
const token = readToken();
refreshToken(token);
console.warn("Token expires soon"); // [!code warning]
```
