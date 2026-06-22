import type { ReceivedAssetGroup, StoredAsset, SubjectAssessment } from "./appTypes";
import type { IntelligenceSetupState } from "./intelligence";
import { append, commandButton, el } from "./domHelpers";
import { renderEvaluationPanel } from "./evaluationPanel";
import { renderFilesPanel } from "./filesPanel";

type SubjectAssessmentsByGroup = Record<string, SubjectAssessment[]>;

export type InspectorPanelOptions = {
  busy: string | null;
  subjectAssessments: SubjectAssessmentsByGroup;
  groupDetail: StoredAsset[];
  closeInspector: () => void;
  currentIntelligenceSetup: () => IntelligenceSetupState;
  toggleFavorite: () => Promise<void>;
  toggleMarked: () => Promise<void>;
  evaluateGroupWithModel: (group: ReceivedAssetGroup) => Promise<void>;
  recommendBurst: () => Promise<void>;
  deleteAssetGroup: (group: ReceivedAssetGroup) => Promise<void>;
};
export function renderInspector(group: ReceivedAssetGroup, options: InspectorPanelOptions) {
  const panel = el("aside", "inspector");
  append(
    panel,
    append(
      el("div", "inspector-head"),
      append(el("div", ""), el("p", "eyebrow", "鐓х墖缁?"), el("h2", "", group.group_key)),
      commandButton("鍏抽棴", "ghost", () => {
        options.closeInspector();
      }),
    ),
    renderInspectorActions(group, options),
    renderEvaluationPanel(group, options.subjectAssessments),
    renderFilesPanel(options.groupDetail),
  );
  return panel;
}

function renderInspectorActions(group: ReceivedAssetGroup, options: InspectorPanelOptions) {
  const actions = el("div", "inspector-actions");
  const setup = options.currentIntelligenceSetup();
  append(
    actions,
    append(
      el("div", "inspector-action-row"),
      commandButton(group.user_marks.favorite ? "鍙栨秷鏀惰棌" : "鏀惰棌", "primary", () => void options.toggleFavorite(), Boolean(options.busy)),
      commandButton(group.user_marks.marked ? "鍙栨秷鏍囪" : "鏍囪", "secondary", () => void options.toggleMarked(), Boolean(options.busy)),
    ),
    append(
      el("div", "inspector-action-row"),
      commandButton("AI 璇勪环", "secondary", () => void options.evaluateGroupWithModel(group), Boolean(options.busy || !setup.modelReady || !group.group_id)),
      commandButton("鎺ㄨ崘杩炴媿", "secondary", () => void options.recommendBurst(), Boolean(options.busy || !group.burst)),
    ),
    append(
      el("div", "inspector-danger-row"),
      commandButton("鍒犻櫎鍘熷浘", "secondary danger-text", () => void options.deleteAssetGroup(group), Boolean(options.busy || !group.group_id)),
    ),
  );
  return actions;
}
