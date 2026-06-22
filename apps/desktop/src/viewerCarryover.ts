import {
  viewerCarryoverSource,
} from "./viewerMode";
import type {
  ViewerCarryoverImage,
} from "./appTypes";

export function currentViewerMainImageCarryover(
  currentCarryover: ViewerCarryoverImage | null,
): ViewerCarryoverImage | null {
  const preview = document.querySelector<HTMLElement>(".viewer-main-preview");
  if (!preview) {
    return currentCarryover;
  }
  const candidates = [
    ...Array.from(preview.querySelectorAll<HTMLImageElement>(":scope > img.viewer-carryover-image")).map((image) => ({
      url: image.currentSrc || image.src,
      loaded: image.complete && image.naturalWidth > 0,
      role: "carryover" as const,
    })),
    ...Array.from(preview.querySelectorAll<HTMLImageElement>(":scope > img.preview-image")).map((image) => ({
      url: image.currentSrc || image.src,
      loaded: image.complete && image.naturalWidth > 0,
      role: "preview" as const,
    })),
  ];
  const url = viewerCarryoverSource(candidates, currentCarryover?.url ?? null);
  return url ? { url } : null;
}
