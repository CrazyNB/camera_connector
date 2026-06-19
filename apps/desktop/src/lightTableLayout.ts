export type ReviewAction = "quality" | "ai" | "global-quality" | "global-recommend";
export type ReviewActionScope = "group" | "project";
export type LoupeZoomDirection = "in" | "out";

export const LOUPE_DEFAULT_ZOOM = 4;
export const LOUPE_MIN_ZOOM = 4;
export const LOUPE_MAX_ZOOM = 16;
export const LOUPE_ZOOM_STEP = 0.5;

export function expandedGridColumn(columns: number, span = 2): string | null {
  if (!Number.isFinite(columns) || columns <= span) {
    return null;
  }
  const safeSpan = Math.max(1, Math.floor(span));
  const safeColumns = Math.max(1, Math.floor(columns));
  const start = Math.floor((safeColumns - safeSpan) / 2) + 1;
  return `${Math.max(1, start)} / span ${safeSpan}`;
}

export function viewerActionScope(action: ReviewAction): ReviewActionScope {
  return action.startsWith("global-") ? "project" : "group";
}

export function initialLoupeZoom(currentZoom: number | null | undefined): number {
  if (!Number.isFinite(currentZoom)) {
    return LOUPE_DEFAULT_ZOOM;
  }
  return clamp(currentZoom as number, LOUPE_MIN_ZOOM, LOUPE_MAX_ZOOM);
}

export function nextLoupeZoom(currentZoom: number | null | undefined, direction: LoupeZoomDirection): number {
  const current = initialLoupeZoom(currentZoom);
  const delta = direction === "in" ? LOUPE_ZOOM_STEP : -LOUPE_ZOOM_STEP;
  return clamp(current + delta, LOUPE_MIN_ZOOM, LOUPE_MAX_ZOOM);
}

function clamp(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value));
}
