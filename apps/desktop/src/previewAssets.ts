import type { ReceivedAsset, ReceivedAssetGroup, StoredObjectLocation } from "./appTypes";
import {
  isBrowserPreviewFormat,
  isPreviewableFormat,
  shouldRequestFullPreview,
  shouldRequestOriginalPreview,
  supportsFullThumbnailFormat,
} from "./mediaPreview";
import { cssToken } from "./presentation";

export function previewAssetForGroup(group: ReceivedAssetGroup) {
  const candidates = [group.jpeg, group.primary, group.video, group.raw].filter(Boolean) as ReceivedAsset[];
  return candidates.find(isPreviewableAsset) ?? null;
}

export function localPreviewablePath(asset: ReceivedAsset) {
  if (!isPreviewableAsset(asset)) return null;
  return localPathFromLocation(asset.storage_location) ?? absolutePathOrNull(asset.original_path);
}

export function localPathFromLocation(location: StoredObjectLocation | null | undefined): string | null {
  if (!location) return null;
  if (typeof location === "string") return absolutePathOrNull(location);
  if (typeof location !== "object") return null;
  const record = location as Record<string, unknown>;
  const direct = record.path ?? record.local_path ?? record.localPath;
  if (typeof direct === "string") return absolutePathOrNull(direct);
  for (const value of Object.values(record)) {
    if (typeof value === "string") {
      const path = absolutePathOrNull(value);
      if (path) return path;
    }
  }
  return null;
}

export function isPreviewableAsset(asset: ReceivedAsset) {
  const format = cssToken(asset.format || extensionOf(asset.filename));
  return isPreviewableFormat(format);
}

export function supportsFullThumbnailAsset(asset: ReceivedAsset) {
  const format = cssToken(asset.format || extensionOf(asset.filename));
  return supportsFullThumbnailFormat(format);
}

export function supportsBrowserOriginalAsset(asset: ReceivedAsset) {
  const format = cssToken(asset.format || extensionOf(asset.filename));
  return isBrowserPreviewFormat(format);
}

export function shouldRequestFullPreviewAsset(asset: ReceivedAsset, original: boolean) {
  const format = cssToken(asset.format || extensionOf(asset.filename));
  return shouldRequestFullPreview(format, original);
}

export function shouldRequestOriginalPreviewAsset(asset: ReceivedAsset) {
  const format = cssToken(asset.format || extensionOf(asset.filename));
  return shouldRequestOriginalPreview(format);
}

export function extensionOf(path: string | null | undefined) {
  return path?.split(/[./\\]/).filter(Boolean).at(-1) ?? "";
}

export function absolutePathOrNull(path: string | null | undefined) {
  if (!path) return null;
  return /^[a-zA-Z]:[\\/]/.test(path) || path.startsWith("\\\\") || path.startsWith("/") ? path : null;
}
