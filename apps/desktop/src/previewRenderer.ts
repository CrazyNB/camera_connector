import { convertFileSrc } from "@tauri-apps/api/core";

import {
  THUMBNAIL_MAX_EDGE,
  THUMBNAIL_MIN_EDGE,
} from "./appState";
import type { PreviewImageOptions, PreviewImageQuality, ReceivedAssetGroup } from "./appTypes";
import { el } from "./domHelpers";
import {
  localPreviewablePath,
  previewAssetForGroup,
  shouldRequestFullPreviewAsset,
  shouldRequestOriginalPreviewAsset,
  supportsBrowserOriginalAsset,
} from "./previewAssets";
import {
  originalPreviewCacheKey,
  originalPreviewUrlCache,
  previewStageForGroup,
  thumbnailCacheKey,
  thumbnailUrlCache,
} from "./previewCache";
import { originalPreviewUrlForPath, thumbnailUrlForPath } from "./previewQueue";
import { previewBadge, type PreviewStage } from "./previewStatus";

type PreviewRendererDeps = {
  clearViewerCarryover: (node: HTMLElement) => void;
  currentPreviewProgressLabel: () => string;
  getThumbSize: () => number;
  onPreviewImageSettled: (node: HTMLElement) => void;
  requestAnimationFrame: (callback: FrameRequestCallback) => number;
};

