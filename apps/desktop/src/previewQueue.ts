import { convertFileSrc } from "@tauri-apps/api/core";

import { fullThumbnailConcurrency } from "./thumbnailScheduler";
import { shouldApplyPreviewSync, type PreviewSyncQuality } from "./previewSync";
import {
  ORIGINAL_PREVIEW_CONCURRENCY,
  THUMBNAIL_BATCH_SIZE,
  THUMBNAIL_CONCURRENCY,
  THUMBNAIL_MAX_EDGE,
} from "./appState";
import type { ThumbnailQuality } from "./appTypes";
import { api } from "./desktopApi";
import {
  originalPreviewActiveKeys,
  originalPreviewCacheKey,
  originalPreviewFailedKeys,
  originalPreviewPending,
  originalPreviewQueue,
  originalPreviewQueued,
  originalPreviewStageForLocalPath,
  originalPreviewUrlCache,
  previewStageForLocalPath,
  thumbnailActiveKeys,
  thumbnailBatchPending,
  thumbnailCacheKey,
  thumbnailFailedKeys,
  thumbnailPending,
  thumbnailQueue,
  thumbnailQueued,
  thumbnailUrlCache,
  type OriginalPreviewQueueItem,
  type ThumbnailPriority,
  type ThumbnailQueueItem,
} from "./previewCache";
import type { PreviewStage } from "./previewStatus";

type CachedPreviewNode = {
  localPath: string;
  maxEdge: string;
  quality: PreviewSyncQuality;
  url: string | null;
};

type PreviewQueueCallbacks = {
  insertPreviewImage: (node: HTMLElement, url: string, quality: PreviewSyncQuality) => void;
  refreshPreviewProgressDom: () => void;
  syncPreviewStatusBadge: (node: HTMLElement, stage: PreviewStage) => void;
  getThumbnailScrolling: () => boolean;
};

let thumbnailActiveCount = 0;
let thumbnailFullActiveCount = 0;
let originalPreviewActiveCount = 0;

let callbacks: PreviewQueueCallbacks | null = null;

export function configurePreviewQueue(nextCallbacks: PreviewQueueCallbacks) {
  callbacks = nextCallbacks;
}

export async function thumbnailUrlForPath(
  localPath: string,
  maxEdge = THUMBNAIL_MAX_EDGE,
  priority: ThumbnailPriority = "visible",
  quality: ThumbnailQuality = "fast",
) {
  const key = thumbnailCacheKey(localPath, maxEdge, quality);
  const cached = thumbnailUrlCache.get(key);
  if (cached) return cached;
  const pending = thumbnailPending.get(key);
  if (pending) {
    if (priority === "visible") {
      promoteQueuedThumbnail(key);
    }
    return pending;
  }
  const request = enqueueThumbnailRequest(key, localPath, maxEdge, priority, quality).finally(() => {
    thumbnailPending.delete(key);
  });
  thumbnailPending.set(key, request);
  return request;
}

export async function originalPreviewUrlForPath(localPath: string, priority: ThumbnailPriority = "visible") {
  const key = originalPreviewCacheKey(localPath);
  const cached = originalPreviewUrlCache.get(key);
  if (cached) return cached;
  const pending = originalPreviewPending.get(key);
  if (pending) {
    if (priority === "visible") {
      promoteQueuedOriginalPreview(key);
    }
    return pending;
  }
  const request = enqueueOriginalPreviewRequest(key, localPath, priority).finally(() => {
    originalPreviewPending.delete(key);
  });
  originalPreviewPending.set(key, request);
  return request;
}

export async function warmThumbnailBatch(localPaths: string[], maxEdge: number, quality: ThumbnailQuality) {
  const batch: Array<{ key: string; localPath: string }> = [];
  const seen = new Set<string>();
  for (const localPath of localPaths) {
    const key = thumbnailCacheKey(localPath, maxEdge, quality);
    if (
      seen.has(key) ||
      thumbnailUrlCache.has(key) ||
      thumbnailPending.has(key) ||
      thumbnailQueued.has(key) ||
      thumbnailBatchPending.has(key)
    ) {
      continue;
    }
    seen.add(key);
    batch.push({ key, localPath });
  }

  for (let index = 0; index < batch.length; index += THUMBNAIL_BATCH_SIZE) {
    const chunk = batch.slice(index, index + THUMBNAIL_BATCH_SIZE);
    for (const item of chunk) {
      thumbnailBatchPending.add(item.key);
    }
    refreshPreviewProgressDom();
    try {
      const response = await api.getAssetThumbnails(
        chunk.map((item) => item.localPath),
        maxEdge,
        quality,
      );
      for (const item of response.thumbnails) {
        if (!item.path) {
          thumbnailFailedKeys.add(thumbnailCacheKey(item.source_path, maxEdge, quality));
          continue;
        }
        const key = thumbnailCacheKey(item.source_path, maxEdge, quality);
        thumbnailFailedKeys.delete(key);
        const url = convertFileSrc(item.path);
        thumbnailUrlCache.set(key, url);
        syncPreviewNodesForCachedThumbnail(item.source_path, maxEdge, quality, url);
      }
    } catch {
      // Visible thumbnails still use the priority queue; background warmup can fail quietly.
    } finally {
      for (const item of chunk) {
        thumbnailBatchPending.delete(item.key);
      }
      refreshPreviewProgressDom();
    }
  }
}

