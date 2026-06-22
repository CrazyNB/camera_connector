import {
  LOUPE_OVERLAY_HEIGHT,
  LOUPE_OVERLAY_MAX_WIDTH,
  LOUPE_SHOW_DELAY_MS,
  LOUPE_ZOOM_READY_MS,
} from "./appState";
import type { LoupeState, ReceivedAssetGroup } from "./appTypes";
import { containedImageRect, coverImageRect, normalizedPointInRect } from "./imageViewport";
import { initialLoupeZoom, nextLoupeZoom } from "./lightTableLayout";
import type { ThumbnailPriority } from "./previewCache";

type LoupeInteractionDeps = {
  applyPreviewBackground: (node: HTMLElement, group: ReceivedAssetGroup, maxEdge: number, original: boolean) => void;
  ensureOriginalPreviewForGroup: (group: ReceivedAssetGroup, priority: ThumbnailPriority) => unknown;
  getLoupe: () => LoupeState | null;
  groupIdentity: (group: ReceivedAssetGroup) => string;
  hasPreviewUrlForGroup: (group: ReceivedAssetGroup, maxEdge: number, original: boolean) => boolean;
  render: () => void;
  setLoupe: (loupe: LoupeState | null) => void;
};

export function createLoupeInteraction(deps: LoupeInteractionDeps) {
  let pendingLoupeTimer: number | null = null;
  let pendingLoupeGroupId: string | null = null;
  let pendingLoupeState: LoupeState | null = null;
  let pendingLoupeGroup: ReceivedAssetGroup | null = null;

  function updateFromPointer(
    event: PointerEvent,
    group: ReceivedAssetGroup,
    maxEdge: number,
    original = false,
  ) {
    if (original) {
      void deps.ensureOriginalPreviewForGroup(group, "visible");
    }
    if (!deps.hasPreviewUrlForGroup(group, maxEdge, original)) return;
    const next = loupeFromPointer(event, group, maxEdge, original, deps.getLoupe(), deps.groupIdentity);
    const previous = deps.getLoupe();
    if (!previous || previous.groupId !== next.groupId) {
      schedule(next, group);
      return;
    }
    deps.setLoupe(next);
    applyDom(next, group);
  }

  function handleWheel(event: WheelEvent, group: ReceivedAssetGroup, maxEdge: number, original = false) {
    const loupe = deps.getLoupe();
    if (!loupe || loupe.groupId !== deps.groupIdentity(group)) {
      return;
    }
    if (performance.now() - loupe.startedAtMs < LOUPE_ZOOM_READY_MS) {
      return;
    }
    event.preventDefault();
    event.stopPropagation();
    const nextZoom = nextLoupeZoom(loupe.zoom, event.deltaY < 0 ? "in" : "out");
    const next = {
      ...loupeFromPointer(event, group, maxEdge, original, loupe, deps.groupIdentity),
      zoom: nextZoom,
      startedAtMs: loupe.startedAtMs,
    };
    deps.setLoupe(next);
    applyDom(next, group);
  }

  function clearIfFloating() {
    clearPending();
    if (deps.getLoupe()) {
      deps.setLoupe(null);
      deps.render();
    }
  }

  function schedule(next: LoupeState, group: ReceivedAssetGroup) {
    const current = deps.getLoupe();
    if (current && current.groupId !== next.groupId) {
      deps.setLoupe(null);
      deps.render();
    }
    pendingLoupeState = next;
    pendingLoupeGroup = group;
    if (pendingLoupeGroupId === next.groupId && pendingLoupeTimer) {
      return;
    }
    clearPending();
    pendingLoupeGroupId = next.groupId;
    pendingLoupeState = next;
    pendingLoupeGroup = group;
    pendingLoupeTimer = window.setTimeout(() => {
      if (!pendingLoupeState || !pendingLoupeGroup) return;
      deps.setLoupe({
        ...pendingLoupeState,
        startedAtMs: performance.now(),
      });
      clearPending();
      deps.render();
    }, LOUPE_SHOW_DELAY_MS);
  }

  function clearPending() {
    if (pendingLoupeTimer) {
      window.clearTimeout(pendingLoupeTimer);
    }
    pendingLoupeTimer = null;
    pendingLoupeGroupId = null;
    pendingLoupeState = null;
    pendingLoupeGroup = null;
  }

  function applyDom(loupe: LoupeState, group: ReceivedAssetGroup) {
    const overlay = document.querySelector<HTMLElement>(".loupe-overlay");
    if (overlay) {
      positionLoupeOverlay(overlay, loupe);
    }

    const crop = document.querySelector<HTMLElement>(".loupe-crop");
    if (crop) {
      deps.applyPreviewBackground(crop, group, loupe.maxEdge, loupe.original);
      crop.style.backgroundPosition = `${loupe.x * 100}% ${loupe.y * 100}%`;
      crop.style.backgroundSize = `${loupe.zoom * 100}% auto`;
    }

    const zoomLabel = document.querySelector<HTMLElement>(".loupe-caption strong");
    if (zoomLabel) zoomLabel.textContent = `${loupe.zoom.toFixed(1)}x`;
  }

  return {
    applyDom,
    clearIfFloating,
    handleWheel,
    positionOverlay: positionLoupeOverlay,
    updateFromPointer,
  };
}

