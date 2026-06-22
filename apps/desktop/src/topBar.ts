import type { Project, SettingsPanel } from "./appTypes";
import { append, commandButton, el } from "./domHelpers";
import { compactError, scanIsActive } from "./presentation";
import { state } from "./appState";

export type TopBarOptions = {
  selectedProject: () => Project | null;
  renderProjectCreate: (variant: "compact" | "hero") => HTMLElement;
  selectProject: (projectId: string) => Promise<void>;
  openSettingsPanel: (panel: SettingsPanel) => Promise<void>;
  currentPreviewProgress: () => { label: string };
  render: () => void;
};
export function renderTopBar(options: TopBarOptions) {
  if (state.layoutMode === "viewer") {
    const top = el("header", "topbar viewer-microbar");
    const summary = state.assetPage?.summary;
    const project = options.selectedProject();
    const source = state.rootPath ? state.rootPath.split(/[\\/]/).filter(Boolean).at(-1) : "鏈粦瀹氭枃浠跺す";
    const status = el("div", state.error ? "viewer-status-pill is-error" : "viewer-status-pill", state.error ?? state.busy ?? state.status);
    append(
      top,
      append(
        el("div", "viewer-microbar-left"),
        commandButton("缃戞牸", "viewer-micro-button", () => {
          state.layoutMode = "grid";
          state.loupe = null;
          options.render();
        }),
        append(
          el("div", "viewer-micro-project"),
          el("span", "", project?.name ?? "鏈€夋嫨椤圭洰"),
          el("strong", "", source),
        ),
      ),
      append(
        el("div", "viewer-micro-context"),
        el("span", "", source),
        el("strong", "", `${summary?.group_count ?? 0} 缁刞`),
        el("span", "", `${summary?.asset_count ?? 0} 鏂囦欢`),
        renderScanProgressPill(),
        renderCvProgressPill(),
        renderPreviewProgressPill(options),
      ),
      status,
    );
    return top;
  }
  const top = el("header", "topbar");
  const status = el("div", state.error ? "status is-error" : "status", state.error ?? state.busy ?? state.status);
  append(top, renderProjectSwitcher(options), renderTopContext(options), status);
  return top;
}

function renderProjectSwitcher(options: TopBarOptions) {
  const project = options.selectedProject();
  const wrap = el("div", "project-switcher-wrap");
  const chooser = el("div", "project-switcher");
  const trigger = commandButton("", "project-menu-trigger", () => {
    state.projectMenuOpen = !state.projectMenuOpen;
    state.projectCreatorOpen = false;
    options.render();
  });
  append(
    trigger,
    append(
      el("div", "switcher-copy"),
      el("span", "product-name", "鐩告満杩炴帴鍣?"),
      el("strong", "", project ? project.name : "鏈€夋嫨椤圭洰"),
    ),
    el("span", "switcher-caret", ""),
  );
  append(chooser, trigger);
  append(
    chooser,
    commandButton("鏂板缓", "new-project-button", () => {
      state.projectCreatorOpen = !state.projectCreatorOpen;
      state.projectMenuOpen = false;
      options.render();
    }),
    commandButton("璁剧疆", "global-settings-button", () => void options.openSettingsPanel("global")),
  );
  append(wrap, chooser);
  if (state.projectMenuOpen) {
    append(wrap, renderProjectMenu(options));
  }
  if (state.projectCreatorOpen) {
    append(wrap, append(el("div", "project-create-popover"), options.renderProjectCreate("compact")));
  }
  return wrap;
}

function renderProjectMenu(options: TopBarOptions) {
  const menu = el("div", "project-menu-popover");
  if (!state.projects.length) {
    append(menu, el("div", "project-menu-empty", "鏆傛棤椤圭洰"));
    return menu;
  }
  const list = el("div", "project-menu-list");
  for (const project of state.projects) {
    const item = commandButton("", "project-menu-item", () => void options.selectProject(project.project_id));
    if (project.project_id === state.selectedProjectId) item.classList.add("is-active");
    append(
      item,
      append(el("span", ""), el("strong", "", project.name), el("small", "", project.slug)),
      project.project_id === state.selectedProjectId ? el("span", "project-menu-meta", "褰撳墠") : null,
    );
    append(list, item);
  }
  append(menu, list);
  return menu;
}

