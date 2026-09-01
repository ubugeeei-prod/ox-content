(() => {
  const DATA = JSON.parse(document.getElementById("data").textContent);
  const state = { skin: 0, scheme: 1, mode: "dark", page: "landing" };

  const el = (tag, props = {}, kids = []) => {
    const node = Object.assign(document.createElement(tag), props);
    for (const kid of kids) node.append(kid);
    return node;
  };

  function swatchStrip(colors) {
    const strip = el("span", { className: "swatches" });
    for (const key of ["bg", "primary", "green", "yellow", "magenta"]) {
      strip.append(el("i", { style: `background:${colors[key]}` }));
    }
    return strip;
  }

  function optionList(items, group, render) {
    const list = el("div", { className: "list" });
    items.forEach((item, index) => {
      const button = el("button", { className: "opt", type: "button", title: item.description });
      render(button, item);
      button.addEventListener("click", () => {
        state[group] = index;
        paint();
      });
      list.append(button);
    });
    return list;
  }

  const panel = el("aside", { className: "panel" });
  panel.append(
    el("h1", { className: "brand", textContent: "Theme Gallery" }),
    el("p", {
      className: "brand-sub",
      textContent: `${DATA.skins.length} skins × ${DATA.schemes.length} color schemes = ${DATA.skins.length * DATA.schemes.length} combinations`,
    }),
    el("div", { className: "group-title", textContent: "Skin — @ox-content/theme-*" }),
  );

  const skinList = optionList(DATA.skins, "skin", (button, item) => {
    button.append(el("span", { textContent: item.title }));
  });
  panel.append(
    skinList,
    el("div", { className: "group-title", textContent: "Color — @ox-content/theme-color-*" }),
  );

  const schemeList = optionList(DATA.schemes, "scheme", (button, item) => {
    button.append(swatchStrip(item.dark), el("span", { textContent: item.title }));
  });
  panel.append(schemeList);

  const modeSeg = el("div", { className: "seg" });
  const pageSeg = el("div", { className: "seg" });
  const recipe = el("code", { className: "recipe" });

  for (const [label, value] of [
    ["Light", "light"],
    ["Dark", "dark"],
  ]) {
    const button = el("button", { type: "button", textContent: label });
    button.addEventListener("click", () => {
      state.mode = value;
      paint();
    });
    modeSeg.append(button);
  }

  for (const [label, value] of [
    ["Landing", "landing"],
    ["Docs page", "article"],
  ]) {
    const button = el("button", { type: "button", textContent: label });
    button.addEventListener("click", () => {
      state.page = value;
      paint();
    });
    pageSeg.append(button);
  }

  const frame = el("iframe", { title: "Theme preview", loading: "lazy" });
  const meta = el("p", { className: "meta" });
  const stage = el("div", { className: "stage" }, [
    el("div", { className: "controls" }, [
      pageSeg,
      modeSeg,
      el("span", { className: "spacer" }),
      recipe,
    ]),
    el("div", { className: "frame-wrap" }, [frame]),
  ]);
  stage.append(meta);
  document.getElementById("app").append(panel, stage);

  // The preview's own header toggle should work, the way it does on a built
  // site — otherwise the control looks broken. This mirrors what ssg.js does,
  // and reports the new mode back so the gallery's segmented control stays in
  // sync without re-rendering (and re-flashing) the frame.
  const TOGGLE_SCRIPT = `
    for (const el of document.querySelectorAll(".theme-toggle")) {
      el.addEventListener("click", () => {
        const root = document.documentElement;
        const next = root.getAttribute("data-theme") === "dark" ? "light" : "dark";
        root.setAttribute("data-theme", next);
        try { parent.postMessage({ octcMode: next }, "*"); } catch {}
      });
    }`;

  // Assembled rather than written literally: a formatter will happily strip the
  // backslash out of `<\/script>`, and the resulting literal tag closes the
  // script this template lives in.
  const END = "</" + "script>";

  function srcdoc(skin, scheme) {
    // The preview runs in its own document so the base stylesheet's `body`,
    // `:root` and fixed-position rules behave exactly as on a built page.
    return `<!doctype html><html lang="en" data-theme="${state.mode}"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
${skin.head || ""}<style>${DATA.base}</style><style>${scheme.css}</style><style>${skin.css}</style>
<style>.layout{padding-top:var(--octc-header-height)}.hero{min-height:calc(100vh - var(--octc-header-height))}</style>
</head><body class="${state.page === "landing" ? "entry-page" : ""}">${DATA[state.page]}<script>${TOGGLE_SCRIPT}${END}${skin.js ? `<script>${skin.js}${END}` : ""}</body></html>`;
  }

  window.addEventListener("message", (event) => {
    const mode = event.data && event.data.octcMode;
    if (mode !== "light" && mode !== "dark") return;
    state.mode = mode;
    paintChrome();
  });

  /** Updates the gallery's own controls without touching the iframe. */
  function paintChrome() {
    const skin = DATA.skins[state.skin];
    const scheme = DATA.schemes[state.scheme];

    [...skinList.children].forEach((node, index) =>
      node.setAttribute("aria-pressed", String(index === state.skin)),
    );
    [...schemeList.children].forEach((node, index) =>
      node.setAttribute("aria-pressed", String(index === state.scheme)),
    );
    [...modeSeg.children].forEach((node) =>
      node.setAttribute("aria-pressed", String(node.textContent.toLowerCase() === state.mode)),
    );
    [...pageSeg.children].forEach((node, index) =>
      node.setAttribute(
        "aria-pressed",
        String((index === 0 ? "landing" : "article") === state.page),
      ),
    );

    const camel = (id) => id.replace(/-([a-z0-9])/g, (_, c) => c.toUpperCase());
    recipe.textContent = `theme: [${camel(skin.id)}, ${camel(scheme.id)}]`;
    meta.textContent = `${skin.title} — ${skin.description}. ${scheme.title} — ${scheme.description}.`;
    document.documentElement.setAttribute("data-theme", state.mode);
  }

  function paint() {
    paintChrome();
    frame.srcdoc = srcdoc(DATA.skins[state.skin], DATA.schemes[state.scheme]);
  }

  paint();
})();
