import type { SvelteHtmlComponentRenderer } from "./html-host";

export const defaultRenderSvelteHtmlComponent: SvelteHtmlComponentRenderer = async (
  component,
  props,
  slotHtml,
) => {
  const [{ render }, { createRawSnippet }] = await Promise.all([
    import("svelte/server"),
    import("svelte"),
  ]);
  const componentProps = slotHtml
    ? { ...props, children: createRawSnippet(() => ({ render: () => slotHtml })) }
    : props;
  const rendered = render(component as never, { props: componentProps as never }) as {
    body?: string;
    html?: string;
  };
  return rendered.html ?? rendered.body ?? "";
};