function renderTopContext(options: TopBarOptions) {
  const summary = state.assetPage?.summary;
  const source = state.rootPath ? state.rootPath.split(/[\\/]/).filter(Boolean).at(-1) : "鏈粦瀹氭枃浠跺す";
  const context = el("div", "top-context");
  append(
    context,
    el("span", "", source),
    el("strong", "", `${summary?.group_count ?? 0} 缁刞`),
    el("span", "", `${summary?.asset_count ?? 0} 鏂囦欢`),
    renderScanProgressPill(),
    renderCvProgressPill(),
    renderPreviewProgressPill(options),
  );
  return context;
}

function renderScanProgressPill() {
  const scan = scanProgressDisplay();
  const pill = el("span", `progress-pill ${scan.tone}`, scan.label);
  pill.title = scan.title;
  return pill;
}

function renderPreviewProgressPill(options: TopBarOptions) {
  const progress = options.currentPreviewProgress();
  const pill = el("span", "progress-pill preview-progress-pill", progress.label);
  pill.title = "楂樻竻棰勮杩涘害銆備綆娓呰〃绀哄凡缁忓彲鐪嬶紝楂樻竻鎴栧師鍥捐〃绀哄綋鍓嶉瑙堝畬鎴愩€?";
  pill.dataset.previewProgress = "true";
  return pill;
}

function renderCvProgressPill() {
  const progress = state.cvProgress;
  if (!progress) {
    return null;
  }
  const done = progress.assessed_count + progress.failed_count + progress.skipped_count;
  const total = Math.max(progress.total_count, done);
  const label = progress.scope === "group" ? `鍗曞紶璐ㄦ ${done}/${total}` : `鍏ㄥ眬璐ㄦ ${done}/${total}`;
  const pill = el("span", `progress-pill quality-progress-pill ${done >= total ? "ready" : "working"}`, label);
  pill.title = `瀹屾垚 ${progress.assessed_count}锛屽け璐?${progress.failed_count}锛岃烦杩?${progress.skipped_count}锛屼汉鑴?${progress.subject_count}`;
  return pill;
}

function scanProgressDisplay() {
  const scan = state.scan;
  const summary = state.assetPage?.summary;
  if (state.lanSyncPhase === "discovering" || state.lanSyncPhase === "syncing") {
    return {
      label: state.lanSyncPhase === "discovering" ? "project-sync discovery" : "project-sync match",
      title: "Matching LAN project context to the current scan index",
      tone: "working",
    };
  }
  if (!scan && !summary?.asset_count) {
    return { label: "Scan pending", title: "Bind a folder before scanning", tone: "idle" };
  }
  if (scanIsActive(scan?.phase)) {
    return {
      label: `鎵弿 ${scan?.files_seen ?? 0} 鏂囦欢 / ${scan?.groups_updated ?? 0} 缁刞`,
      title: `鎵弿闃舵锛?{readable(scan?.phase ?? "pending")}锛屽凡鍙戠幇 ${scan?.files_seen ?? 0} 涓枃浠讹紝宸茬储寮?${scan?.assets_indexed ?? 0} 涓収鐗囨枃浠禶`,
      tone: "working",
    };
  }
  if (scan?.phase === "failed") {
    return {
      label: `鎵弿澶辫触锛屼繚鐣?${summary?.group_count ?? scan.groups_updated} 缁刞`,
      title: compactError(scan.error ?? null) ?? "鎵弿澶辫触锛屽綋鍓嶇储寮曚粛鍙敤",
      tone: "failed",
    };
  }
  return {
    label: `鎵弿瀹屾垚 ${summary?.group_count ?? scan?.groups_updated ?? 0} 缁刞`,
    title: `宸茬储寮?${summary?.asset_count ?? scan?.assets_indexed ?? 0} 涓収鐗囨枃浠禶`,
    tone: "ready",
  };
}
