import type { PreviewImageOptions, ReceivedAssetGroup, SubjectAssessment } from "./appTypes";
import { append, commandButton, el, statusDot } from "./domHelpers";
import { groupIdentity, selectedBurstIndex } from "./groupSelectors";
import { renderEvaluationPanel } from "./evaluationPanel";
import { formatPairLabel, modelLabel, readable, sourceStatus } from "./presentation";

type SubjectAssessmentsByGroup = Record<string, SubjectAssessment[]>;

export type ViewerChromeOptions = {
  subjectAssessments: SubjectAssessmentsByGroup;
  burstMembersOf: (group: ReceivedAssetGroup) => ReceivedAssetGroup[];
  appendPreviewImage: (container: HTMLElement, group: ReceivedAssetGroup, options?: PreviewImageOptions) => void;
  currentThumbnailMaxEdge: () => number;
  previewTooltipForGroup: (group: ReceivedAssetGroup) => string;
  selectBurstMember: (group: ReceivedAssetGroup) => void;
  openInspector: () => void;
  closeInspector: () => void;
};
export function renderViewerRightRail(group: ReceivedAssetGroup, options: ViewerChromeOptions) {
  const rail = el("aside", "viewer-right-rail");
  const metrics: Array<[string, string]> = [
    ["鏂囦欢", sourceStatus(group)],
    ["璐ㄩ噺", group.technical_gate_status ?? group.technical_status ?? "pending"],
    ["AI", group.model_status ?? "pending"],
    ["鎺ㄨ崘", group.burst?.recommendation_status ?? "pending"],
  ];
  append(
    rail,
    commandButton("璇︽儏", "viewer-right-toggle", () => options.openInspector()),
  );
  for (const [label, status] of metrics) {
    const item = el("div", "viewer-right-dot");
    item.title = `${label}: ${readable(status)}`;
    append(item, statusDot(status), el("span", "", label.slice(0, 1)));
    append(rail, item);
  }
  return rail;
}

export function renderViewerInspector(group: ReceivedAssetGroup, groups: ReceivedAssetGroup[], options: ViewerChromeOptions) {
  const index = Math.max(0, groups.findIndex((candidate) => groupIdentity(candidate) === groupIdentity(group))) + 1;
  const panel = el("aside", "viewer-inspector");
  append(
    panel,
    append(
      el("div", "viewer-inspector-head"),
      append(
        el("div", "viewer-inspector-title"),
        el("span", "", `${index} / ${groups.length} / ${formatPairLabel(group)}`),
        el("h2", "", group.group_key),
      ),
      commandButton("鍏抽棴", "viewer-inspector-close", () => options.closeInspector()),
    ),
    append(
      el("div", "viewer-inspector-status"),
      viewerScoreMetric("鏂囦欢", readable(sourceStatus(group)), sourceStatus(group)),
      viewerScoreMetric("璐ㄩ噺", readable(group.technical_gate_status ?? group.technical_status ?? "pending"), group.technical_gate_status ?? group.technical_status ?? "pending"),
      viewerScoreMetric("AI", modelLabel(group), group.model_status ?? "pending"),
      viewerScoreMetric("鎺ㄨ崘", readable(group.burst?.recommendation_status ?? "not generated"), group.burst?.recommendation_status ?? "pending"),
    ),
    renderViewerBurstSummary(group, options),
    renderEvaluationPanel(group, options.subjectAssessments),
  );
  return panel;
}

function renderViewerBurstSummary(group: ReceivedAssetGroup, options: ViewerChromeOptions) {
  const members = options.burstMembersOf(group);
  const panel = el("section", "viewer-inspector-panel");
  append(panel, append(el("div", "viewer-section-label"), el("strong", "", "杩炴媿"), el("span", "", `${selectedBurstIndex(group, members)} / ${members.length}`)));
  if (members.length <= 1) {
    append(panel, el("div", "empty-note", "鍗曞紶鎷嶆憚"));
    return panel;
  }
  const frames = el("div", "viewer-inspector-burst");
  members.slice(0, 8).forEach((member, index) => {
    const frame = commandButton("", "viewer-inspector-frame", () =>
      options.selectBurstMember(member),
    );
    frame.title = options.previewTooltipForGroup(member);
    options.appendPreviewImage(frame, member, { maxEdge: options.currentThumbnailMaxEdge() });
    append(frame, el("span", "viewer-frame-index", String(index + 1)));
    if (groupIdentity(member) === groupIdentity(group)) {
      frame.classList.add("is-current");
    }
    append(frames, frame);
  });
  append(panel, frames);
  return panel;
}

function viewerScoreMetric(label: string, value: string, status: string) {
  return append(
    el("div", "viewer-score-metric"),
    append(el("span", "viewer-score-label"), statusDot(status), el("span", "", label)),
    el("strong", "", value),
  );
}

export function renderViewerBurstQueue(group: ReceivedAssetGroup, options: ViewerChromeOptions) {
  const members = options.burstMembersOf(group);
  if (members.length <= 1) {
    return null;
  }
  const strip = el("section", "viewer-burst-strip");
  strip.title = `杩炴媿 ${selectedBurstIndex(group, members)} / ${members.length}`;
  const frames = el("div", "viewer-burst-frames");
  members.forEach((member, index) => {
    const frame = commandButton("", "viewer-burst-frame", () =>
      options.selectBurstMember(member),
    );
    frame.title = options.previewTooltipForGroup(member);
    const media = el("span", "viewer-burst-media");
    options.appendPreviewImage(media, member, { maxEdge: options.currentThumbnailMaxEdge() });
    append(frame, media, el("span", "viewer-frame-index", String(index + 1)));
    if (groupIdentity(member) === groupIdentity(group)) {
      frame.classList.add("is-current");
    }
    append(frames, frame);
  });
  append(strip, frames);
  return strip;
}
