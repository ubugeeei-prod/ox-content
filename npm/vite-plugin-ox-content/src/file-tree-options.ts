import type { FileTreeIconOptions, OxContentOptions, ResolvedOptions } from "./types";

type JsFileTreeOptions = {
  enabled: true;
  defaultOpen: boolean;
  icons: boolean;
  iconFolder?: string;
  iconFolderOpen?: string;
  iconFile?: string;
  iconFiles?: Record<string, string>;
};

const disabled: ResolvedOptions["fileTree"] = {
  enabled: false,
  defaultOpen: true,
  icons: true,
};

export function resolveFileTreeOptions(
  options: OxContentOptions["fileTree"],
): ResolvedOptions["fileTree"] {
  if (!options) return { ...disabled };
  if (options === true) return enabledDefaults();
  if (options.enabled === false) {
    return { ...enabledDefaults(), enabled: false, ...resolveIcons(options.icons) };
  }
  return {
    enabled: options.enabled ?? true,
    defaultOpen: options.defaultOpen ?? true,
    ...resolveIcons(options.icons),
  };
}

export function toJsFileTreeOptions(
  options: ResolvedOptions["fileTree"] | undefined,
): JsFileTreeOptions | undefined {
  if (!options?.enabled) return undefined;
  return {
    enabled: true,
    defaultOpen: options.defaultOpen,
    icons: options.icons,
    iconFolder: options.iconFolder,
    iconFolderOpen: options.iconFolderOpen,
    iconFile: options.iconFile,
    iconFiles: options.iconFiles,
  };
}

function enabledDefaults(): ResolvedOptions["fileTree"] {
  return { enabled: true, defaultOpen: true, icons: true };
}

function resolveIcons(
  icons: boolean | FileTreeIconOptions | undefined,
): Pick<
  ResolvedOptions["fileTree"],
  "icons" | "iconFolder" | "iconFolderOpen" | "iconFile" | "iconFiles"
> {
  if (icons === false) return { icons: false };
  if (icons === true || icons == null) return { icons: true };
  return {
    icons: true,
    iconFolder: icons.folder,
    iconFolderOpen: icons.folderOpen,
    iconFile: icons.file,
    iconFiles: icons.files,
  };
}
