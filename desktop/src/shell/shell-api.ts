import { invoke } from "@tauri-apps/api/core";

const ICON_MEDIA_TYPE = "image/png";

const iconUrls = new Map<string, Promise<string | null>>();

export function iconUrlFor(path: string): Promise<string | null> {
  const knownIconUrl = iconUrls.get(path);
  if (knownIconUrl !== undefined) {
    return knownIconUrl;
  }
  const loadingIconUrl = invoke<ArrayBuffer | number[]>("read_icon", { path }).then(
    (png) => URL.createObjectURL(new Blob([bytesOf(png)], { type: ICON_MEDIA_TYPE })),
    () => null,
  );
  iconUrls.set(path, loadingIconUrl);
  return loadingIconUrl;
}

function bytesOf(png: ArrayBuffer | number[]): ArrayBuffer | Uint8Array<ArrayBuffer> {
  return png instanceof ArrayBuffer ? png : new Uint8Array(png);
}

export function measureSizes(paths: readonly string[]): Promise<number[]> {
  return invoke<number[]>("measure_sizes", { paths });
}

export function showInFileManager(path: string): Promise<void> {
  return invoke<void>("show_in_file_manager", { path });
}

export function hasMica(): Promise<boolean> {
  return invoke<boolean>("has_mica").catch(() => false);
}
