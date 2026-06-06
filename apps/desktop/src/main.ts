import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import "./styles.css";

type Project = {
  project_id: string;
  name: string;
  slug: string;
  status: string;
  created_at_ms: number;
  updated_at_ms: number;
};

type DesktopScanRun = {
  scan_id: string;
  project_id: string;
  root_path: string;
  root_label: string;
  phase: string;
  files_seen: number;
  assets_indexed: number;
  groups_updated: number;
  started_at_ms: number;
  updated_at_ms: number;
  completed_at_ms?: number | null;
  error?: string | null;
};

type StoredObjectLocation = unknown;

type ReceivedAsset = {
  id: string;
  filename: string;
  size_bytes: number;
  format: string;
  source: string;
  received_time_ms?: number | null;
  capture_time_ms?: number | null;
  width?: number | null;
  height?: number | null;
  group_key?: string | null;
  storage_location?: StoredObjectLocation | null;
  original_path?: string | null;
  username?: string | null;
  display_source?: string | null;
  remote_addr?: string | null;
  virtual_display_path?: string | null;
  source_status?: string | null;
  source_modified_at_ms?: number | null;
  last_seen_scan_id?: string | null;
  duplicate_index?: number | null;
  duplicate_count?: number | null;
};

type AssetUserMarks = {
  favorite: boolean;
  marked: boolean;
};

type ReceivedAssetBurstSummary = {
  burst_group_id: string;
  member_count: number;
  recommendation_status: string;
  best_asset_group_id?: string | null;
  best_score?: number | null;
};

type ReceivedAssetTechnicalDefectSummary = {
  defect_type: string;
  severity: string;
  confidence: number;
  reason?: string | null;
};

type ReceivedAssetGroup = {
  group_id?: string | null;
  group_key: string;
  primary: ReceivedAsset;
  jpeg?: ReceivedAsset | null;
  raw?: ReceivedAsset | null;
  video?: ReceivedAsset | null;
  burst?: ReceivedAssetBurstSummary | null;
  technical_status?: string | null;
  technical_gate_status?: string | null;
  technical_defects: ReceivedAssetTechnicalDefectSummary[];
  model_status?: string | null;
  model_score?: number | null;
  model_tier?: string | null;
  model_evaluator_kind?: string | null;
  model_summary?: string | null;
  is_model_select: boolean;
  is_favorite: boolean;
  is_flagged: boolean;
  user_marks: AssetUserMarks;
};

type AssetFacetCount = {
  value: string;
  group_count: number;
};

type AssetGroupSummary = {
  group_count: number;
  asset_count: number;
  groups_with_jpeg: number;
  groups_with_raw: number;
  groups_with_video: number;
  source_counts: AssetFacetCount[];
  remote_addr_counts: AssetFacetCount[];
};

type AssetGroupPage = {
  groups: ReceivedAssetGroup[];
  summary: AssetGroupSummary;
  offset: number;
  limit: number;
  total_groups: number;
  has_more: boolean;
};

type StoredAsset = {
  asset_id: string;
  project_id: string;
  group_id?: string | null;
  transfer_id: string;
  group_role: string;
  media_kind: string;
  format: string;
  original_filename: string;
  final_filename: string;
  normalized_stem: string;
  original_path: string;
  original_parent_path?: string | null;
  final_location?: StoredObjectLocation | null;
  size_bytes: number;
  capture_at_ms?: number | null;
  received_at_ms?: number | null;
  published_at_ms?: number | null;
  source_identity?: string | null;
  username?: string | null;
  remote_addr?: string | null;
  source_status: string;
  source_modified_at_ms?: number | null;
  last_seen_scan_id?: string | null;
};

type AnalysisDrainSummary = {
  claimed_count: number;
  completed_count: number;
  failed_count: number;
};

type SelectionRecommendation = {
  recommendation_id: string;
  scope: string;
  subject_id: string;
  selected_asset_group_ids: string[];
  candidate_asset_group_ids: string[];
  rejected_asset_group_ids: string[];
  status: string;
  confidence: number;
  reason: string;
};

type DesktopError = {
  code?: string;
  message?: string;
};

type AppState = {
  projects: Project[];
  selectedProjectId: string | null;
  rootPath: string;
  projectNameDraft: string;
  scan: DesktopScanRun | null;
  assetPage: AssetGroupPage | null;
  selectedGroupId: string | null;
  selectedGroup: ReceivedAssetGroup | null;
  groupDetail: StoredAsset[];
  busy: string | null;
  status: string;
  error: string | null;
  lastRecommendation: SelectionRecommendation | null;
};

