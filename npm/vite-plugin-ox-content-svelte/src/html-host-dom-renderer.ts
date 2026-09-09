import type {
  SvelteHtmlHostClientRenderer,
  SvelteHtmlHostDomMode,
  SvelteHtmlHostDomRendererInput,
  SvelteHtmlHostDomRuntime,
} from "./html-host-client-types";

const DOM_RENDERER_MODE = Symbol("svelte-html-host-dom-renderer-mode");

export type SvelteHtmlHostDomRenderer = SvelteHtmlHostClientRenderer<SvelteHtmlHostDomRuntime>;

type ModeTaggedSvelteHtmlHostDomRenderer = SvelteHtmlHostDomRenderer & {
  readonly [DOM_RENDERER_MODE]: SvelteHtmlHostDomMode;
};

export async function loadSvelteHtmlHostDomRuntime(): Promise<SvelteHtmlHostDomRuntime> {
  const { createRawSnippet, hydrate, mount, unmount } = await import("svelte");

  return {
    createRawSnippet: createRawSnippet as SvelteHtmlHostDomRuntime["createRawSnippet"],
    hydrate: hydrate as SvelteHtmlHostDomRuntime["hydrate"],
    mount: mount as SvelteHtmlHostDomRuntime["mount"],
    unmount: unmount as SvelteHtmlHostDomRuntime["unmount"],
  };
}

export function createSvelteHtmlHostDomRenderer(
  input: SvelteHtmlHostDomRendererInput,
): SvelteHtmlHostDomRenderer {
  const renderer = ((context) => {
    const runtime = context.runtime;
    if (!runtime) {
      throw new Error("Svelte HTML host DOM renderer requires a Svelte runtime.");
    }
    const componentProps = componentPropsWithSlot(runtime, context.props, context.slotHtml);
    const instance =
      input.mode === "hydrate"
        ? runtime.hydrate(context.component, { target: context.element, props: componentProps })
        : runtime.mount(context.component, { target: context.element, props: componentProps });
    return () => {
      void runtime.unmount(instance);
    };
  }) as ModeTaggedSvelteHtmlHostDomRenderer;

  Object.defineProperty(renderer, DOM_RENDERER_MODE, { value: input.mode });
  return renderer;
}

export function svelteHtmlHostDomRendererMode(
  renderer: SvelteHtmlHostClientRenderer<unknown>,
): SvelteHtmlHostDomMode | undefined {
  return (renderer as Partial<ModeTaggedSvelteHtmlHostDomRenderer>)[DOM_RENDERER_MODE];
}

function componentPropsWithSlot(
  runtime: SvelteHtmlHostDomRuntime,
  props: Record<string, unknown>,
  slotHtml: string | undefined,
): Record<string, unknown> {
  if (!slotHtml) return props;
  return {
    ...props,
    children: runtime.createRawSnippet(() => ({
      render: () => slotHtml,
      setup: (element) => {
        element.innerHTML = slotHtml;
      },
    })),
  };
}
