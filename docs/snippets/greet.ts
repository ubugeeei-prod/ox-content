export interface Greeting {
  name: string;
  message: string;
}

// #region greet
export function greet(name: string): Greeting {
  return {
    name,
    message: `Hello, ${name}!`,
  };
}
// #endregion greet

export function farewell(name: string): string {
  return `Goodbye, ${name}.`;
}
