import { describe, expect, it } from "vite-plus/test";
import { getPageLocale, resolveSsgOptions } from "./ssg";

describe("resolveSsgOptions", () => {
  it("disables git timestamps by default", () => {
    expect(resolveSsgOptions(undefined).lastUpdated).toBe(false);
  });

  it("enables git timestamps when requested", () => {
    expect(resolveSsgOptions({ lastUpdated: true }).lastUpdated).toBe(true);
  });
});

describe("getPageLocale", () => {
  it("derives BCP 47 locales from localized paths", () => {
    const i18n = {
      enabled: true,
      dir: "content/i18n",
      defaultLocale: "en-US",
      locales: [
        { code: "en-US", name: "English" },
        { code: "zh-Hans-CN", name: "简体中文" },
      ],
      hideDefaultLocale: true,
      check: false,
      functionNames: ["t"],
    };

    expect(getPageLocale("zh-Hans-CN/guide", i18n)).toBe("zh-Hans-CN");
    expect(getPageLocale("guide", i18n)).toBe("en-US");
    expect(getPageLocale("guide", false)).toBeUndefined();
  });
});