const state: AppState = {
  projects: [],
  selectedProjectId: null,
  rootPath: "",
  projectNameDraft: "",
  scan: null,
  assetPage: null,
  selectedGroupId: null,
  selectedGroup: null,
  groupDetail: [],
  busy: null,
  status: "Ready",
  error: null,
  lastRecommendation: null,
};

const api = {
  createProject(name: string): Promise<Project> {
    return invoke("create_project", { name });
  },
  listProjects(): Promise<Project[]> {
    return invoke("list_projects");
  },
  selectProject(projectId: string): Promise<void> {
    return invoke("select_project", { projectId });
  },
  startProjectScan(projectId: string, rootPath: string): Promise<DesktopScanRun> {
    return invoke("start_project_scan", { projectId, rootPath });
  },
  getScanStatus(projectId: string): Promise<DesktopScanRun | null> {
    return invoke("get_scan_status", { projectId });
  },
  getAssetPage(projectId: string, offset = 0, limit = 80): Promise<AssetGroupPage> {
    return invoke("get_project_asset_page", {
      request: { project_id: projectId, offset, limit },
    });
  },
  getGroupDetail(projectId: string, groupId: string): Promise<StoredAsset[]> {
    return invoke("get_project_group_detail", { projectId, groupId });
  },
  saveGroupUserMarks(
    projectId: string,
    groupId: string,
    favorite: boolean | null,
    marked: boolean | null,
  ): Promise<AssetUserMarks> {
    return invoke("save_group_user_marks", {
      request: { project_id: projectId, group_id: groupId, favorite, marked },
    });
  },
  drainAnalysisJobs(limit: number): Promise<AnalysisDrainSummary> {
    return invoke("drain_analysis_jobs", { limit });
  },
  recommendBurstGroup(burstGroupId: string): Promise<SelectionRecommendation> {
    return invoke("recommend_burst_group", { burstGroupId });
  },
  generateProjectRecommendation(projectId: string): Promise<SelectionRecommendation> {
    return invoke("generate_project_recommendation", { projectId });
  },
};

const app = document.querySelector<HTMLDivElement>("#app");
if (!app) {
  throw new Error("app root not found");
}
const appRoot = app;

function el<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  className?: string,
  text?: string,
): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  if (className) {
    node.className = className;
  }
  if (text !== undefined) {
    node.textContent = text;
  }
  return node;
}

function append(parent: HTMLElement, ...children: Array<Node | null | undefined>): HTMLElement {
  for (const child of children) {
    if (child) {
      parent.appendChild(child);
    }
  }
  return parent;
}

function textButton(label: string, className: string, onClick: () => void, disabled = false) {
  const node = el("button", className, label);
  node.type = "button";
  node.disabled = disabled;
  node.addEventListener("click", onClick);
  return node;
}

function input(value: string, placeholder: string, onInput: (value: string) => void) {
  const node = el("input", "field-input") as HTMLInputElement;
  node.value = value;
  node.placeholder = placeholder;
  node.addEventListener("input", () => onInput(node.value));
  return node;
}

function selectedProject() {
  return state.projects.find((project) => project.project_id === state.selectedProjectId) ?? null;
}

function errorMessage(error: unknown): string {
  if (typeof error === "string") {
    return error;
  }
  if (error && typeof error === "object") {
    const desktopError = error as DesktopError;
    if (desktopError.message) {
      return desktopError.code ? `${desktopError.code}: ${desktopError.message}` : desktopError.message;
    }
  }
  return "Unexpected desktop error";
}

function setStatus(message: string, error: string | null = null) {
  state.status = message;
  state.error = error;
  render();
}

async function withBusy<T>(label: string, task: () => Promise<T>): Promise<T | null> {
  state.busy = label;
  state.error = null;
  render();
  try {
    const result = await task();
    state.busy = null;
    render();
    return result;
  } catch (error) {
    state.busy = null;
    setStatus(label, errorMessage(error));
    return null;
  }
}