export function positionLoupeOverlay(overlay: HTMLElement, loupe: LoupeState) {
  const overlayWidth = Math.min(LOUPE_OVERLAY_MAX_WIDTH, window.innerWidth - 16);
  const overlayHeight = Math.min(LOUPE_OVERLAY_HEIGHT, window.innerHeight - 16);
  const gap = 18;
  const rightSide = loupe.clientX + gap + overlayWidth;
  const preferredLeft = rightSide > window.innerWidth - 8 ? loupe.clientX - overlayWidth - gap : loupe.clientX + gap;
  overlay.style.left = `${clamp(preferredLeft, 8, window.innerWidth - overlayWidth - 8)}px`;
  overlay.style.top = `${clamp(loupe.clientY - overlayHeight / 2, 8, window.innerHeight - overlayHeight - 8)}px`;
}

function loupeFromPointer(
  event: PointerEvent | WheelEvent,
  group: ReceivedAssetGroup,
  maxEdge: number,
  original: boolean,
  current: LoupeState | null,
  groupIdentity: (group: ReceivedAssetGroup) => string,
): LoupeState {
  const target = event.currentTarget as HTMLElement;
  const groupId = groupIdentity(group);
  const point = normalizedPreviewPoint(target, event.clientX, event.clientY);
  return {
    groupId,
    x: point.x,
    y: point.y,
    clientX: event.clientX,
    clientY: event.clientY,
    zoom: initialLoupeZoom(current?.zoom),
    maxEdge,
    original,
    startedAtMs: current?.groupId === groupId ? current.startedAtMs : performance.now(),
  };
}

function normalizedPreviewPoint(target: HTMLElement, clientX: number, clientY: number) {
  const rect = target.getBoundingClientRect();
  const image = target.querySelector<HTMLImageElement>(":scope > img.preview-image");
  const naturalWidth = image?.naturalWidth || rect.width || 1;
  const naturalHeight = image?.naturalHeight || rect.height || 1;
  const objectFit = image ? getComputedStyle(image).objectFit : "cover";
  const imageRect = objectFit === "cover"
    ? coverImageRect(
        { left: rect.left, top: rect.top, width: rect.width, height: rect.height },
        { width: naturalWidth, height: naturalHeight },
      )
    : containedImageRect(
        { left: rect.left, top: rect.top, width: rect.width, height: rect.height },
        { width: naturalWidth, height: naturalHeight },
      );
  return normalizedPointInRect(imageRect, { x: clientX, y: clientY });
}

function clamp(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value));
}
