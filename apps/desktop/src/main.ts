import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { deriveWorkbenchStage, type WorkbenchStage } from "./workflow";
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
  group_key?: string | null;
  storage_location?: StoredObjectLocation | null;
  original_path?: string | null;
  display_source?: string | null;
  virtual_display_path?: string | null;
  source_status?: string | null;
  source_modified_at_ms?: number | null;
  last_seen_scan_id?: string | null;
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
  model_summary?: string | null;
  is_model_select: boolean;
  user_marks: AssetUserMarks;
};

type AssetGroupSummary = {
  group_count: number;
  asset_count: number;
  groups_with_jpeg: number;
  groups_with_raw: number;
  groups_with_video: number;
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
  transfer_id: string;
  group_role: string;
  media_kind: string;
  format: string;
  original_filename: string;
  original_path: string;
  size_bytes: number;
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

type SourceFilter = "all" | "available" | "changed" | "missing";

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
  sourceFilter: SourceFilter;
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
  sourceFilter: "all",
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
  getAssetPage(projectId: string, offset = 0, limit = 120): Promise<AssetGroupPage> {
    return invoke("get_project_asset_page", { request: { project_id: projectId, offset, limit } });
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

void bootstrap().catch((error) => {
  setStatus("Startup failed", errorMessage(error));
});

async function bootstrap() {
  await loadProjects();
  await refreshCurrentProject(false);
  await listen<boolean>("desktop-scan-finished", async (event) => {
    setStatus(event.payload ? "Scan completed" : "Scan failed");
    await refreshCurrentProject(false);
  });
  window.setInterval(() => {
    if (state.scan && scanIsActive(state.scan.phase)) {
      void refreshCurrentProject(false);
    }
  }, 1400);
  render();
}

async function loadProjects() {
  state.projects = await api.listProjects();
  if (!state.selectedProjectId || !state.projects.some((project) => project.project_id === state.selectedProjectId)) {
    state.selectedProjectId = state.projects[0]?.project_id ?? null;
  }
}

async function refreshCurrentProject(showLoadedStatus = true) {
  const projectId = state.selectedProjectId;
  if (!projectId) {
    state.scan = null;
    state.assetPage = null;
    state.selectedGroupId = null;
    state.selectedGroup = null;
    state.groupDetail = [];
    render();
    return;
  }

  const [scan, assetPage] = await Promise.all([api.getScanStatus(projectId), api.getAssetPage(projectId)]);
  state.scan = scan;
  state.assetPage = assetPage;
  syncSelectedGroup();
  if (state.selectedGroupId) {
    state.groupDetail = await api.getGroupDetail(projectId, state.selectedGroupId);
  }
  if (showLoadedStatus) {
    state.status = "Project loaded";
  }
  render();
}

function syncSelectedGroup() {
  if (!state.selectedGroupId) {
    state.selectedGroup = null;
    state.groupDetail = [];
    return;
  }
  const selected = state.assetPage?.groups.find((group) => group.group_id === state.selectedGroupId) ?? null;
  state.selectedGroup = selected;
  if (!selected) {
    state.selectedGroupId = null;
    state.groupDetail = [];
  }
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
    setStatus("Start scan", "Create or select a project first");
    return;
  }
  if (!state.rootPath) {
    setStatus("Start scan", "Choose a folder first");
    return;
  }
  const scan = await withBusy("Start scan", () => api.startProjectScan(projectId, state.rootPath));
  if (scan) {
    state.scan = scan;
    state.selectedGroupId = null;
    state.selectedGroup = null;
    state.groupDetail = [];
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
  const summary = await withBusy("Run evaluation jobs", () => api.drainAnalysisJobs(50));
  if (summary) {
    state.status = `Evaluation jobs: ${summary.completed_count}/${summary.claimed_count} complete, ${summary.failed_count} failed`;
    await refreshCurrentProject(false);
  }
}

async function recommendBurst() {
  const burstId = state.selectedGroup?.burst?.burst_group_id;
  if (!burstId) {
    setStatus("Recommend burst", "Selected group is not part of a burst");
    return;
  }
  const recommendation = await withBusy("Recommend burst", () => api.recommendBurstGroup(burstId));
  if (recommendation) {
    state.lastRecommendation = recommendation;
    state.status = `Burst recommendation: ${readable(recommendation.status)}`;
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
    state.status = `Project recommendation: ${readable(recommendation.status)}`;
    await refreshCurrentProject(false);
  }
}

function render() {
  appRoot.replaceChildren(renderShell());
}

function renderShell() {
  const stage = currentStage();
  return append(el("div", "app-shell"), renderTopBar(stage), renderWorkflow(stage));
}

function renderTopBar(stage: WorkbenchStage) {
  const top = el("header", "topbar");
  const project = selectedProject();
  const title = append(
    el("div", "title-stack"),
    renderWindowControls(),
    el("div", "product-name", "Camera Connector"),
    el("div", "project-context", project ? project.name : "No project selected"),
  );
  const status = el("div", state.error ? "status is-error" : "status", state.error ?? state.busy ?? state.status);
  append(top, title, renderStageRail(stage), status);
  return top;
}

function renderWindowControls() {
  const controls = el("div", "window-controls");
  append(controls, el("span", "traffic close"), el("span", "traffic minimize"), el("span", "traffic zoom"));
  return controls;
}

function renderStageRail(stage: WorkbenchStage) {
  const stages: Array<[WorkbenchStage, string]> = [
    ["project", "Project"],
    ["folder", "Folder"],
    ["scan", "Scan"],
    ["review", "Review"],
  ];
  const rail = el("nav", "stage-rail");
  const activeIndex = stages.findIndex(([key]) => key === stage);
  stages.forEach(([key, label], index) => {
    const node = el("span", "stage-step");
    if (index < activeIndex) node.classList.add("is-complete");
    if (key === stage) node.classList.add("is-active");
    append(node, el("span", "step-index", String(index + 1)), el("span", "step-label", label));
    append(rail, node);
  });
  return rail;
}

function renderWorkflow(stage: WorkbenchStage) {
  const layout = el("main", "workflow-layout");
  append(layout, renderProjectSidebar(), renderStageSurface(stage));
  return layout;
}

function renderProjectSidebar() {
  const side = el("aside", "project-sidebar");
  append(side, el("h2", "", "Projects"));
  if (state.projects.length) {
    append(side, renderProjectCreate("compact"));
  }
  append(side, renderProjectList(), renderScanMemory());
  return side;
}

function renderProjectCreate(variant: "compact" | "hero" = "compact") {
  const row = el("form", `project-create project-create-${variant}`);
  row.addEventListener("submit", (event) => {
    event.preventDefault();
    void createProject();
  });
  append(
    row,
    textInput(state.projectNameDraft, "New project name", (value) => {
      state.projectNameDraft = value;
    }),
    commandButton(variant === "hero" ? "Create Project" : "Create", "primary", () => void createProject(), Boolean(state.busy)),
  );
  return row;
}

function renderProjectList() {
  const list = el("div", "project-list");
  if (!state.projects.length) {
    append(list, el("div", "empty-note", "Create a project to begin."));
    return list;
  }
  for (const project of state.projects) {
    const item = commandButton(project.name, "project-item", () => void selectProject(project.project_id), Boolean(state.busy));
    if (project.project_id === state.selectedProjectId) {
      item.classList.add("is-selected");
    }
    append(list, item);
  }
  return list;
}

function renderScanMemory() {
  const box = el("section", "scan-memory");
  append(box, el("h3", "", "Current Source"));
  append(box, el("div", "path-readout", state.rootPath || "No folder chosen"));
  if (state.scan) {
    append(
      box,
      compactMetric("Phase", readable(state.scan.phase)),
      compactMetric("Files", String(state.scan.files_seen)),
      compactMetric("Groups", String(state.scan.groups_updated)),
    );
  }
  return box;
}

function renderStageSurface(stage: WorkbenchStage) {
  const surface = el("section", `stage-surface stage-${stage}`);
  switch (stage) {
    case "project":
      append(surface, renderProjectStage());
      break;
    case "folder":
      append(surface, renderFolderStage());
      break;
    case "scan":
      append(surface, renderScanStage());
      break;
    case "review":
      append(surface, renderReviewStage());
      break;
  }
  return surface;
}

function renderProjectStage() {
  const card = el("section", "focus-panel");
  append(
    card,
    append(
      el("div", "project-hero-layout"),
      append(
        el("div", "project-intro"),
        el("p", "eyebrow", "Step 1"),
        el("h1", "", "Start a review workbench"),
        el(
          "p",
          "lead",
          "Create a project to hold the folder scan, grouped assets, marks, evaluations, and recommendations for this session.",
        ),
        renderProjectCreate("hero"),
      ),
      renderFlowPreview(),
    ),
  );
  return card;
}

function renderFlowPreview() {
  const preview = el("div", "flow-preview");
  append(
    preview,
    el("h2", "", "Next actions"),
    flowPreviewStep("Folder", "Choose a local source without importing files yet."),
    flowPreviewStep("Scan", "Index assets into the project as a desktop scan transfer."),
    flowPreviewStep("Review", "Inspect groups, mark keepers, and run recommendations."),
  );
  return preview;
}

function flowPreviewStep(title: string, detail: string) {
  return append(
    el("div", "flow-preview-step"),
    el("span", "flow-dot"),
    append(el("div", "flow-copy"), el("strong", "", title), el("span", "", detail)),
  );
}

function renderFolderStage() {
  const card = el("section", "focus-panel");
  append(
    card,
    el("p", "eyebrow", "Step 2"),
    el("h1", "", "Choose a source folder"),
    el("p", "lead", "The scan will index local files as desktop_scan transfers while keeping marks attached to stable groups."),
    renderFolderPicker(),
  );
  return card;
}

function renderFolderPicker() {
  const picker = el("div", "folder-picker");
  append(
    picker,
    el("div", "folder-icon", "DIR"),
    append(
      el("div", "folder-copy"),
      el("strong", "", state.rootPath ? "Selected folder" : "No folder selected"),
      el("span", "", state.rootPath || "Pick a local photo folder to continue."),
    ),
    commandButton(state.rootPath ? "Change Folder" : "Choose Folder", "secondary", () => void chooseFolder(), Boolean(state.busy)),
    commandButton("Continue", "primary", () => render(), !state.rootPath),
  );
  return picker;
}

function renderScanStage() {
  const card = el("section", "focus-panel scan-focus");
  const scan = state.scan;
  const scanLabel = scan ? readable(scan.phase) : "Ready";
  append(
    card,
    el("p", "eyebrow", "Step 3"),
    el("h1", "", scanIsActive(scan?.phase) ? "Scanning source folder" : "Scan the selected folder"),
    el("p", "lead", state.rootPath || "Choose a folder before scanning."),
    renderScanProgress(),
  );
  const actions = el("div", "primary-actions");
  append(
    actions,
    commandButton(scan?.assets_indexed ? "Rescan Folder" : "Start Scan", "primary large", () => void startScan(), !canStartScan()),
    commandButton("Change Folder", "secondary large", () => void chooseFolder(), Boolean(state.busy)),
  );
  append(card, actions, el("div", "scan-caption", `Status: ${scanLabel}`));
  return card;
}

function renderReviewStage() {
  const wrap = el("div", state.selectedGroup ? "review-stage has-inspector" : "review-stage");
  append(wrap, renderReviewMain());
  if (state.selectedGroup) {
    append(wrap, renderInspector(state.selectedGroup));
  }
  return wrap;
}

function renderReviewMain() {
  const main = el("section", "review-main");
  append(main, renderReviewHeader(), renderSourceTabs(), renderGroupBoard());
  return main;
}

function renderReviewHeader() {
  const header = el("div", "review-header");
  const summary = state.assetPage?.summary;
  const copy = append(
    el("div", "review-title"),
    el("p", "eyebrow", "Step 4"),
    el("h1", "", "Review scanned groups"),
    el("p", "lead", `${summary?.group_count ?? 0} groups, ${summary?.asset_count ?? 0} files indexed from the current source.`),
  );
  const actions = el("div", "review-actions");
  append(
    actions,
    commandButton("Run Evaluation", "secondary", () => void runAnalysisJobs(), Boolean(state.busy)),
    commandButton("Project Recommend", "primary", () => void recommendProject(), Boolean(state.busy)),
    commandButton("Rescan", "secondary", () => void startScan(), !canStartScan()),
  );
  append(header, copy, actions);
  return header;
}

function renderSourceTabs() {
  const tabs = el("div", "source-tabs");
  const filters: Array<[SourceFilter, string]> = [
    ["all", "All"],
    ["available", "Available"],
    ["changed", "Changed"],
    ["missing", "Missing"],
  ];
  for (const [filter, label] of filters) {
    const count = filter === "all" ? allGroups().length : allGroups().filter((group) => sourceStatus(group) === filter).length;
    const tab = commandButton(`${label} ${count}`, "source-tab", () => {
      state.sourceFilter = filter;
      state.selectedGroupId = null;
      state.selectedGroup = null;
      state.groupDetail = [];
      render();
    });
    if (state.sourceFilter === filter) {
      tab.classList.add("is-active");
    }
    append(tabs, tab);
  }
  return tabs;
}

function renderGroupBoard() {
  const groups = filteredGroups();
  const board = el("div", "group-board");
  if (!groups.length) {
    append(board, el("div", "empty-board", "No groups match this state."));
    return board;
  }
  for (const group of groups) {
    append(board, renderGroupCard(group));
  }
  return board;
}

function renderGroupCard(group: ReceivedAssetGroup) {
  const card = commandButton("", "group-card", () => void selectGroup(group));
  if (group.group_id === state.selectedGroupId) {
    card.classList.add("is-selected");
  }
  const thumb = append(el("div", `asset-thumb ${sourceStatus(group)}`), el("span", "", group.group_key.slice(0, 2)));
  const body = el("div", "group-card-body");
  append(
    body,
    append(el("div", "group-card-title"), el("strong", "", group.group_key), renderFormatBadges(group)),
    append(
      el("div", "group-card-meta"),
      statusChip(sourceStatus(group), "source"),
      statusChip(group.technical_gate_status ?? group.technical_status ?? "technical_pending", "technical"),
      statusChip(modelLabel(group), "model"),
    ),
    renderMarks(group),
  );
  append(card, thumb, body);
  return card;
}

function renderInspector(group: ReceivedAssetGroup) {
  const panel = el("aside", "inspector");
  append(
    panel,
    append(
      el("div", "inspector-head"),
      append(el("div", ""), el("p", "eyebrow", "Selected Group"), el("h2", "", group.group_key)),
      commandButton("Close", "ghost", () => {
        state.selectedGroupId = null;
        state.selectedGroup = null;
        state.groupDetail = [];
        render();
      }),
    ),
    renderInspectorActions(group),
    renderEvaluationPanel(group),
    renderFilesPanel(),
    renderRecommendationPanel(),
  );
  return panel;
}

function renderInspectorActions(group: ReceivedAssetGroup) {
  const actions = el("div", "inspector-actions");
  append(
    actions,
    commandButton(group.user_marks.favorite ? "Unfavorite" : "Favorite", "secondary", () => void toggleFavorite(), Boolean(state.busy)),
    commandButton(group.user_marks.marked ? "Unmark" : "Mark", "secondary", () => void toggleMarked(), Boolean(state.busy)),
    commandButton("Recommend Burst", "primary", () => void recommendBurst(), Boolean(state.busy || !group.burst)),
  );
  return actions;
}

function renderEvaluationPanel(group: ReceivedAssetGroup) {
  const panel = el("section", "detail-panel");
  append(
    panel,
    el("h3", "", "Evaluation"),
    kvGrid([
      ["Source", readable(sourceStatus(group))],
      ["Technical", readable(group.technical_gate_status ?? group.technical_status ?? "Pending")],
      ["Model", modelLabel(group)],
      ["Tier", readable(group.model_tier ?? "None")],
      ["Burst", group.burst ? `${group.burst.member_count} groups` : "None"],
      ["Recommendation", readable(group.burst?.recommendation_status ?? "None")],
    ]),
  );
  if (group.model_summary) {
    append(panel, el("p", "summary-text", group.model_summary));
  }
  if (group.technical_defects.length) {
    const defects = el("div", "defects");
    for (const defect of group.technical_defects) {
      append(defects, el("div", "defect", `${readable(defect.defect_type)} / ${readable(defect.severity)}`));
    }
    append(panel, defects);
  }
  return panel;
}

function renderFilesPanel() {
  const panel = el("section", "detail-panel");
  append(panel, el("h3", "", "Files"));
  const list = el("div", "file-list");
  if (!state.groupDetail.length) {
    append(list, el("div", "empty-note", "Select a group to load file details."));
  }
  for (const asset of state.groupDetail) {
    append(
      list,
      append(
        el("div", "file-item"),
        append(el("div", "file-name"), el("strong", "", asset.original_filename), el("span", "", asset.original_path)),
        statusChip(asset.source_status, "source"),
        el("span", "file-size", formatBytes(asset.size_bytes)),
      ),
    );
  }
  append(panel, list);
  return panel;
}

function renderRecommendationPanel() {
  const panel = el("section", "detail-panel");
  append(panel, el("h3", "", "Latest Recommendation"));
  const recommendation = state.lastRecommendation;
  if (!recommendation) {
    append(panel, el("div", "empty-note", "No recommendation has been generated in this session."));
    return panel;
  }
  append(
    panel,
    kvGrid([
      ["Scope", readable(recommendation.scope)],
      ["Status", readable(recommendation.status)],
      ["Confidence", `${Math.round(recommendation.confidence * 100)}%`],
      ["Selected", String(recommendation.selected_asset_group_ids.length)],
    ]),
    el("p", "summary-text", recommendation.reason),
  );
  return panel;
}

function renderScanProgress() {
  const scan = state.scan;
  const phase = scan?.phase ?? "not_started";
  const box = el("div", "scan-progress");
  const track = el("div", "scan-track");
  const fill = el("div", `scan-fill ${phase}`);
  fill.style.width = `${phaseProgress(phase)}%`;
  append(track, fill);
  append(
    box,
    append(
      el("div", "scan-metrics"),
      metric("Phase", readable(phase)),
      metric("Files Seen", scan?.files_seen ?? 0),
      metric("Assets", scan?.assets_indexed ?? 0),
      metric("Groups", scan?.groups_updated ?? 0),
    ),
    track,
  );
  if (scan?.error) {
    append(box, el("div", "inline-error", scan.error));
  }
  return box;
}

function selectedProject() {
  return state.projects.find((project) => project.project_id === state.selectedProjectId) ?? null;
}

function currentStage(): WorkbenchStage {
  return deriveWorkbenchStage({
    hasProject: Boolean(state.selectedProjectId),
    hasRootPath: Boolean(state.rootPath),
    scanPhase: state.scan?.phase ?? null,
    groupCount: state.assetPage?.total_groups ?? 0,
  });
}

function allGroups() {
  return state.assetPage?.groups ?? [];
}

function filteredGroups() {
  const groups = allGroups();
  if (state.sourceFilter === "all") {
    return groups;
  }
  return groups.filter((group) => sourceStatus(group) === state.sourceFilter);
}

function canStartScan() {
  return Boolean(state.selectedProjectId && state.rootPath && !state.busy && !scanIsActive(state.scan?.phase));
}

function scanIsActive(phase?: string | null) {
  return Boolean(phase && ["queued", "scanning", "indexing"].includes(phase));
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

function membersOf(group: ReceivedAssetGroup) {
  const members = [group.primary, group.jpeg, group.raw, group.video].filter(Boolean) as ReceivedAsset[];
  const seen = new Set<string>();
  return members.filter((member) => {
    if (seen.has(member.id)) return false;
    seen.add(member.id);
    return true;
  });
}

function sourceStatus(group: ReceivedAssetGroup) {
  const statuses = membersOf(group).map((asset) => asset.source_status ?? "available");
  if (statuses.includes("missing")) return "missing";
  if (statuses.includes("changed")) return "changed";
  return "available";
}

function modelLabel(group: ReceivedAssetGroup) {
  if (typeof group.model_score === "number") {
    return `${group.model_score} ${readable(group.model_tier ?? "model")}`;
  }
  return readable(group.model_status ?? "pending");
}

function phaseProgress(phase: string) {
  switch (phase) {
    case "queued":
      return 18;
    case "scanning":
      return 48;
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

function renderFormatBadges(group: ReceivedAssetGroup) {
  const badges = el("div", "format-badges");
  if (group.jpeg) append(badges, el("span", "format-badge jpeg", "JPG"));
  if (group.raw) append(badges, el("span", "format-badge raw", "RAW"));
  if (group.video) append(badges, el("span", "format-badge video", "MOV"));
  return badges;
}

function renderMarks(group: ReceivedAssetGroup) {
  const marks = el("div", "marks");
  if (group.user_marks.favorite) append(marks, el("span", "mark", "Favorite"));
  if (group.user_marks.marked) append(marks, el("span", "mark", "Marked"));
  if (group.is_model_select) append(marks, el("span", "mark model", "Model Select"));
  return marks;
}

function kvGrid(rows: Array<[string, string]>) {
  const grid = el("div", "kv-grid");
  for (const [label, value] of rows) {
    append(grid, append(el("div", "kv"), el("span", "kv-label", label), el("strong", "", value)));
  }
  return grid;
}

function metric(label: string, value: string | number) {
  return append(el("div", "metric"), el("strong", "", String(value)), el("span", "", label));
}

function compactMetric(label: string, value: string) {
  return append(el("div", "compact-metric"), el("span", "", label), el("strong", "", value));
}

function statusChip(value: string, kind: string) {
  return el("span", `status-chip ${kind} ${cssToken(value)}`, readable(value));
}

function commandButton(label: string, className: string, onClick: () => void, disabled = false) {
  const node = el("button", className, label);
  node.type = "button";
  node.disabled = disabled;
  node.addEventListener("click", onClick);
  return node;
}

function textInput(value: string, placeholder: string, onInput: (value: string) => void) {
  const node = el("input", "text-input") as HTMLInputElement;
  node.value = value;
  node.placeholder = placeholder;
  node.addEventListener("input", () => onInput(node.value));
  return node;
}

function el<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  className?: string,
  text?: string,
): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text !== undefined) node.textContent = text;
  return node;
}

function append<T extends HTMLElement>(parent: T, ...children: Array<Node | null | undefined>) {
  for (const child of children) {
    if (child) parent.appendChild(child);
  }
  return parent;
}

function readable(value: string) {
  return value
    .replace(/_/g, " ")
    .replace(/([a-z])([A-Z])/g, "$1 $2")
    .replace(/\b\w/g, (letter) => letter.toUpperCase());
}

function cssToken(value: string) {
  return value.toLowerCase().replace(/[^a-z0-9]+/g, "-");
}

function formatBytes(bytes: number) {
  if (bytes < 1024) return `${bytes} B`;
  const units = ["KB", "MB", "GB", "TB"];
  let value = bytes / 1024;
  let unit = units[0];
  for (let index = 1; value >= 1024 && index < units.length; index += 1) {
    value /= 1024;
    unit = units[index];
  }
  return `${value.toFixed(value >= 10 ? 0 : 1)} ${unit}`;
}
