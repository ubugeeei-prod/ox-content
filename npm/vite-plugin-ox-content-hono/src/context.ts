import { createContext, useContext } from "hono/jsx";

export type Frontmatter = Readonly<Record<string, unknown>>;
/** Values deliberately shared with both SSR and the browser. */
export const FrontmatterContext = createContext<Frontmatter>(Object.freeze({}));
export function useFrontmatter<T extends Frontmatter = Frontmatter>(): T {
  return useContext(FrontmatterContext) as T;
}