export function pumpThumbnailQueue() {
  while (thumbnailQueue.length) {
    const index = thumbnailQueue.findIndex(canStartThumbnailItem);
    if (index < 0) {
      return;
    }
    const [item] = thumbnailQueue.splice(index, 1);
    thumbnailQueued.delete(item.key);
    if (thumbnailUrlCache.has(item.key)) {
      item.resolve(thumbnailUrlCache.get(item.key) ?? null);
      syncPreviewNodesForThumbnailItem(item);
      refreshPreviewProgressDom();
      continue;
    }
    startThumbnailItem(item);
    void api
      .getAssetThumbnail(item.localPath, item.maxEdge, item.quality)
      .then((response) => {
        const url = convertFileSrc(response.path);
        thumbnailFailedKeys.delete(item.key);
        thumbnailUrlCache.set(item.key, url);
        item.resolve(url);
      })
      .catch(() => {
        thumbnailFailedKeys.add(item.key);
        item.resolve(null);
      })
      .finally(() => {
        finishThumbnailItem(item);
        pumpThumbnailQueue();
      });
  }
}

function enqueueOriginalPreviewRequest(key: string, localPath: string, priority: ThumbnailPriority) {
  const request = new Promise<string | null>((resolve) => {
    const item: OriginalPreviewQueueItem = { key, localPath, priority, resolve };
    originalPreviewFailedKeys.delete(key);
    originalPreviewQueued.set(key, item);
    if (priority === "visible") {
      originalPreviewQueue.unshift(item);
    } else if (priority === "upgrade") {
      const firstPrefetchIndex = originalPreviewQueue.findIndex((candidate) => candidate.priority === "prefetch");
      if (firstPrefetchIndex >= 0) {
        originalPreviewQueue.splice(firstPrefetchIndex, 0, item);
      } else {
        originalPreviewQueue.push(item);
      }
    } else {
      originalPreviewQueue.push(item);
    }
    pumpOriginalPreviewQueue();
    refreshPreviewProgressDom();
  });
  return request;
}

function promoteQueuedOriginalPreview(key: string) {
  const item = originalPreviewQueued.get(key);
  if (!item) {
    return;
  }
  const index = originalPreviewQueue.indexOf(item);
  if (index <= 0) {
    return;
  }
  originalPreviewQueue.splice(index, 1);
  originalPreviewQueue.unshift(item);
}

function pumpOriginalPreviewQueue() {
  while (originalPreviewQueue.length && originalPreviewActiveCount < ORIGINAL_PREVIEW_CONCURRENCY) {
    const [item] = originalPreviewQueue.splice(0, 1);
    originalPreviewQueued.delete(item.key);
    if (originalPreviewUrlCache.has(item.key)) {
      item.resolve(originalPreviewUrlCache.get(item.key) ?? null);
      syncPreviewNodesForOriginalItem(item);
      refreshPreviewProgressDom();
      continue;
    }
    startOriginalPreviewItem(item);
    void api
      .getAssetOriginalPreview(item.localPath)
      .then((response) => {
        const url = convertFileSrc(response.path);
        originalPreviewFailedKeys.delete(item.key);
        originalPreviewUrlCache.set(item.key, url);
        item.resolve(url);
      })
      .catch(() => {
        originalPreviewFailedKeys.add(item.key);
        item.resolve(null);
      })
      .finally(() => {
        finishOriginalPreviewItem(item);
        pumpOriginalPreviewQueue();
      });
  }
}

function startOriginalPreviewItem(item: OriginalPreviewQueueItem) {
  originalPreviewActiveKeys.add(item.key);
  originalPreviewActiveCount += 1;
  syncPreviewNodesForOriginalItem(item);
  refreshPreviewProgressDom();
}

function finishOriginalPreviewItem(item: OriginalPreviewQueueItem) {
  originalPreviewActiveKeys.delete(item.key);
  originalPreviewActiveCount = Math.max(0, originalPreviewActiveCount - 1);
  syncPreviewNodesForOriginalItem(item);
  refreshPreviewProgressDom();
}

function enqueueThumbnailRequest(
  key: string,
  localPath: string,
  maxEdge: number,
  priority: ThumbnailPriority,
  quality: ThumbnailQuality,
) {
  const request = new Promise<string | null>((resolve) => {
    const item: ThumbnailQueueItem = { key, localPath, maxEdge, quality, priority, resolve };
    thumbnailFailedKeys.delete(key);
    thumbnailQueued.set(key, item);
    if (priority === "visible") {
      thumbnailQueue.unshift(item);
    } else if (priority === "upgrade") {
      const firstPrefetchIndex = thumbnailQueue.findIndex((candidate) => candidate.priority === "prefetch");
      if (firstPrefetchIndex >= 0) {
        thumbnailQueue.splice(firstPrefetchIndex, 0, item);
      } else {
        thumbnailQueue.push(item);
      }
    } else {
      thumbnailQueue.push(item);
    }
    pumpThumbnailQueue();
    refreshPreviewProgressDom();
  });
  return request;
}

