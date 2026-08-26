import { describe, expect, it } from "vite-plus/test";
import { resolveKeyboardKeysOptions } from "./index";

describe("resolveKeyboardKeysOptions", () => {
  it("omitted => false; true => true; {} => true", () => {
    expect(resolveKeyboardKeysOptions(undefined)).toEqual({
      enabled: false,
      aliases: {},
      style: "words",
    });
    expect(resolveKeyboardKeysOptions(true)).toEqual({
      enabled: true,
      aliases: {},
      style: "words",
    });
    expect(resolveKeyboardKeysOptions({})).toEqual({
      enabled: true,
      aliases: {},
      style: "words",
    });
  });

  it("accepts boolean and object forms", () => {
    expect(resolveKeyboardKeysOptions({ enabled: false }).enabled).toBe(false);
    expect(
      resolveKeyboardKeysOptions({
        style: "symbols",
        aliases: { cmd: "Cmd" },
      }),
    ).toEqual({
      enabled: true,
      aliases: { cmd: "Cmd" },
      style: "symbols",
    });
    expect(resolveKeyboardKeysOptions({ style: "words" }).style).toBe("words");
  });
});
