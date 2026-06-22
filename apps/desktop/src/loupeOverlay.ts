import type { LoupeState, ReceivedAssetGroup } from "./appTypes";
import { append, el } from "./domHelpers";

export type LoupeOverlayOptions = {
  loupe: LoupeState | null;
  groupByIdentity: (groupId: string) => ReceivedAssetGroup | null | undefined;
  positionLoupeOverlay: (overlay: HTMLElement, loupe: LoupeState) => void;
  setPreviewBackground: (container: HTMLElement, group: ReceivedAssetGroup, maxEdge: number, original?: boolean) => void;
};
export function renderLoupeOverlay(options: LoupeOverlayOptions) {
  const loupe = options.loupe;
  const group = loupe ? options.groupByIdentity(loupe.groupId) : null;
  const overlay = el("div", "loupe-overlay");
  if (!loupe || !group) {
    return overlay;
  }
  options.positionLoupeOverlay(overlay, loupe);
  const crop = el("div", "loupe-crop");
  crop.dataset.loupeGroup = loupe.groupId;
  options.setPreviewBackground(crop, group, loupe.maxEdge, loupe.original);
  crop.style.backgroundPosition = `${loupe.x * 100}% ${loupe.y * 100}%`;
  crop.style.backgroundSize = `${loupe.zoom * 100}% auto`;
  append(
    overlay,
    crop,
    append(el("div", "loupe-caption"), el("span", "", group.group_key), el("strong", "", `${loupe.zoom.toFixed(1)}x`)),
  );
  return overlay;
}