async function bootstrap() {
  await loadProjects();
  await refreshCurrentProject();
  await listen<boolean>("desktop-scan-finished", async (event) => {
    setStatus(event.payload ? "Scan completed" : "Scan failed");
    await refreshCurrentProject();
  });
  window.setInterval(() => {
    if (state.scan && ["queued", "scanning", "indexing"].includes(state.scan.phase)) {
      void refreshCurrentProject(false);
    }
  }, 1400);
  render();
}

async function loadProjects() {
  const projects = await api.listProjects();
  state.projects = projects;
  if (!state.selectedProjectId || !projects.some((project) => project.project_id === state.selectedProjectId)) {
    state.selectedProjectId = projects[0]?.project_id ?? null;
  }
}

async function refreshCurrentProject(showStatus = true) {
  const projectId = state.selectedProjectId;
  if (!projectId) {
    state.scan = null;
    state.assetPage = null;
    state.selectedGroup = null;
    state.groupDetail = [];
    return;
  }
  const [scan, assetPage] = await Promise.all([api.getScanStatus(projectId), api.getAssetPage(projectId)]);
  state.scan = scan;
  state.assetPage = assetPage;
  syncSelectedGroup();
  if (state.selectedGroupId) {
    state.groupDetail = await api.getGroupDetail(projectId, state.selectedGroupId);
  }
  if (showStatus) {
    state.status = "Project loaded";
  }
  render();
}

function syncSelectedGroup() {
  const groups = state.assetPage?.groups ?? [];
  if (!groups.length) {
    state.selectedGroupId = null;
    state.selectedGroup = null;
    state.groupDetail = [];
    return;
  }
  const selected =
    groups.find((group) => group.group_id === state.selectedGroupId) ??
    groups.find((group) => group.group_id) ??
    groups[0];
  state.selectedGroupId = selected.group_id ?? null;
  state.selectedGroup = selected;
}

async function createProject() {
  const name = state.projectNameDraft.trim();
  if (!name) {
    setStatus("Create project", "Project name is required");
    return;
  }
  const project = await withBusy("Create project", () => api.createProject(name));
  if (!project) {
    return;
  }
  state.projectNameDraft = "";
  await loadProjects();
  await selectProject(project.project_id);
}

async function selectProject(projectId: string) {
  await withBusy("Select project", async () => {
    await api.selectProject(projectId);
    state.selectedProjectId = projectId;
    state.selectedGroupId = null;
    state.selectedGroup = null;
    state.groupDetail = [];
    await refreshCurrentProject(false);
  });
}

async function chooseFolder() {
  const selected = await open({ directory: true, multiple: false });
  if (typeof selected === "string") {
    state.rootPath = selected;
    setStatus("Folder selected");
  }
}

async function startScan() {
  const projectId = state.selectedProjectId;
  if (!projectId) {
    setStatus("Start scan", "Select or create a project first");
    return;
  }
  if (!state.rootPath) {
    setStatus("Start scan", "Choose a folder first");
    return;
  }
  const scan = await withBusy("Start scan", () => api.startProjectScan(projectId, state.rootPath));
  if (scan) {
    state.scan = scan;
    state.status = "Scan queued";
    render();
  }
}

async function selectGroup(group: ReceivedAssetGroup) {
  state.selectedGroupId = group.group_id ?? null;
  state.selectedGroup = group;
  state.groupDetail = [];
  render();
  const projectId = state.selectedProjectId;
  if (projectId && group.group_id) {
    const detail = await withBusy("Load group", () => api.getGroupDetail(projectId, group.group_id as string));
    state.groupDetail = detail ?? [];
    render();
  }
}

async function toggleFavorite() {
  const projectId = state.selectedProjectId;
  const group = state.selectedGroup;
  if (!projectId || !group?.group_id) {
    return;
  }
  await withBusy("Save favorite", () =>
    api.saveGroupUserMarks(projectId, group.group_id as string, !group.user_marks.favorite, null),
  );
  await refreshCurrentProject(false);
}

async function toggleMarked() {
  const projectId = state.selectedProjectId;
  const group = state.selectedGroup;
  if (!projectId || !group?.group_id) {
    return;
  }
  await withBusy("Save mark", () =>
    api.saveGroupUserMarks(projectId, group.group_id as string, null, !group.user_marks.marked),
  );
  await refreshCurrentProject(false);
}

async function runAnalysisJobs() {
  const summary = await withBusy("Run jobs", () => api.drainAnalysisJobs(40));
  if (summary) {
    state.status = `Jobs completed ${summary.completed_count}/${summary.claimed_count}, failed ${summary.failed_count}`;
    await refreshCurrentProject(false);
  }
}

