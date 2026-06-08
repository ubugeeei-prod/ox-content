export function greet(name: string) {
  return `Hello, ${name}!`;
}

// #region demo
export const message = greet("docs");
console.log(message);
// #endregion demo
