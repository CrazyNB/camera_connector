import type { ReceivedAssetGroup, SettingsPanel, SourceFilter, ViewFilter } from "./appTypes";
import type { ScanStartBlocker } from "./workflow";
import { scanTransferDisplay } from "./workflow";
import type { IntelligenceSetupState } from "./intelligence";
import { intelligenceStatusLabel } from "./intelligence";
import { append, commandButton, compactMetric, el, intelligenceLine, statusDot } from "./domHelpers";
import { needsWork } from "./groupSelectors";
import { compactError, readable, scanTransferDot, sourceStatus } from "./presentation";
import { state } from "./appState";

export type SidebarPanelOptions = {
  allGroups: () => ReceivedAssetGroup[];
  displayGroupsFor: (groups: ReceivedAssetGroup[]) => ReceivedAssetGroup[];
  chooseFolder: () => Promise<void>;
  startScan: () => Promise<void>;
  syncLanProjectContext: (showNoSourceStatus?: boolean) => Promise<void>;
  openSettingsPanel: (panel: SettingsPanel) => Promise<void>;
  evaluateGroupWithModel: () => Promise<void>;
  currentIntelligenceSetup: () => IntelligenceSetupState;
  getScanStartBlocker: () => ScanStartBlocker | null;
  renderProjectCreate: (variant: "compact" | "hero") => HTMLElement;
  lanSyncTransferDot: () => string;
  lanSyncTransferLabel: () => string;
  render: () => void;
};

export function renderViewerLeftRail(options: SidebarPanelOptions) {
  const side = el("aside", "project-sidebar viewer-left-rail");
  const summary = state.assetPage?.summary;
  const filters: Array<[SourceFilter, string]> = [
    ["all", "鍏ㄩ儴"],
    ["available", "鍙敤"],
    ["changed", "宸插彉鍖?"],
    ["missing", "缂哄け"],
  ];
  append(
    side,
    commandButton("缃戞牸", "viewer-left-button", () => {
      state.layoutMode = "grid";
      state.loupe = null;
      options.render();
    }),
    commandButton("鏂囦欢澶?", "viewer-left-button", () => void options.chooseFolder(), Boolean(state.busy || !state.selectedProjectId)),
    append(el("div", "viewer-left-separator")),
  );
  for (const [filter, label] of filters) {
    const count =
      filter === "all"
        ? options.displayGroupsFor(options.allGroups()).length
        : options.displayGroupsFor(options.allGroups().filter((group) => sourceStatus(group) === filter)).length;
    const button = commandButton("", "viewer-left-filter", () => {
      state.sourceFilter = filter;
      state.viewFilter = filter === "missing" ? "missing" : "light-table";
      options.render();
    });
    button.title = `${label} ${count}`;
    if (state.sourceFilter === filter) button.classList.add("is-active");
    append(button, statusDot(filter === "all" ? "neutral" : filter), el("strong", "", String(count)));
    append(side, button);
  }
  append(side, append(el("div", "viewer-left-total"), el("strong", "", String(summary?.group_count ?? 0)), el("span", "", "缁?")));
  return side;
}

export function renderSourcePanel(options: SidebarPanelOptions) {
  const box = el("section", "side-section source-section");
  const canChoose = Boolean(state.selectedProjectId && !state.busy);
  const blocker = options.getScanStartBlocker();
  append(
    box,
    append(
      el("div", "side-section-head"),
      el("h3", "", "鏂囦欢澶?"),
      commandButton(state.rootPath ? "鏇存崲" : "閫夋嫨", "side-link", () => void options.chooseFolder(), !canChoose),
    ),
    append(
      el("div", "source-path-row"),
      el("span", "source-folder-icon", ""),
      el("div", "path-readout", state.rootPath || "鏈€夋嫨鏂囦欢澶?"),
    ),
    el("p", "side-note", state.rootPath ? "Indexes all subfolders recursively." : "Bind a local photo folder for this project."),
  );
  if (state.rootPath) {
    append(
      box,
      commandButton(state.scan?.assets_indexed ? "閲嶆柊鎵弿" : "鎵弿鏂囦欢澶?", "source-action", () => void options.startScan(), Boolean(blocker)),
      commandButton("鍚屾灞€鍩熺綉椤圭洰", "source-action", () => void options.syncLanProjectContext(true), Boolean(blocker)),
    );
  }
  return box;
}

