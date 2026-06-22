import type { ReceivedAssetGroup, ThumbnailQuality } from "./appTypes";
import type { PreviewStage } from "./previewStatus";
import {
  localPreviewablePath,
  previewAssetForGroup,
  shouldRequestOriginalPreviewAsset,
  supportsBrowserOriginalAsset,
} from "./previewAssets";
import { THUMBNAIL_MAX_EDGE } from "./appState";

export const thumbnailUrlCache = new Map<string, string>();
export const thumbnailPending = new Map<string, Promise<string | null>>();
export const originalImageWarmCache = new Set<string>();
export const originalPreviewUrlCache = new Map<string, string>();
export const originalPreviewPending = new Map<string, Promise<string | null>>();
export type ThumbnailPriority = "visible" | "upgrade" | "prefetch";
export type ThumbnailQueueItem = {
  key: string;
  localPath: string;
  maxEdge: number;
  quality: ThumbnailQuality;
  priority: ThumbnailPriority;
  resolve: (url: string | null) => void;
};
export type OriginalPreviewQueueItem = {
  key: string;
  localPath: string;
  priority: ThumbnailPriority;
  resolve: (url: string | null) => void;
};
export const thumbnailQueue: ThumbnailQueueItem[] = [];
export const thumbnailQueued = new Map<string, ThumbnailQueueItem>();
export const thumbnailBatchPending = new Set<string>();
export const thumbnailActiveKeys = new Set<string>();
export const thumbnailFailedKeys = new Set<string>();
export const originalPreviewQueue: OriginalPreviewQueueItem[] = [];
export const originalPreviewQueued = new Map<string, OriginalPreviewQueueItem>();
export const originalPreviewActiveKeys = new Set<string>();
export const originalPreviewFailedKeys = new Set<string>();

export function thumbnailCacheKey(localPath: string, maxEdge = THUMBNAIL_MAX_EDGE, quality: ThumbnailQuality = "fast") {
  return `${quality}:${maxEdge}:${localPath}`;
}

export function originalPreviewCacheKey(localPath: string) {
  return `original:${localPath}`;
}

export function previewStageForGroup(group: ReceivedAssetGroup, maxEdge: number, original = false): PreviewStage {
  const asset = previewAssetForGroup(group);
  const localPath = asset ? localPreviewablePath(asset) : null;
  if (!asset || !localPath) {
    return "idle";
  }
  if (original && supportsBrowserOriginalAsset(asset)) {
    return "original";
  }
  if (original && shouldRequestOriginalPreviewAsset(asset)) {
    return originalPreviewStageForLocalPath(localPath);
  }
  return previewStageForLocalPath(localPath, maxEdge);
}

export function originalPreviewStageForLocalPath(localPath: string): PreviewStage {
  const key = originalPreviewCacheKey(localPath);
  if (originalPreviewUrlCache.has(key)) {
    return "original";
  }
  if (originalPreviewActiveKeys.has(key)) {
    return "loading";
  }
  if (originalPreviewPending.has(key) || originalPreviewQueued.has(key)) {
    return "queued";
  }
  if (originalPreviewFailedKeys.has(key)) {
    return "failed";
  }
  return "idle";
}

export function previewStageForLocalPath(localPath: string, maxEdge: number): PreviewStage {
  const fullKey = thumbnailCacheKey(localPath, maxEdge, "full");
  const fastKey = thumbnailCacheKey(localPath, maxEdge, "fast");
  if (thumbnailUrlCache.has(fullKey)) {
    return "full";
  }
  if (thumbnailUrlCache.has(fastKey)) {
    return "fast";
  }
  if (thumbnailActiveKeys.has(fullKey) || thumbnailActiveKeys.has(fastKey)) {
    return "loading";
  }
  if (
    thumbnailPending.has(fullKey) ||
    thumbnailPending.has(fastKey) ||
    thumbnailQueued.has(fullKey) ||
    thumbnailQueued.has(fastKey) ||
    thumbnailBatchPending.has(fullKey) ||
    thumbnailBatchPending.has(fastKey)
  ) {
    return "queued";
  }
  if (thumbnailFailedKeys.has(fullKey) || thumbnailFailedKeys.has(fastKey)) {
    return "failed";
  }
  return "idle";
}
