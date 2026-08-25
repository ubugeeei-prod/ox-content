import { describe, expect, it } from "vite-plus/test";
import { resolveFileTreeOptions, toJsFileTreeOptions } from "./file-tree-options";

describe("fileTree option resolution", () => {
  it("treats omitted as disabled, and true or {} as enabled", () => {
    expect(resolveFileTreeOptions(undefined)).toEqual({
      enabled: false,
      defaultOpen: true,
      icons: true,
    });
    expect(resolveFileTreeOptions(false).enabled).toBe(false);
    expect(resolveFileTreeOptions(true)).toEqual({
      enabled: true,
      defaultOpen: true,
      icons: true,
    });
    expect(resolveFileTreeOptions({}).enabled).toBe(true);
    expect(resolveFileTreeOptions({ enabled: false }).enabled).toBe(false);
  });

  it("flattens replaceable icons and collapse defaults", () => {
    expect(
      resolveFileTreeOptions({
        defaultOpen: false,
        icons: {
          folder: "<svg class='folder'></svg>",
          folderOpen: "<svg class='open'></svg>",
          file: "codicon-file",
          files: { ts: "<svg class='ts'></svg>" },
        },
      }),
    ).toEqual({
      enabled: true,
      defaultOpen: false,
      icons: true,
      iconFolder: "<svg class='folder'></svg>",
      iconFolderOpen: "<svg class='open'></svg>",
      iconFile: "codicon-file",
      iconFiles: { ts: "<svg class='ts'></svg>" },
    });
    expect(resolveFileTreeOptions({ icons: false }).icons).toBe(false);
  });

  it("omits native options when the transform is off", () => {
    expect(toJsFileTreeOptions(resolveFileTreeOptions(false))).toBeUndefined();
    expect(toJsFileTreeOptions(resolveFileTreeOptions(true))).toEqual({
      enabled: true,
      defaultOpen: true,
      icons: true,
      iconFolder: undefined,
      iconFolderOpen: undefined,
      iconFile: undefined,
      iconFiles: undefined,
    });
  });
});