async function recommendBurst() {
  const burstId = state.selectedGroup?.burst?.burst_group_id;
  if (!burstId) {
    setStatus("Recommend burst", "Selected group is not in a burst");
    return;
  }
  const recommendation = await withBusy("Recommend burst", () => api.recommendBurstGroup(burstId));
  if (recommendation) {
    state.lastRecommendation = recommendation;
    state.status = `Burst recommendation ${recommendation.status}`;
    await refreshCurrentProject(false);
  }
}

async function recommendProject() {
  const projectId = state.selectedProjectId;
  if (!projectId) {
    return;
  }
  const recommendation = await withBusy("Project recommendation", () => api.generateProjectRecommendation(projectId));
  if (recommendation) {
    state.lastRecommendation = recommendation;
    state.status = `Project recommendation ${recommendation.status}`;
    await refreshCurrentProject(false);
  }
}

function render() {
  appRoot.replaceChildren(renderShell());
}

function renderShell() {
  const shell = el("div", "app-shell");
  append(shell, renderTopBar(), renderBody());
  return shell;
}

function renderTopBar() {
  const top = el("header", "topbar");
  const titleBlock = el("div", "title-block");
  append(titleBlock, el("div", "brand", "Camera Connector"), el("div", "context", selectedProject()?.name ?? "No project"));
  const status = el("div", state.error ? "status status-error" : "status", state.error ?? state.busy ?? state.status);
  append(top, titleBlock, status);
  return top;
}

function renderBody() {
  const body = el("main", "workspace");
  append(body, renderProjectRail(), renderWorkbench(), renderDetailPanel());
  return body;
}

function renderProjectRail() {
  const rail = el("aside", "project-rail");
  const header = el("div", "rail-header");
  append(header, el("h2", "", "Projects"));
  const createRow = el("div", "create-row");
  append(
    createRow,
    input(state.projectNameDraft, "Project name", (value) => {
      state.projectNameDraft = value;
    }),
    textButton("New", "primary small", () => void createProject(), Boolean(state.busy)),
  );
  const list = el("div", "project-list");
  if (!state.projects.length) {
    append(list, el("div", "empty", "No projects"));
  }
  for (const project of state.projects) {
    const item = textButton(project.name, "project-item", () => void selectProject(project.project_id), Boolean(state.busy));
    if (project.project_id === state.selectedProjectId) {
      item.classList.add("is-selected");
    }
    append(list, item);
  }
  append(rail, header, createRow, list);
  return rail;
}

function renderWorkbench() {
  const section = el("section", "workbench");
  append(section, renderScanPanel(), renderSummaryPanel(), renderAssetGrid());
  return section;
}

function renderScanPanel() {
  const panel = el("section", "scan-panel");
  const row = el("div", "scan-row");
  const root = el("div", "root-path", state.rootPath || "No folder selected");
  const canScan = Boolean(state.selectedProjectId && state.rootPath && !state.busy);
  append(
    row,
    root,
    textButton("Folder", "secondary", () => void chooseFolder(), Boolean(state.busy)),
    textButton("Scan", "primary", () => void startScan(), !canScan),
  );
  append(panel, row, renderScanProgress());
  return panel;
}

function renderScanProgress() {
  const scan = state.scan;
  const wrap = el("div", "scan-progress");
  const meta = el("div", "scan-meta");
  const phase = scan?.phase ?? "not_started";
  append(
    meta,
    pill("Phase", readable(phase), `phase-${phase}`),
    pill("Files", String(scan?.files_seen ?? 0)),
    pill("Assets", String(scan?.assets_indexed ?? 0)),
    pill("Groups", String(scan?.groups_updated ?? 0)),
  );
  const bar = el("div", "progress-track");
  const fill = el("div", "progress-fill");
  fill.style.width = `${phaseProgress(phase)}%`;
  if (phase === "failed") {
    fill.classList.add("failed");
  }
  append(bar, fill);
  append(wrap, meta, bar);
  if (scan?.error) {
    append(wrap, el("div", "inline-error", scan.error));
  }
  return wrap;
}

