import type { ReceivedAssetGroup } from "./appTypes";
import { state } from "./appState";
import { append, commandButton, el } from "./domHelpers";
import { compactError, scanIsActive } from "./presentation";

export type WorkbenchViewOptions = {
  renderInspector: (group: ReceivedAssetGroup) => HTMLElement;
  renderLightTable: () => HTMLElement;
  renderProjectCreate: (variant: "compact" | "hero") => HTMLElement;
  chooseFolder: () => Promise<void>;
  runAnalysisJobs: () => Promise<void>;
  recommendProject: () => Promise<void>;
  startScan: () => Promise<void>;
  canStartScan: () => boolean;
  render: () => void;
};
export function renderWorkbenchSurface(options: WorkbenchViewOptions) {
  const surface = el("section", "stage-surface review-surface");
  append(surface, renderReviewStage(options));
  return surface;
}

function renderReviewStage(options: WorkbenchViewOptions) {
  const inspectorGroup = state.layoutMode === "grid" ? state.selectedGroup : null;
  const showInspector = Boolean(inspectorGroup);
  const wrap = el("div", showInspector ? "review-stage has-inspector" : "review-stage");
  append(wrap, renderReviewMain(options));
  if (inspectorGroup) {
    append(wrap, options.renderInspector(inspectorGroup));
  }
  return wrap;
}

function renderReviewMain(options: WorkbenchViewOptions) {
  const main = el("section", "review-main");
  if (state.layoutMode !== "viewer") {
    append(main, renderReviewHeader(options));
  }
  append(main, options.renderLightTable());
  return main;
}

function renderReviewHeader(options: WorkbenchViewOptions) {
  const header = el("div", "review-header");
  const summary = state.assetPage?.summary;
  const hasProject = Boolean(state.selectedProjectId);
  const hasSource = Boolean(state.rootPath);
  const hasGroups = Boolean(summary?.group_count);
  const copy = append(
    el("div", "review-title"),
    el("h1", "", hasProject ? "閫夌墖鍙?" : "鍒涘缓閫夌墖椤圭洰"),
  );
  const actions = el("div", "review-actions");
  if (!hasProject) {
    // The project creation card owns the first-run action.
  } else if (!hasSource) {
    append(actions, commandButton("閫夋嫨鏂囦欢澶?", "primary", () => void options.chooseFolder(), Boolean(state.busy)));
  } else {
    append(
      actions,
      commandButton("鍏ㄥ眬璐ㄦ", "secondary", () => void options.runAnalysisJobs(), Boolean(state.busy || !hasGroups)),
      commandButton("鍏ㄥ眬鎺ㄨ崘", "primary", () => void options.recommendProject(), Boolean(state.busy || !hasGroups)),
    );
  }
  append(header, copy, actions);
  return header;
}

export function renderWorkbenchEmptyState(options: WorkbenchViewOptions) {
  const empty = el("div", "empty-workbench");
  if (!state.selectedProjectId) {
    append(
      empty,
      el("h2", "", "鍒涘缓椤圭洰骞剁粦瀹氭枃浠跺す"),
      el("p", "", "閫夋嫨鏈湴鐓х墖鐩綍鍚庝細绔嬪嵆閫掑綊绱㈠紩锛屾寜鎷嶆憚鍚嶅悎骞?RAW/JPG銆?"),
      options.renderProjectCreate("hero"),
    );
    return empty;
  }
  if (!state.rootPath) {
    append(
      empty,
      el("h2", "", "杩欎釜椤圭洰杩樻病鏈夋枃浠跺す"),
      el("p", "", "涓烘棫椤圭洰琛ヤ竴涓湰鍦扮収鐗囩洰褰曪紝闅忓悗浼氳嚜鍔ㄥ紑濮嬬储寮曘€?"),
      commandButton("缁戝畾鏂囦欢澶?", "primary large", () => void options.chooseFolder(), Boolean(state.busy)),
    );
    return empty;
  }
  if (scanIsActive(state.scan?.phase)) {
    append(empty, el("h2", "", "姝ｅ湪鎵弿"), el("p", "", "绱㈠紩鏇存柊鍚庯紝鐓х墖缁勪細閫愭鍑虹幇鍦ㄨ繖閲屻€?"));
    return empty;
  }
  if (state.scan?.phase === "failed") {
    append(
      empty,
      el("h2", "", "鎵弿澶辫触"),
      el("p", "", compactError(state.scan.error ?? null) ?? "鏃犳硶绱㈠紩杩欎釜鏂囦欢澶广€?"),
      commandButton("閲嶆柊鎵弿", "primary large", () => void options.startScan(), !options.canStartScan()),
    );
    return empty;
  }
  append(
    empty,
    el("h2", "", state.viewFilter === "light-table" && state.sourceFilter === "all" ? "No indexed photos yet" : "No results for current filters"),
    el("p", "", state.viewFilter === "light-table" && state.sourceFilter === "all" ? "Start a scan to show grouped photos here." : "Change filters or show all photo groups."),
    state.viewFilter === "light-table" && state.sourceFilter === "all"
      ? commandButton("鎵弿鏂囦欢澶?", "primary large", () => void options.startScan(), !options.canStartScan())
      : commandButton("鏄剧ず鍏ㄩ儴", "secondary large", () => {
          state.viewFilter = "light-table";
          state.sourceFilter = "all";
          options.render();
        }),
  );
  return empty;
}
