import type {
  OxContentCustomHostMarkdownInput,
  OxContentCustomHostMarkdownRenderer,
} from "./custom-host-markdown";
import type { OxContentCustomHostBaseContext } from "./custom-host-types";

/** Keep hosts that never render Markdown from initializing its runtime. */
export function createLazyCustomHostMarkdownRenderer(
  context: Omit<OxContentCustomHostBaseContext, "markdown">,
): OxContentCustomHostMarkdownRenderer {
  let renderer: OxContentCustomHostMarkdownRenderer | undefined;
  let pending: Promise<OxContentCustomHostMarkdownRenderer> | undefined;
  return {
    async render<Metadata>(input: OxContentCustomHostMarkdownInput<Metadata>) {
      renderer ??= await (pending ??= import("./custom-host-markdown")
        .then(({ createCustomHostMarkdownRenderer }) => createCustomHostMarkdownRenderer(context))
        .catch((error: unknown) => {
          pending = undefined;
          throw error;
        }));
      return renderer.render(input);
    },
  };
}
