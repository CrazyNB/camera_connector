import type { PreviewImageOptions, ReceivedAssetGroup } from "./appTypes";
import { append, commandButton, el, statusDot } from "./domHelpers";
import { selectedBurstIndex } from "./groupSelectors";
import { expandedGridColumn } from "./lightTableLayout";
import { compactEvaluationLabel, evaluationDot, formatPairLabel, readable, sourceStatus } from "./presentation";

export type AssetGroupCardOptions = {
  selectedGroupId: string | null;
  selectGroup: (group: ReceivedAssetGroup) => Promise<void>;
  appendPreviewImage: (container: HTMLElement, group: ReceivedAssetGroup, options?: PreviewImageOptions) => void;
  renderPreviewStatusBadge: (group: ReceivedAssetGroup, maxEdge?: number, original?: boolean) => HTMLElement;
  renderFaceRiskOverlay: (group: ReceivedAssetGroup) => HTMLElement;
  currentThumbnailMaxEdge: () => number;
  updateLoupeFromPointer: (event: PointerEvent, group: ReceivedAssetGroup, maxEdge?: number, original?: boolean) => void;
  clearLoupeIfFloating: () => void;
  handleLoupeWheel: (event: WheelEvent, group: ReceivedAssetGroup, maxEdge?: number, original?: boolean) => void;
  burstMembersOf: (group: ReceivedAssetGroup) => ReceivedAssetGroup[];
  previewTooltipForGroup: (group: ReceivedAssetGroup) => string;
};
export function renderGroupCard(group: ReceivedAssetGroup, gridColumns = 0, options: AssetGroupCardOptions) {
  const card = el("article", "group-card");
  const isExpanded = group.group_id === options.selectedGroupId;
  card.tabIndex = 0;
  card.addEventListener("click", () => void options.selectGroup(group));
  card.addEventListener("keydown", (event) => {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      void options.selectGroup(group);
    }
  });
  if (isExpanded) {
    card.classList.add("is-selected");
    card.classList.add("is-expanded");
    const gridColumn = expandedGridColumn(gridColumns);
    if (gridColumn) {
      card.style.gridColumn = gridColumn;
    }
  }
  const thumb = renderAssetThumb(group, "asset-thumb", isExpanded, options);
  append(thumb, renderThumbMeta(group));
  const body = el("div", "group-card-body");
  append(
    body,
    append(
      el("div", "group-card-title"),
      append(
        el("div", "capture-title-row"),
        el("strong", "", group.group_key),
        group.burst ? el("span", "burst-count", `${group.burst.member_count} burst`) : null,
      ),
    ),
    renderCardStatusLine(group),
    renderMarks(group),
  );
  append(card, thumb, body);
  if (isExpanded && group.burst) {
    append(card, renderBurstStrip(group, options));
  }
  return card;
}

function renderAssetThumb(group: ReceivedAssetGroup, className: string, original: boolean, options: AssetGroupCardOptions) {
  const thumb = el("div", `${className} ${sourceStatus(group)}`);
  options.appendPreviewImage(thumb, group, { original, eager: original });
  append(thumb, options.renderPreviewStatusBadge(group, options.currentThumbnailMaxEdge(), original), options.renderFaceRiskOverlay(group));
  thumb.addEventListener("pointermove", (event) => options.updateLoupeFromPointer(event, group, options.currentThumbnailMaxEdge(), true));
  thumb.addEventListener("pointerleave", () => options.clearLoupeIfFloating());
  thumb.addEventListener("wheel", (event) => options.handleLoupeWheel(event, group, options.currentThumbnailMaxEdge(), true), { passive: false });
  return thumb;
}

function renderThumbMeta(group: ReceivedAssetGroup) {
  const meta = el("div", "thumb-meta");
  append(meta, el("span", "pair-badge", formatPairLabel(group)));
  if (group.burst) {
    append(meta, el("span", "pair-badge burst", `${group.burst.member_count} 寮燻`));
  }
  return meta;
}

function renderCardStatusLine(group: ReceivedAssetGroup) {
  return append(
    el("div", "card-status-line"),
    append(el("span", "card-status-item"), statusDot(sourceStatus(group)), el("span", "", readable(sourceStatus(group)))),
    append(el("span", "card-status-item"), statusDot(evaluationDot(group)), el("span", "", compactEvaluationLabel(group))),
  );
}

function renderBurstStrip(group: ReceivedAssetGroup, options: AssetGroupCardOptions) {
  const strip = el("div", "burst-strip");
  const members = options.burstMembersOf(group);
  append(strip, el("span", "burst-label", `杩炴媿 ${selectedBurstIndex(group, members)} / ${group.burst?.member_count ?? members.length}`));
  const frameRow = el("div", "burst-frames");
  members.slice(0, 10).forEach((member, index) => {
    const frame = commandButton("", "burst-frame", (event?: Event) => {
      event?.stopPropagation();
      void options.selectGroup(member);
    });
    frame.title = options.previewTooltipForGroup(member);
    options.appendPreviewImage(frame, member);
    append(frame, el("span", "viewer-frame-index", String(index + 1)));
    if (member.group_id === group.group_id) frame.classList.add("is-current");
    append(frameRow, frame);
  });
  append(strip, frameRow);
  return strip;
}

function renderMarks(group: ReceivedAssetGroup) {
  const marks = el("div", "marks");
  if (group.user_marks.favorite) append(marks, el("span", "mark", "宸叉敹钘?"));
  if (group.user_marks.marked) append(marks, el("span", "mark", "宸叉爣璁?"));
  if (group.is_model_select) append(marks, el("span", "mark model", "AI 鎺ㄨ崘"));
  if (!marks.childElementCount) return null;
  return marks;
}