function renderSummaryPanel() {
  const summary = state.assetPage?.summary;
  const panel = el("section", "summary-panel");
  append(
    panel,
    metric("Groups", summary?.group_count ?? 0),
    metric("Assets", summary?.asset_count ?? 0),
    metric("JPEG", summary?.groups_with_jpeg ?? 0),
    metric("RAW", summary?.groups_with_raw ?? 0),
    metric("Video", summary?.groups_with_video ?? 0),
  );
  return panel;
}

function renderAssetGrid() {
  const panel = el("section", "asset-grid");
  const header = el("div", "section-header");
  const count = state.assetPage ? `${state.assetPage.total_groups} groups` : "No assets";
  append(header, el("h2", "", "Assets"), el("span", "muted", count));
  append(panel, header);
  const groups = state.assetPage?.groups ?? [];
  if (!groups.length) {
    append(panel, el("div", "empty-grid", "No scanned assets"));
    return panel;
  }
  const list = el("div", "group-list");
  for (const group of groups) {
    append(list, renderGroupRow(group));
  }
  append(panel, list);
  return panel;
}

function renderGroupRow(group: ReceivedAssetGroup) {
  const row = textButton("", "group-row", () => void selectGroup(group));
  if (group.group_id && group.group_id === state.selectedGroupId) {
    row.classList.add("is-selected");
  }
  const title = el("div", "group-title");
  append(title, el("strong", "", group.group_key), renderFormatBadges(group));
  const states = el("div", "group-states");
  append(
    states,
    stateBadge(sourceStatus(group), "source"),
    stateBadge(group.technical_gate_status ?? group.technical_status ?? "technical_pending", "technical"),
    stateBadge(modelLabel(group), "model"),
    stateBadge(group.burst?.recommendation_status ?? "no_burst", "recommendation"),
  );
  const marks = el("div", "marks");
  if (group.user_marks.favorite) {
    append(marks, el("span", "mark", "Favorite"));
  }
  if (group.user_marks.marked) {
    append(marks, el("span", "mark", "Marked"));
  }
  append(row, title, states, marks);
  return row;
}

function renderFormatBadges(group: ReceivedAssetGroup) {
  const badges = el("div", "format-badges");
  if (group.jpeg) append(badges, el("span", "format jpeg", "JPG"));
  if (group.raw) append(badges, el("span", "format raw", "RAW"));
  if (group.video) append(badges, el("span", "format video", "MOV"));
  return badges;
}

function renderDetailPanel() {
  const panel = el("aside", "detail-panel");
  const group = state.selectedGroup;
  if (!group) {
    append(panel, el("h2", "", "Detail"), el("div", "empty", "No group selected"));
    return panel;
  }
  append(panel, renderDetailHeader(group), renderDetailActions(group), renderEvaluationBlock(group), renderAssetDetailList(), renderRecommendationBlock());
  return panel;
}

function renderDetailHeader(group: ReceivedAssetGroup) {
  const header = el("div", "detail-header");
  append(header, el("h2", "", group.group_key), renderFormatBadges(group));
  const path = group.primary.original_path ?? group.primary.virtual_display_path ?? group.primary.filename;
  append(header, el("div", "detail-path", path));
  return header;
}

function renderDetailActions(group: ReceivedAssetGroup) {
  const actions = el("div", "detail-actions");
  append(
    actions,
    textButton(group.user_marks.favorite ? "Unfavorite" : "Favorite", "secondary", () => void toggleFavorite(), Boolean(state.busy)),
    textButton(group.user_marks.marked ? "Unmark" : "Mark", "secondary", () => void toggleMarked(), Boolean(state.busy)),
    textButton("Jobs", "secondary", () => void runAnalysisJobs(), Boolean(state.busy)),
    textButton("Burst", "secondary", () => void recommendBurst(), Boolean(state.busy || !group.burst)),
    textButton("Project", "primary", () => void recommendProject(), Boolean(state.busy || !state.selectedProjectId)),
  );
  return actions;
}

function renderEvaluationBlock(group: ReceivedAssetGroup) {
  const block = el("section", "detail-block");
  append(block, el("h3", "", "Evaluation"));
  const grid = el("div", "kv-grid");
  append(
    grid,
    kv("Source", readable(sourceStatus(group))),
    kv("Technical", readable(group.technical_gate_status ?? group.technical_status ?? "pending")),
    kv("Model", modelLabel(group)),
    kv("Tier", readable(group.model_tier ?? "none")),
    kv("Burst", group.burst ? `${group.burst.member_count} files` : "None"),
  );
  append(block, grid);
  if (group.model_summary) {
    append(block, el("p", "summary-text", group.model_summary));
  }
  if (group.technical_defects.length) {
    const defects = el("div", "defect-list");
    for (const defect of group.technical_defects) {
      append(defects, el("div", "defect", `${readable(defect.defect_type)} / ${defect.severity}`));
    }
    append(block, defects);
  }
  return block;
}