export function renderIntelligencePanel(options: SidebarPanelOptions) {
  if (!state.selectedProjectId) return null;
  const setup = options.currentIntelligenceSetup();
  const box = el("section", "side-section intelligence-section");
  append(
    box,
    append(
      el("div", "side-section-head"),
      el("h3", "", "AI 杈呭姪"),
      commandButton("璁剧疆", "side-link", () => {
        void options.openSettingsPanel("project");
      }),
    ),
    append(
      el("div", `intelligence-status ${setup.modelReady ? "is-ready" : "needs-setup"}`),
      statusDot(setup.modelReady ? "available" : "changed"),
      el("strong", "", readable(intelligenceStatusLabel(setup))),
    ),
    append(
      el("div", "intelligence-lines"),
      intelligenceLine("AI 鏈嶅姟", setup.selectedProvider?.provider_label ?? "鏈€夋嫨"),
      intelligenceLine("閫夌墖瑙勫垯", setup.selectedPrompt?.name ?? "鏈€夋嫨"),
      intelligenceLine("鍦烘櫙", readable(state.intelligenceSettings?.scene_profile ?? "general")),
      intelligenceLine("鑷姩 AI 璇勪环", setup.autoEvaluate ? "寮€鍚?" : "鍏抽棴"),
    ),
  );
  if (state.selectedGroup?.group_id) {
    append(
      box,
      commandButton("AI 璇勪环褰撳墠缁?", "source-action", () => void options.evaluateGroupWithModel(), Boolean(state.busy || !setup.modelReady)),
    );
  }
  return box;
}

export function renderTransferPanel(options: SidebarPanelOptions) {
  const scan = state.scan;
  const summary = state.assetPage?.summary;
  const box = el("section", "side-section");
  append(box, el("h3", "", "鎵弿璁板綍"));
  if (!scan && !summary?.asset_count) {
    append(box, el("p", "side-note", "褰撳墠椤圭洰杩樻病鏈夌储寮曡褰曘€?"));
    return box;
  }
  const transfer = scanTransferDisplay({
    scanPhase: scan?.phase ?? null,
    scanFilesSeen: scan?.files_seen ?? 0,
    scanAssetsIndexed: scan?.assets_indexed ?? 0,
    scanGroupsUpdated: scan?.groups_updated ?? 0,
    scanError: scan?.error ?? null,
    indexedAssetCount: summary?.asset_count ?? 0,
    indexedGroupCount: summary?.group_count ?? 0,
  });
  append(
    box,
    append(
      el("div", "transfer-title"),
      el("strong", "", "desktop-scan"),
      statusDot(scanTransferDot(transfer.health)),
      el("span", "", transfer.label),
    ),
    compactMetric("鏂囦欢", String(transfer.files)),
    compactMetric("鐓х墖缁?", String(transfer.groups)),
    compactMetric("鐓х墖鏂囦欢", String(transfer.assets)),
  );
  if (state.lanSyncPhase !== "idle" || state.lanSyncSummary) {
    append(
      box,
      append(
        el("div", "transfer-title"),
        el("strong", "", "project-sync"),
        statusDot(options.lanSyncTransferDot()),
        el("span", "", options.lanSyncTransferLabel()),
      ),
    );
  }
  if (transfer.note) {
    append(box, append(el("div", "transfer-note"), statusDot("missing"), el("span", "", transfer.note)));
  }
  if (state.lanSyncError) {
    append(box, append(el("div", "transfer-note"), statusDot("missing"), el("span", "", compactError(state.lanSyncError) ?? "")));
  }
  return box;
}

export function renderViewsPanel(options: SidebarPanelOptions) {
  const box = el("section", "side-section");
  append(box, el("h3", "", "瑙嗗浘"));
  const views: Array<[ViewFilter, string, number]> = [
    ["light-table", "閫夌墖鍙?", options.displayGroupsFor(options.allGroups()).length],
    ["needs-work", "寰呭鐞?", options.displayGroupsFor(options.allGroups().filter(needsWork)).length],
    ["missing", "缂哄け", options.displayGroupsFor(options.allGroups().filter((group) => sourceStatus(group) === "missing")).length],
  ];
  for (const [view, label, count] of views) {
    const item = commandButton("", "view-item", () => {
      state.viewFilter = view;
      if (view === "missing") state.sourceFilter = "missing";
      if (view === "light-table") state.sourceFilter = "all";
      options.render();
    });
    if (state.viewFilter === view) {
      item.classList.add("is-active");
    }
    append(item, el("span", "view-dot", ""), el("span", "", label), el("strong", "", String(count)));
    append(box, item);
  }
  return box;
}

export function renderFiltersPanel(options: SidebarPanelOptions) {
  const box = el("section", "side-section filters-section");
  append(box, el("h3", "", "鏂囦欢鐘舵€?"));
  const filters: Array<[SourceFilter, string]> = [
    ["all", "鍏ㄩ儴鐘舵€?"],
    ["available", "鍙敤"],
    ["changed", "宸插彉鍖?"],
    ["missing", "缂哄け"],
  ];
  for (const [filter, label] of filters) {
    const item = commandButton("", "filter-row", () => {
      state.sourceFilter = filter;
      state.viewFilter = filter === "missing" ? "missing" : "light-table";
      options.render();
    });
    if (state.sourceFilter === filter) item.classList.add("is-active");
    append(
      item,
      statusDot(filter === "all" ? "neutral" : filter),
      el("span", "", label),
      el("strong", "", String(filter === "all" ? options.displayGroupsFor(options.allGroups()).length : options.displayGroupsFor(options.allGroups().filter((group) => sourceStatus(group) === filter)).length)),
    );
    append(box, item);
  }
  return box;
}