export function createPreviewRenderer(deps: PreviewRendererDeps) {
  let previewProgressFrame: number | null = null;

  function previewUrlForGroup(group: ReceivedAssetGroup, maxEdge = currentThumbnailMaxEdge(), original = false) {
    const asset = previewAssetForGroup(group);
    const localPath = asset ? localPreviewablePath(asset) : null;
    if (!asset || !localPath) return "";
    if (original && supportsBrowserOriginalAsset(asset)) {
      return convertFileSrc(localPath);
    }
    if (original && shouldRequestOriginalPreviewAsset(asset)) {
      return originalPreviewUrlCache.get(originalPreviewCacheKey(localPath)) ?? "";
    }
    return (
      thumbnailUrlCache.get(thumbnailCacheKey(localPath, maxEdge, "full")) ??
      thumbnailUrlCache.get(thumbnailCacheKey(localPath, maxEdge, "fast")) ??
      (supportsBrowserOriginalAsset(asset) ? convertFileSrc(localPath) : "")
    );
  }

  function previewLocalPathForGroup(group: ReceivedAssetGroup) {
    const asset = previewAssetForGroup(group);
    return asset ? localPreviewablePath(asset) : null;
  }

  function originalPreviewUrlForGroup(group: ReceivedAssetGroup) {
    const asset = previewAssetForGroup(group);
    const localPath = asset ? localPreviewablePath(asset) : null;
    if (!asset || !localPath) return null;
    if (supportsBrowserOriginalAsset(asset)) {
      return convertFileSrc(localPath);
    }
    if (shouldRequestOriginalPreviewAsset(asset)) {
      return originalPreviewUrlCache.get(originalPreviewCacheKey(localPath)) ?? null;
    }
    return null;
  }

  function setPreviewBackground(
    node: HTMLElement,
    group: ReceivedAssetGroup,
    maxEdge = currentThumbnailMaxEdge(),
    original = false,
  ) {
    const url = previewUrlForGroup(group, maxEdge, original);
    if (!url) {
      node.classList.add("no-preview");
      node.style.backgroundImage = "";
      return;
    }
    node.classList.remove("no-preview");
    node.style.backgroundImage = `url("${url}")`;
  }

  function appendPreviewImage(node: HTMLElement, group: ReceivedAssetGroup, options: PreviewImageOptions = {}) {
    const previewAsset = previewAssetForGroup(group);
    const localPath = previewAsset ? localPreviewablePath(previewAsset) : null;
    if (!previewAsset || !localPath) {
      node.classList.add("no-preview");
      syncPreviewStatusBadge(node, "idle");
      return;
    }
    const shouldUpgradeFull = shouldRequestFullPreviewAsset(previewAsset, Boolean(options.original));
    const maxEdge = options.maxEdge ?? currentThumbnailMaxEdge();
    if (options.original && supportsBrowserOriginalAsset(previewAsset)) {
      node.dataset.previewPath = localPath;
      node.dataset.previewMaxEdge = "original";
      node.classList.remove("no-preview", "is-loading");
      insertPreviewImage(node, convertFileSrc(localPath), "original", options.eager);
      syncPreviewStatusBadge(node, "original");
      refreshPreviewProgressDom();
      return;
    }
    if (options.original && shouldRequestOriginalPreviewAsset(previewAsset)) {
      appendOriginalPreviewImage(node, group, localPath, maxEdge, options.eager);
      return;
    }
    const fullKey = thumbnailCacheKey(localPath, maxEdge, "full");
    const fastKey = thumbnailCacheKey(localPath, maxEdge, "fast");
    node.dataset.previewPath = localPath;
    node.dataset.previewMaxEdge = String(maxEdge);
    node.classList.remove("no-preview");
    const cachedFullUrl = thumbnailUrlCache.get(fullKey);
    if (cachedFullUrl) {
      insertPreviewImage(node, cachedFullUrl, "full", options.eager);
      syncPreviewStatusBadge(node, "full");
      return;
    }
    const cachedFastUrl = thumbnailUrlCache.get(fastKey);
    if (cachedFastUrl) {
      insertPreviewImage(node, cachedFastUrl, "fast", options.eager);
      syncPreviewStatusBadge(node, "fast");
      if (shouldUpgradeFull) scheduleFullQualityUpgrade(node, localPath, maxEdge);
      return;
    }
    node.classList.add("is-loading");
    const request = thumbnailUrlForPath(localPath, maxEdge, "visible", "fast");
    syncPreviewStatusBadge(node, previewStageForGroup(group, maxEdge, Boolean(options.original)));
    void request.then((url) => {
      if (!node.isConnected || node.dataset.previewPath !== localPath || node.dataset.previewMaxEdge !== String(maxEdge)) {
        return;
      }
      node.classList.remove("is-loading");
      if (!url) {
        node.classList.add("no-preview");
        syncPreviewStatusBadge(node, "failed");
        return;
      }
      node.classList.remove("no-preview");
      insertPreviewImage(node, url, "fast", options.eager);
      syncPreviewStatusBadge(node, "fast");
      refreshPreviewProgressDom();
      if (shouldUpgradeFull) scheduleFullQualityUpgrade(node, localPath, maxEdge);
    });
  }

  function appendOriginalPreviewImage(
    node: HTMLElement,
    group: ReceivedAssetGroup,
    localPath: string,
    fallbackMaxEdge: number,
    eager = false,
  ) {
    const key = originalPreviewCacheKey(localPath);
    node.dataset.previewPath = localPath;
    node.dataset.previewMaxEdge = "original";
    node.classList.remove("no-preview");
    const cachedOriginal = originalPreviewUrlCache.get(key);
    if (cachedOriginal) {
      node.classList.remove("is-loading");
      insertPreviewImage(node, cachedOriginal, "original", eager);
      syncPreviewStatusBadge(node, "original");
      refreshPreviewProgressDom();
      return;
    }

    const fallback =
      thumbnailUrlCache.get(thumbnailCacheKey(localPath, fallbackMaxEdge, "full")) ??
      thumbnailUrlCache.get(thumbnailCacheKey(localPath, fallbackMaxEdge, "fast"));
    if (fallback) {
      insertPreviewImage(node, fallback, "full", eager);
    } else {
      node.classList.add("is-loading");
      void thumbnailUrlForPath(localPath, fallbackMaxEdge, "visible", "fast").then((url) => {
        if (!node.isConnected || node.dataset.previewPath !== localPath || node.dataset.previewMaxEdge !== "original") {
          return;
        }
        node.classList.remove("is-loading");
        if (url && !originalPreviewUrlCache.has(key)) {
          insertPreviewImage(node, url, "fast", eager);
        }
      });
    }

    syncPreviewStatusBadge(node, previewStageForGroup(group, fallbackMaxEdge, true));
    void originalPreviewUrlForPath(localPath, "visible").then((url) => {
      if (!node.isConnected || node.dataset.previewPath !== localPath || node.dataset.previewMaxEdge !== "original") {
        return;
      }
      node.classList.remove("is-loading");
      if (!url) {
        node.classList.add("no-preview");
        syncPreviewStatusBadge(node, "failed");
        return;
      }
      node.classList.remove("no-preview");
      insertPreviewImage(node, url, "original", eager);
      syncPreviewStatusBadge(node, "original");
      refreshPreviewProgressDom();
    });
  }

  function scheduleFullQualityUpgrade(node: HTMLElement, localPath: string, maxEdge: number) {
    const key = thumbnailCacheKey(localPath, maxEdge, "full");
    const cachedUrl = thumbnailUrlCache.get(key);
    if (cachedUrl) {
      insertPreviewImage(node, cachedUrl, "full");
      syncPreviewStatusBadge(node, "full");
      return;
    }
    if (node.dataset.previewFullPending === key) {
      return;
    }
    node.dataset.previewFullPending = key;
    void thumbnailUrlForPath(localPath, maxEdge, "upgrade", "full").then((url) => {
      if (node.dataset.previewFullPending === key) {
        delete node.dataset.previewFullPending;
      }
      if (
        !node.isConnected ||
        !url ||
        node.dataset.previewPath !== localPath ||
        node.dataset.previewMaxEdge !== String(maxEdge)
      ) {
        return;
      }
      insertPreviewImage(node, url, "full");
      syncPreviewStatusBadge(node, "full");
      refreshPreviewProgressDom();
    });
  }

  function insertPreviewImage(node: HTMLElement, url: string, quality: PreviewImageQuality, eager = false) {
    const current = node.querySelector<HTMLImageElement>(":scope > img.preview-image");
    if (current?.src === url && current.dataset.quality === quality) {
      return;
    }
    const image = el("img", "preview-image") as HTMLImageElement;
    image.src = url;
    image.alt = "";
    image.loading = eager ? "eager" : "lazy";
    image.decoding = "async";
    image.draggable = false;
    image.dataset.quality = quality;
    image.setAttribute("fetchpriority", eager ? "high" : "low");
    const settle = () => {
      node.querySelectorAll<HTMLImageElement>(":scope > img.preview-image").forEach((candidate) => {
        if (candidate !== image) {
          candidate.remove();
        }
      });
      deps.onPreviewImageSettled(node);
      if (node.classList.contains("viewer-main-preview")) {
        deps.clearViewerCarryover(node);
      }
    };
    image.addEventListener("load", settle, { once: true });
    image.addEventListener("error", () => {
      image.remove();
      if (node.classList.contains("viewer-main-preview")) {
        deps.clearViewerCarryover(node);
      }
    }, { once: true });
    node.prepend(image);
    if (image.complete) {
      settle();
    }
  }

  function currentThumbnailMaxEdge() {
    const pixelRatio = Math.min(Math.max(window.devicePixelRatio || 1, 1), 2.25);
    const edge = Math.ceil(deps.getThumbSize() * pixelRatio);
    return Math.min(THUMBNAIL_MAX_EDGE, Math.max(THUMBNAIL_MIN_EDGE, edge));
  }

  function renderPreviewStatusBadge(
    group: ReceivedAssetGroup,
    maxEdge = currentThumbnailMaxEdge(),
    original = false,
  ) {
    const badge = el("span", "preview-status-badge");
    badge.dataset.previewStatusBadge = "true";
    applyPreviewStatusBadge(badge, previewStageForGroup(group, maxEdge, original));
    return badge;
  }

  function previewTooltipForGroup(
    group: ReceivedAssetGroup,
    maxEdge = currentThumbnailMaxEdge(),
    original = false,
  ) {
    const display = previewBadge(previewStageForGroup(group, maxEdge, original));
    return `${group.group_key} 路 ${display.label}`;
  }

  function applyPreviewStatusBadge(badge: HTMLElement, stage: PreviewStage) {
    const display = previewBadge(stage);
    badge.className = `preview-status-badge ${display.tone}`;
    badge.textContent = display.label;
    badge.title = display.title;
    badge.dataset.previewStage = stage;
  }

  function syncPreviewStatusBadge(node: HTMLElement, stage: PreviewStage) {
    const badge = node.querySelector<HTMLElement>(":scope > .preview-status-badge");
    if (badge) {
      applyPreviewStatusBadge(badge, stage);
    }
  }

  function refreshPreviewProgressDom() {
    if (previewProgressFrame !== null) {
      return;
    }
    previewProgressFrame = deps.requestAnimationFrame(() => {
      previewProgressFrame = null;
      const label = deps.currentPreviewProgressLabel();
      document.querySelectorAll<HTMLElement>("[data-preview-progress='true']").forEach((node) => {
        node.textContent = label;
      });
    });
  }

  return {
    appendPreviewImage,
    currentThumbnailMaxEdge,
    insertPreviewImage,
    originalPreviewUrlForGroup,
    previewLocalPathForGroup,
    previewTooltipForGroup,
    previewUrlForGroup,
    refreshPreviewProgressDom,
    renderPreviewStatusBadge,
    setPreviewBackground,
    syncPreviewStatusBadge,
  };
}