function syncPreviewNodesForThumbnailItem(item: ThumbnailQueueItem) {
  document.querySelectorAll<HTMLElement>("[data-preview-path]").forEach((node) => {
    if (node.dataset.previewPath !== item.localPath || node.dataset.previewMaxEdge !== String(item.maxEdge)) {
      return;
    }
    const url = thumbnailUrlCache.get(item.key) ?? null;
    applyCachedPreviewToNode(node, {
      localPath: item.localPath,
      maxEdge: String(item.maxEdge),
      quality: item.quality,
      url,
    });
    syncPreviewStatusBadge(node, previewStageForLocalPath(item.localPath, item.maxEdge));
  });
}

function syncPreviewNodesForOriginalItem(item: OriginalPreviewQueueItem) {
  document.querySelectorAll<HTMLElement>("[data-preview-path]").forEach((node) => {
    if (node.dataset.previewPath !== item.localPath || node.dataset.previewMaxEdge !== "original") {
      return;
    }
    const url = originalPreviewUrlCache.get(item.key) ?? null;
    applyCachedPreviewToNode(node, {
      localPath: item.localPath,
      maxEdge: "original",
      quality: "original",
      url,
    });
    syncPreviewStatusBadge(node, originalPreviewStageForLocalPath(item.localPath));
  });
}

function syncPreviewNodesForCachedThumbnail(
  localPath: string,
  maxEdge: number,
  quality: ThumbnailQuality,
  url: string | null,
) {
  document.querySelectorAll<HTMLElement>("[data-preview-path]").forEach((node) => {
    if (node.dataset.previewPath !== localPath || node.dataset.previewMaxEdge !== String(maxEdge)) {
      return;
    }
    applyCachedPreviewToNode(node, {
      localPath,
      maxEdge: String(maxEdge),
      quality,
      url,
    });
    syncPreviewStatusBadge(node, previewStageForLocalPath(localPath, maxEdge));
  });
}

function applyCachedPreviewToNode(node: HTMLElement, item: CachedPreviewNode) {
  const current = node.querySelector<HTMLImageElement>(":scope > img.preview-image");
  if (
    !shouldApplyPreviewSync(
      {
        previewPath: node.dataset.previewPath,
        previewMaxEdge: node.dataset.previewMaxEdge,
        previewFullPending: node.dataset.previewFullPending,
        currentQuality: previewQualityFromImage(current),
      },
      item,
    )
  ) {
    return;
  }
  if (!item.url) {
    return;
  }
  node.classList.remove("no-preview", "is-loading");
  insertPreviewImage(node, item.url, item.quality);
}

function previewQualityFromImage(image: HTMLImageElement | null): PreviewSyncQuality | undefined {
  const quality = image?.dataset.quality;
  if (quality === "fast" || quality === "full" || quality === "original") {
    return quality;
  }
  return undefined;
}

function promoteQueuedThumbnail(key: string) {
  const item = thumbnailQueued.get(key);
  if (!item) {
    return;
  }
  const index = thumbnailQueue.indexOf(item);
  if (index <= 0) {
    return;
  }
  thumbnailQueue.splice(index, 1);
  thumbnailQueue.unshift(item);
}

function canStartThumbnailItem(item: ThumbnailQueueItem) {
  if (item.quality === "full") {
    return thumbnailFullActiveCount < fullThumbnailConcurrency(getThumbnailScrolling());
  }
  return thumbnailActiveCount < THUMBNAIL_CONCURRENCY;
}

function startThumbnailItem(item: ThumbnailQueueItem) {
  thumbnailActiveKeys.add(item.key);
  if (item.quality === "full") {
    thumbnailFullActiveCount += 1;
  } else {
    thumbnailActiveCount += 1;
  }
  syncPreviewNodesForThumbnailItem(item);
  refreshPreviewProgressDom();
}

function finishThumbnailItem(item: ThumbnailQueueItem) {
  thumbnailActiveKeys.delete(item.key);
  if (item.quality === "full") {
    thumbnailFullActiveCount = Math.max(0, thumbnailFullActiveCount - 1);
  } else {
    thumbnailActiveCount = Math.max(0, thumbnailActiveCount - 1);
  }
  syncPreviewNodesForThumbnailItem(item);
  refreshPreviewProgressDom();
}

function insertPreviewImage(node: HTMLElement, url: string, quality: PreviewSyncQuality) {
  callbacks?.insertPreviewImage(node, url, quality);
}

function refreshPreviewProgressDom() {
  callbacks?.refreshPreviewProgressDom();
}

function syncPreviewStatusBadge(node: HTMLElement, stage: PreviewStage) {
  callbacks?.syncPreviewStatusBadge(node, stage);
}

function getThumbnailScrolling() {
  return callbacks?.getThumbnailScrolling() ?? false;
}
