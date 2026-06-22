import type { PreviewImageOptions, ReceivedAssetGroup } from "./appTypes";
import { append, commandButton, el, statusDot } from "./domHelpers";
import { adjacentViewerGroup, viewerGroupIdentity, viewerQueueWindow } from "./viewerMode";
import { compactEvaluationLabel, formatPairLabel, sourceStatus } from "./presentation";

export type ViewerStageOptions = {
  viewerPreviewMaxEdge: number;
  viewerTransformZoom: number;
  viewerFilmstripCollapsed: boolean;
  appendPreviewImage: (container: HTMLElement, group: ReceivedAssetGroup, options?: PreviewImageOptions) => void;
  appendViewerCarryoverImage: (preview: HTMLElement) => void;
  renderPreviewStatusBadge: (group: ReceivedAssetGroup, maxEdge?: number, original?: boolean) => HTMLElement;
  renderFaceRiskOverlay: (group: ReceivedAssetGroup) => HTMLElement;
  handleWheel: (event: WheelEvent, preview: HTMLElement) => void;
  handleDoubleClick: (event: MouseEvent, preview: HTMLElement) => void;
  handlePointerDown: (event: PointerEvent, preview: HTMLElement) => void;
  handlePointerMove: (event: PointerEvent, preview: HTMLElement) => void;
  endDrag: (preview: HTMLElement, event?: PointerEvent) => void;
  applyTransformToNode: (preview: HTMLElement) => void;
  selectGroup: (group: ReceivedAssetGroup) => void;
  renderActionDock: (group: ReceivedAssetGroup) => HTMLElement;
  renderBurstQueue: (group: ReceivedAssetGroup) => HTMLElement | null;
  currentThumbnailMaxEdge: () => number;
  setFilmstripCollapsed: (collapsed: boolean) => void;
};
export function renderViewerStage(group: ReceivedAssetGroup, groups: ReceivedAssetGroup[], options: ViewerStageOptions) {
  const stage = el("section", "viewer-stage");
  const previous = adjacentViewerGroup(groups, group, -1);
  const next = adjacentViewerGroup(groups, group, 1);
  const preview = el("div", "viewer-main-preview");
  if (options.viewerTransformZoom > 1) {
    preview.classList.add("is-zoomed");
  }
  options.appendPreviewImage(preview, group, { maxEdge: options.viewerPreviewMaxEdge, original: true, eager: true });
  options.appendViewerCarryoverImage(preview);
  append(preview, options.renderPreviewStatusBadge(group, options.viewerPreviewMaxEdge, true), options.renderFaceRiskOverlay(group));
  preview.addEventListener("wheel", (event) => options.handleWheel(event, preview), { passive: false });
  preview.addEventListener("dblclick", (event) => options.handleDoubleClick(event, preview));
  preview.addEventListener("pointerdown", (event) => options.handlePointerDown(event, preview));
  preview.addEventListener("pointermove", (event) => options.handlePointerMove(event, preview));
  preview.addEventListener("pointerup", (event) => options.endDrag(preview, event));
  preview.addEventListener("pointercancel", (event) => options.endDrag(preview, event));
  preview.addEventListener("pointerleave", (event) => options.endDrag(preview, event));
  window.requestAnimationFrame(() => options.applyTransformToNode(preview));
  append(
    preview,
    append(
      el("div", "viewer-main-caption"),
      append(el("span", ""), el("strong", "", group.group_key), el("span", "", formatPairLabel(group))),
      append(el("span", ""), statusDot(sourceStatus(group)), el("span", "", compactEvaluationLabel(group))),
    ),
    previous ? commandButton("涓婁竴寮?", "viewer-nav previous", () => options.selectGroup(previous)) : null,
    next ? commandButton("涓嬩竴寮?", "viewer-nav next", () => options.selectGroup(next)) : null,
    options.renderActionDock(group),
    options.renderBurstQueue(group),
  );
  append(stage, preview);
  return stage;
}

export function renderViewerFilmstrip(groups: ReceivedAssetGroup[], current: ReceivedAssetGroup, options: ViewerStageOptions) {
  if (options.viewerFilmstripCollapsed) {
    const collapsed = el("section", "viewer-filmstrip-rail");
    append(
      collapsed,
      commandButton(`闃熷垪 / ${groups.length}`, "viewer-filmstrip-toggle", () => options.setFilmstripCollapsed(false)),
    );
    return collapsed;
  }
  const filmstrip = el("section", "viewer-filmstrip");
  const queue = viewerQueueWindow(groups, current, 10);
  append(
    filmstrip,
    append(
      el("div", "viewer-section-label"),
      el("strong", "", "闃熷垪"),
      append(
        el("span", "viewer-filmstrip-actions"),
        el("span", "", `${groups.length} 缁刞`),
        commandButton("鏀惰捣", "viewer-filmstrip-toggle", () => options.setFilmstripCollapsed(true)),
      ),
    ),
  );
  const frames = el("div", "viewer-filmstrip-frames");
  for (const group of queue) {
    const frame = commandButton("", "viewer-filmstrip-card", () => options.selectGroup(group));
    options.appendPreviewImage(frame, group, { maxEdge: options.currentThumbnailMaxEdge() });
    append(
      frame,
      options.renderPreviewStatusBadge(group),
      append(el("span", "viewer-filmstrip-meta"), el("strong", "", group.group_key), el("span", "", formatPairLabel(group))),
    );
    if (viewerGroupIdentity(group) === viewerGroupIdentity(current)) {
      frame.classList.add("is-current");
    }
    append(frames, frame);
  }
  append(filmstrip, frames);
  return filmstrip;
}
