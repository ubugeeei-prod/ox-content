import { describe, expect, it } from "vite-plus/test";
import { generateI18nModule, resolveI18nOptions } from "./i18n";

function importVirtualI18n() {
  const options = resolveI18nOptions({
    enabled: true,
    defaultLocale: "en-US",
    locales: [
      { code: "en-US", name: "English" },
      { code: "ja-JP", name: "日本語" },
      { code: "ar", name: "العربية", dir: "rtl" },
    ],
    check: false,
  });
  if (!options) throw new Error("expected resolved i18n options");
  const code = generateI18nModule(options, "/__missing__");
  return import(`data:text/javascript;charset=utf-8,${encodeURIComponent(code)}`);
}

describe("virtual i18n Intl helpers", () => {
  it("formats values and exposes locale-scoped helpers", async () => {
    const mod = await importVirtualI18n();
    const intl = mod.createIntl("en-US", { number: { useGrouping: false } });

    expect(mod.formatDate(Date.UTC(2025, 0, 2), { dateStyle: "medium", timeZone: "UTC" })).toBe(
      "Jan 2, 2025",
    );
    expect(mod.formatNumber(1234.5, undefined, "ja-JP")).toBe("1,234.5");
    expect(mod.formatRelativeTime(-1, "day", { numeric: "auto" })).toBe("yesterday");
    expect(mod.formatList(["docs", "api", "cli"], undefined, "en-US")).toBe("docs, api, and cli");
    expect(mod.formatDisplayName("ja", "language", undefined, "en-US")).toBe("Japanese");
    expect(mod.getLocaleMeta("ar")).toMatchObject({ code: "ar", dir: "rtl" });
    expect(mod.createIntl("ar").dir).toBe("rtl");
    expect(intl.number(1234)).toBe("1234");
    expect(intl.number(1234, { useGrouping: true })).toBe("1,234");
    expect(mod.getLocaleFromPath("/ja-JP/guide")).toBe("ja-JP");
    expect(mod.localePath("/ja-JP/guide", "en-US")).toBe("/guide");
  });
});