function renderAssetDetailList() {
  const block = el("section", "detail-block");
  append(block, el("h3", "", "Files"));
  const table = el("div", "file-table");
  if (!state.groupDetail.length) {
    append(table, el("div", "empty", "No file detail"));
  }
  for (const asset of state.groupDetail) {
    const row = el("div", "file-row");
    append(
      row,
      el("span", "file-name", asset.original_filename),
      el("span", "file-meta", asset.format),
      el("span", `file-status ${asset.source_status}`, readable(asset.source_status)),
      el("span", "file-size", formatBytes(asset.size_bytes)),
    );
    append(table, row);
  }
  append(block, table);
  return block;
}

function renderRecommendationBlock() {
  const block = el("section", "detail-block");
  append(block, el("h3", "", "Recommendation"));
  const recommendation = state.lastRecommendation;
  if (!recommendation) {
    append(block, el("div", "empty", "No recommendation"));
    return block;
  }
  const grid = el("div", "kv-grid");
  append(
    grid,
    kv("Scope", readable(recommendation.scope)),
    kv("Status", readable(recommendation.status)),
    kv("Confidence", `${Math.round(recommendation.confidence * 100)}%`),
    kv("Selected", String(recommendation.selected_asset_group_ids.length)),
  );
  append(block, grid, el("p", "summary-text", recommendation.reason));
  return block;
}

function pill(label: string, value: string, extraClass = "") {
  const node = el("span", `pill ${extraClass}`);
  append(node, el("span", "pill-label", label), el("span", "pill-value", value));
  return node;
}

function metric(label: string, value: number) {
  const node = el("div", "metric");
  append(node, el("span", "metric-value", String(value)), el("span", "metric-label", label));
  return node;
}

function kv(label: string, value: string) {
  const node = el("div", "kv");
  append(node, el("span", "kv-label", label), el("span", "kv-value", value));
  return node;
}

function stateBadge(value: string, kind: string) {
  return el("span", `state-badge ${kind} ${value}`, readable(value));
}

function membersOf(group: ReceivedAssetGroup) {
  const members = [group.primary, group.jpeg, group.raw, group.video].filter(Boolean) as ReceivedAsset[];
  const seen = new Set<string>();
  return members.filter((member) => {
    if (seen.has(member.id)) {
      return false;
    }
    seen.add(member.id);
    return true;
  });
}

function sourceStatus(group: ReceivedAssetGroup) {
  const statuses = membersOf(group).map((asset) => asset.source_status ?? "available");
  if (statuses.includes("missing")) return "missing";
  if (statuses.includes("changed")) return "changed";
  return statuses[0] ?? "available";
}

function modelLabel(group: ReceivedAssetGroup) {
  if (typeof group.model_score === "number") {
    return `${group.model_score} ${readable(group.model_tier ?? "model")}`;
  }
  return readable(group.model_status ?? "model_pending");
}

function phaseProgress(phase: string) {
  switch (phase) {
    case "queued":
      return 15;
    case "scanning":
      return 45;
    case "indexing":
      return 72;
    case "completed":
      return 100;
    case "failed":
      return 100;
    default:
      return 0;
  }
}

function readable(value: string) {
  return value
    .replace(/_/g, " ")
    .replace(/([a-z])([A-Z])/g, "$1 $2")
    .replace(/\b\w/g, (letter) => letter.toUpperCase());
}

function formatBytes(bytes: number) {
  if (bytes < 1024) {
    return `${bytes} B`;
  }
  const units = ["KB", "MB", "GB", "TB"];
  let value = bytes / 1024;
  let unit = units[0];
  for (let index = 1; value >= 1024 && index < units.length; index += 1) {
    value /= 1024;
    unit = units[index];
  }
  return `${value.toFixed(value >= 10 ? 0 : 1)} ${unit}`;
}

void bootstrap().catch((error) => {
  setStatus("Startup failed", errorMessage(error));
});
