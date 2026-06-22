import { convertFileSrc } from "@tauri-apps/api/core";
import type { ReceivedAssetGroup } from "./appTypes";
import {
  THUMBNAIL_WARMUP_DELAY_MS,
  VIEWER_PREVIEW_MAX_EDGE,
} from "./appState";
import { originalImageWarmCache } from "./previewCache";
import {
  localPreviewablePath,
  previewAssetForGroup,
  shouldRequestOriginalPreviewAsset,
  supportsBrowserOriginalAsset,
  supportsFullThumbnailAsset,
} from "./previewAssets";
import {
  originalPreviewUrlForPath,
  thumbnailUrlForPath,
  warmThumbnailBatch,
} from "./previewQueue";

export type PreviewWarmupOptions = {
  currentThumbnailMaxEdge: () => number;
  previewLocalPathForGroup: (group: ReceivedAssetGroup) => string | null;
};

export function createPreviewWarmup(options: PreviewWarmupOptions) {
  function warmThumbnailsForGroups(groups: ReceivedAssetGroup[]) {
    if (!groups.length) {
      return;
    }
    window.setTimeout(() => {
      const localPaths: string[] = [];
      for (const group of groups) {
        const localPath = options.previewLocalPathForGroup(group);
        if (localPath) {
          localPaths.push(localPath);
        }
      }
      void warmThumbnailBatch(localPaths, options.currentThumbnailMaxEdge(), "fast");
    }, THUMBNAIL_WARMUP_DELAY_MS);
  }

  function warmOriginalsForGroups(groups: ReceivedAssetGroup[]) {
    if (!groups.length) {
      return;
    }
    window.setTimeout(() => {
      for (const group of groups) {
        const asset = previewAssetForGroup(group);
        const localPath = asset ? localPreviewablePath(asset) : null;
        if (!asset || !localPath) {
          continue;
        }
        if (supportsBrowserOriginalAsset(asset)) {
          const url = convertFileSrc(localPath);
          if (originalImageWarmCache.has(url)) {
            continue;
          }
          originalImageWarmCache.add(url);
          const image = new Image();
          image.decoding = "async";
          image.src = url;
        } else if (shouldRequestOriginalPreviewAsset(asset)) {
          void originalPreviewUrlForPath(localPath, "upgrade");
        } else if (supportsFullThumbnailAsset(asset)) {
          void thumbnailUrlForPath(localPath, VIEWER_PREVIEW_MAX_EDGE, "upgrade", "full");
        }
      }
    }, 80);
  }

  return {
    warmThumbnailsForGroups,
    warmOriginalsForGroups,
  };
}
