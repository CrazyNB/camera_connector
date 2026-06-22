import type { ReceivedAssetGroup } from "./appTypes";
import { append, commandButton, el } from "./domHelpers";

export type ViewerActionDockOptions = {
  busy: string | null;
  modelReady: boolean;
  toggleFavorite: (group: ReceivedAssetGroup) => Promise<void>;
  toggleMarked: (group: ReceivedAssetGroup) => Promise<void>;
  runGroupAnalysis: (group: ReceivedAssetGroup) => Promise<void>;
  evaluateGroupWithModel: (group: ReceivedAssetGroup) => Promise<void>;
  recommendBurst: (group: ReceivedAssetGroup) => Promise<void>;
  removeFromBurst: (group: ReceivedAssetGroup) => Promise<void>;
  deleteAssetGroup: (group: ReceivedAssetGroup) => Promise<void>;
};

export function renderViewerActionDock(
  group: ReceivedAssetGroup,
  options: ViewerActionDockOptions,
) {
  const dock = el("section", "viewer-action-dock");
  const keep = commandButton(
    group.user_marks.favorite ? "宸叉敹钘?" : "鏀惰棌",
    "viewer-action",
    () => void options.toggleFavorite(group),
    Boolean(options.busy),
  );
  const mark = commandButton(
    group.user_marks.marked ? "宸叉爣璁?" : "鏍囪",
    "viewer-action",
    () => void options.toggleMarked(group),
    Boolean(options.busy),
  );
  if (group.user_marks.favorite) keep.classList.add("is-active");
  if (group.user_marks.marked) mark.classList.add("is-active");
  append(
    dock,
    keep,
    mark,
    commandButton("璐ㄩ噺妫€鏌?", "viewer-action", () => void options.runGroupAnalysis(group), Boolean(options.busy || !group.group_id)),
    commandButton("AI 璇勪环", "viewer-action", () => void options.evaluateGroupWithModel(group), Boolean(options.busy || !options.modelReady || !group.group_id)),
    commandButton("鎺ㄨ崘杩炴媿", "viewer-action primary-action", () => void options.recommendBurst(group), Boolean(options.busy || !group.burst)),
    commandButton("绉诲嚭杩炴媿", "viewer-action", () => void options.removeFromBurst(group), Boolean(options.busy || !group.burst)),
    commandButton("鍒犻櫎", "viewer-action danger-action", () => void options.deleteAssetGroup(group), Boolean(options.busy || !group.group_id)),
  );
  return dock;
}
