// Circular reveal for a same-document colour-scheme change.
//
// Prior art: @hooray's VitePress implementation, by way of @ryoppippi's
// svelte-fancy-darkmode, which this is a framework-neutral port of.
//
// This wraps a *same-document* theme mutation. It is unrelated to the
// cross-document `@view-transition` used for MPA navigation, and the CSS it
// depends on is scoped to `data-ox-theme-transition` so the two lifecycles
// cannot reach each other's snapshots.

const OX_THEME_TRANSITION_ATTR = "data-ox-theme-transition";
const OX_THEME_TRANSITION_SUPPRESS_ATTR = "data-ox-theme-transition-suppress";

let oxThemeTransitionCurrent = null;

function oxThemeTransitionRoot() {
  return typeof document === "undefined" ? null : document.documentElement;
}

function oxThemeTransitionReducedMotion() {
  return typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches;
}

function oxThemeTransitionSupported(root) {
  return Boolean(
    root &&
    typeof document !== "undefined" &&
    typeof document.startViewTransition === "function" &&
    typeof root.animate === "function",
  );
}

// A keyboard or programmatic activation still produces a click event, but with
// no meaningful coordinates: `detail` is 0 and clientX/clientY report 0. Only
// a real pointer press is trusted, so the reveal never starts from the corner
// of the screen when someone tabs to the button.
function oxThemeTransitionOrigin(event) {
  const viewportCentre = {
    x: (typeof innerWidth === "number" ? innerWidth : 0) / 2,
    y: (typeof innerHeight === "number" ? innerHeight : 0) / 2,
  };
  if (!event) return viewportCentre;

  const fromPointer =
    typeof event.clientX === "number" &&
    typeof event.clientY === "number" &&
    (event.detail > 0 || (typeof event.pointerType === "string" && event.pointerType !== ""));
  if (fromPointer) return { x: event.clientX, y: event.clientY };

  const target = event.currentTarget ?? event.target;
  if (target && typeof target.getBoundingClientRect === "function") {
    const rect = target.getBoundingClientRect();
    if (rect.width > 0 || rect.height > 0) {
      return { x: rect.left + rect.width / 2, y: rect.top + rect.height / 2 };
    }
  }
  return viewportCentre;
}

// The circle has to reach whichever viewport corner is furthest away, or the
// old theme stays visible in one corner when the animation ends.
function oxThemeTransitionRadius(origin) {
  const width = typeof innerWidth === "number" ? innerWidth : 0;
  const height = typeof innerHeight === "number" ? innerHeight : 0;
  return Math.hypot(Math.max(origin.x, width - origin.x), Math.max(origin.y, height - origin.y));
}

// Element-level colour transitions would otherwise animate underneath the
// snapshot and bleed the old palette into the new one. They are suppressed
// only while the mutation happens, and restored as soon as both snapshots
// have been captured.
function oxThemeTransitionSuppress(root) {
  root.setAttribute(OX_THEME_TRANSITION_SUPPRESS_ATTR, "");
  return () => {
    // Reading a layout property flushes the suppressed styles before they are
    // allowed to animate again.
    void root.offsetHeight;
    root.removeAttribute(OX_THEME_TRANSITION_SUPPRESS_ATTR);
  };
}

// eslint-disable-next-line no-unused-vars
function applyThemeTransition(options) {
  const settings = options ?? {};
  const apply = settings.apply;
  if (typeof apply !== "function") return Promise.resolve();

  const root = oxThemeTransitionRoot();
  if (!oxThemeTransitionSupported(root) || oxThemeTransitionReducedMotion()) {
    apply();
    return Promise.resolve();
  }

  // A second toggle while one is still running would capture a half-animated
  // snapshot. Settle the running one first; its cleanup is synchronous.
  if (oxThemeTransitionCurrent) {
    const running = oxThemeTransitionCurrent;
    oxThemeTransitionCurrent = null;
    if (typeof running.skipTransition === "function") running.skipTransition();
  }

  // Going dark expands the incoming snapshot over the outgoing one; going
  // light shrinks the outgoing snapshot away to reveal the incoming one
  // underneath. Either way the circle grows out of, or collapses into, the
  // point the reader activated.
  const expanding = settings.nextTheme !== "light";
  const origin = oxThemeTransitionOrigin(settings.event);
  const radius = oxThemeTransitionRadius(origin);
  const duration = typeof settings.duration === "number" ? settings.duration : 420;
  const easing = typeof settings.easing === "string" ? settings.easing : "ease-in-out";

  root.setAttribute(OX_THEME_TRANSITION_ATTR, expanding ? "expand" : "shrink");

  let restore = () => {};
  const transition = document.startViewTransition(() => {
    restore = oxThemeTransitionSuppress(root);
    apply();
  });
  oxThemeTransitionCurrent = transition;

  const clipFrom = `circle(0px at ${origin.x}px ${origin.y}px)`;
  const clipTo = `circle(${radius}px at ${origin.x}px ${origin.y}px)`;

  transition.ready.then(
    () => {
      restore();
      root.animate(
        { clipPath: expanding ? [clipFrom, clipTo] : [clipTo, clipFrom] },
        {
          duration,
          easing,
          pseudoElement: expanding ? "::view-transition-new(root)" : "::view-transition-old(root)",
        },
      );
    },
    () => {
      // A skipped or unsupported transition still has to hand the suppressed
      // styles back.
      restore();
    },
  );

  const settled = () => {
    restore();
    // A rapid second toggle skips this one, which lands here *after* the newer
    // transition has already claimed the attribute. Only the transition still
    // holding the slot is allowed to clear it.
    if (oxThemeTransitionCurrent !== transition) return;
    oxThemeTransitionCurrent = null;
    root.removeAttribute(OX_THEME_TRANSITION_ATTR);
  };
  // `finished` rejects when a transition is skipped, which is a normal outcome
  // here — both arms clean up and neither leaves an unhandled rejection.
  return transition.finished.then(settled, settled);
}
