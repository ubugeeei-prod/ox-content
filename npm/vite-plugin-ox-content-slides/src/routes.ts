import type {
  ResolvedSlidesPluginOptions,
  SlideBuildArtifacts,
  SlideDeckData,
  SlideRouteData,
} from "./internal-types";
import { getSlideRouteLookupKey } from "./path-utils";

/**
 * Builds a route lookup map for dev-server slide rendering.
 */
export function buildSlideArtifacts(
  options: ResolvedSlidesPluginOptions,
  decks: SlideDeckData[],
): SlideBuildArtifacts {
  const routes = new Map<string, SlideRouteData>();

  for (const deck of decks) {
    for (const slide of deck.slides) {
      const slideKey = getSlideRouteLookupKey(options, slide.href);
      if (slideKey) {
        routes.set(slideKey, { deck, slide, presenter: false });
      }

      if (!slide.presenterHref) {
        continue;
      }

      const presenterKey = getSlideRouteLookupKey(options, slide.presenterHref);
      if (presenterKey) {
        routes.set(presenterKey, { deck, slide, presenter: true });
      }
    }
  }

  return { routes };
}
