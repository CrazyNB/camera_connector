import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import {
  scanStartBlocker,
  scanTransferDisplay,
  type ScanStartBlocker,
} from "./workflow";
import { collapseBurstGroups } from "./burstDisplay";
import {
  isBrowserPreviewFormat,
  isPreviewableFormat,
  shouldRequestOriginalPreview,
  shouldRequestFullPreview,
  supportsFullThumbnailFormat,
} from "./mediaPreview";
import { fullThumbnailConcurrency } from "./thumbnailScheduler";
import { shouldApplyPreviewSync, type PreviewSyncQuality } from "./previewSync";
import { visibleGridWindow, type VisibleGridWindow } from "./virtualGrid";
import {
  adjacentViewerGroup,
  dragViewerTransform,
  resetViewerTransform,
  shouldPreserveViewerTransformForSelection,
  toggleViewerDoubleClickZoom,
  type ViewerTransform,
  viewerBurstWarmWindow,
  viewerCarryoverSource,
  viewerCurrentGroup,
  viewerGroupIdentity,
  viewerQueueWindow,
  zoomViewerTransformAtPoint,
} from "./viewerMode";
import { containedImageRect, normalizedContainedImagePoint } from "./imageViewport";
import {
  intelligenceSetupState,
  intelligenceStatusLabel,
  cvThresholdControlSpecs,
  newPromptDraft,
  promptDraftFromPack,
  promptDraftIsSaveable,
  promptStyleTagsFromText,
  providerDraftFromSettings,
  providerDraftIsSaveable,
  selectedCvThresholdMode,
  settingsForCvThresholdMode,
  technicalPolicyForCvPolicy,
  updateCvThresholdControl,
  type CreatePromptPackRequest,
  type CvThresholdMode,
  type ForkPromptPackRequest,
  type ModelProviderSettings,
  type PromptDraft,
  type ProjectEvaluationSettings,
  type PromptPack,
  type SaveModelProviderSettingsRequest,
  type SavePromptPackRequest,
} from "./intelligence";
import { previewBadge, previewProgress, type PreviewStage } from "./previewStatus";
import {
  selectLanProjectSnapshotSource,
  type LanProjectSnapshotSource,
} from "./lanProjectSync";
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

type ThumbnailResponse = {
  path: string;
  cached: boolean;
  quality?: ThumbnailQuality;
};

type OriginalPreviewResponse = {
  path: string;
  cached: boolean;
  direct_source: boolean;
};

type ThumbnailQuality = "fast" | "full";
type PreviewImageQuality = ThumbnailQuality | "original";

type SyncProjectSnapshotResponse = {
  matched_assets: number;
  matched_groups: number;
  applied_user_marks: number;
  applied_model_evaluations: number;
  applied_selection_recommendations: number;
  unresolved_records: number;
  ambiguous_records: number;
};

type LanSyncPhase = "idle" | "discovering" | "syncing" | "done" | "failed";

type PreviewImageOptions = {
  maxEdge?: number;
  original?: boolean;
  eager?: boolean;
};

type SelectGroupOptions = {
  preserveViewerTransform?: boolean;
};

type ThumbnailBatchItem = {
  source_path: string;
  path?: string | null;
  cached: boolean;
  error?: string | null;
};

type ThumbnailBatchResponse = {
  thumbnails: ThumbnailBatchItem[];
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
  status: string;
};

type EnqueueModelEvaluationResponse = {
  enqueued_count: number;
};

type DesktopCvAssessmentResponse = {
  assessed_count: number;
  failed_count: number;
  skipped_count: number;
  subject_count: number;
};

type SubjectAssessment = {
  assessment_id: string;
  project_id: string;
  asset_group_id: string;
  subject_type: string;
  detector_kind: string;
  detector_version: string;
  status: string;
  gate_status: string;
  regions_json: string;
  signals_json: string;
  summary: string;
  created_at_ms: number;
  updated_at_ms: number;
};

type SubjectRegion = {
  kind?: string;
  x?: number;
  y?: number;
  width?: number;
  height?: number;
  w?: number;
  h?: number;
};

type SubjectSignals = {
  face_count?: number;
  image_width?: number;
  image_height?: number;
  closed_eyes?: boolean;
  face_shadow_ratio?: number;
  face_highlight_ratio?: number;
  face_color_cast_strength?: number;
  face_exposure_risk?: boolean;
  face_color_cast_risk?: boolean;
};

type DesktopError = {
  code?: string;
  message?: string;
};

type SourceFilter = "all" | "available" | "changed" | "missing";

type ViewFilter = "light-table" | "needs-work" | "missing";

type LayoutMode = "grid" | "viewer";

type SettingsPanel = "project" | "global";

type LoupeState = {
  groupId: string;
  x: number;
  y: number;
  clientX: number;
  clientY: number;
  zoom: number;
  maxEdge: number;
  original: boolean;
  startedAtMs: number;
};

type ViewerCarryoverImage = {
  url: string;
};

type AppState = {
  projects: Project[];
  selectedProjectId: string | null;
  rootPath: string;
  projectNameDraft: string;
  projectFolderDraft: string;
  projectCreatorOpen: boolean;
  projectMenuOpen: boolean;
  scan: DesktopScanRun | null;
  assetPage: AssetGroupPage | null;
  selectedGroupId: string | null;
  selectedGroup: ReceivedAssetGroup | null;
  groupDetail: StoredAsset[];
  subjectAssessments: Record<string, SubjectAssessment[]>;
  intelligenceProviders: ModelProviderSettings[];
  intelligenceSettings: ProjectEvaluationSettings | null;
  promptPacks: PromptPack[];
  globalPromptPacks: PromptPack[];
  settingsPanel: SettingsPanel | null;
  providerDraft: SaveModelProviderSettingsRequest | null;
  promptDraft: PromptDraft | null;
  sourceFilter: SourceFilter;
  viewFilter: ViewFilter;
  layoutMode: LayoutMode;
  thumbSize: number;
  viewerTransform: ViewerTransform;
  viewerCarryoverImage: ViewerCarryoverImage | null;
  viewerInspectorOpen: boolean;
  viewerFilmstripCollapsed: boolean;
  loupe: LoupeState | null;
  busy: string | null;
  status: string;
  error: string | null;
  lanSyncPhase: LanSyncPhase;
  lanSyncSources: LanProjectSnapshotSource[];
  lanSyncSummary: SyncProjectSnapshotResponse | null;
  lanSyncError: string | null;
  boardScrollTop: number;
  boardWidth: number;
  assetPageLoading: boolean;
};

const state: AppState = {
  projects: [],
  selectedProjectId: null,
  rootPath: "",
  projectNameDraft: "",
  projectFolderDraft: "",
  projectCreatorOpen: false,
  projectMenuOpen: false,
  scan: null,
  assetPage: null,
  selectedGroupId: null,
  selectedGroup: null,
  groupDetail: [],
  subjectAssessments: {},
  intelligenceProviders: [],
  intelligenceSettings: null,
  promptPacks: [],
  globalPromptPacks: [],
  settingsPanel: null,
  providerDraft: null,
  promptDraft: null,
  sourceFilter: "all",
  viewFilter: "light-table",
  layoutMode: "grid",
  thumbSize: 320,
  viewerTransform: resetViewerTransform(),
  viewerCarryoverImage: null,
  viewerInspectorOpen: false,
  viewerFilmstripCollapsed: false,
  loupe: null,
  busy: null,
  status: "就绪",
  error: null,
  lanSyncPhase: "idle",
  lanSyncSources: [],
  lanSyncSummary: null,
  lanSyncError: null,
  boardScrollTop: 0,
  boardWidth: 0,
  assetPageLoading: false,
};

const ASSET_PAGE_LIMIT = 96;
const LOUPE_SHOW_DELAY_MS = 1000;
const LOUPE_EDGE_PAD = 0.12;
const GRID_GAP = 12;
const VIRTUAL_OVERSCAN_ROWS = 2;
const THUMBNAIL_MIN_EDGE = 640;
const THUMBNAIL_MAX_EDGE = 1280;
const VIEWER_PREVIEW_MAX_EDGE = 1280;
const THUMBNAIL_CONCURRENCY = 3;
const ORIGINAL_PREVIEW_CONCURRENCY = 4;
const THUMBNAIL_PREFETCH_ROWS = 3;
const THUMBNAIL_INITIAL_WARMUP_LIMIT = 48;
const THUMBNAIL_PAGE_WARMUP_LIMIT = 24;
const THUMBNAIL_WARMUP_DELAY_MS = 180;
const THUMBNAIL_BATCH_SIZE = 8;
const THUMBNAIL_SCROLL_IDLE_MS = 300;
const LOUPE_ZOOM_READY_MS = 1000;

let pendingLoupeTimer: number | null = null;
let pendingLoupeGroupId: string | null = null;
let pendingLoupeState: LoupeState | null = null;
let pendingLoupeGroup: ReceivedAssetGroup | null = null;
let virtualBoardFrame: number | null = null;
let lastVirtualSignature = "";
let thumbnailScrollIdleTimer: number | null = null;
let thumbnailScrolling = false;
let previewProgressFrame: number | null = null;
const thumbnailUrlCache = new Map<string, string>();
const thumbnailPending = new Map<string, Promise<string | null>>();
const originalImageWarmCache = new Set<string>();
const originalPreviewUrlCache = new Map<string, string>();
const originalPreviewPending = new Map<string, Promise<string | null>>();
type ThumbnailPriority = "visible" | "upgrade" | "prefetch";
type ThumbnailQueueItem = {
  key: string;
  localPath: string;
  maxEdge: number;
  quality: ThumbnailQuality;
  priority: ThumbnailPriority;
  resolve: (url: string | null) => void;
};
type OriginalPreviewQueueItem = {
  key: string;
  localPath: string;
  priority: ThumbnailPriority;
  resolve: (url: string | null) => void;
};
const thumbnailQueue: ThumbnailQueueItem[] = [];
const thumbnailQueued = new Map<string, ThumbnailQueueItem>();
const thumbnailBatchPending = new Set<string>();
const thumbnailActiveKeys = new Set<string>();
const thumbnailFailedKeys = new Set<string>();
const originalPreviewQueue: OriginalPreviewQueueItem[] = [];
const originalPreviewQueued = new Map<string, OriginalPreviewQueueItem>();
const originalPreviewActiveKeys = new Set<string>();
const originalPreviewFailedKeys = new Set<string>();
let thumbnailActiveCount = 0;
let thumbnailFullActiveCount = 0;
let originalPreviewActiveCount = 0;
let viewerDragState: { x: number; y: number } | null = null;

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
  getAssetPage(projectId: string, offset = 0, limit = ASSET_PAGE_LIMIT): Promise<AssetGroupPage> {
    return invoke("get_project_asset_page", { request: { project_id: projectId, offset, limit } });
  },
  getAssetThumbnail(
    sourcePath: string,
    maxEdge = THUMBNAIL_MAX_EDGE,
    quality: ThumbnailQuality = "fast",
  ): Promise<ThumbnailResponse> {
    return invoke("get_asset_thumbnail", { request: { source_path: sourcePath, max_edge: maxEdge, quality } });
  },
  getAssetThumbnails(
    sourcePaths: string[],
    maxEdge = THUMBNAIL_MAX_EDGE,
    quality: ThumbnailQuality = "fast",
  ): Promise<ThumbnailBatchResponse> {
    return invoke("get_asset_thumbnails", { request: { source_paths: sourcePaths, max_edge: maxEdge, quality } });
  },
  getAssetOriginalPreview(sourcePath: string): Promise<OriginalPreviewResponse> {
    return invoke("get_asset_original_preview", { request: { source_path: sourcePath } });
  },
  getGroupDetail(projectId: string, groupId: string): Promise<StoredAsset[]> {
    return invoke("get_project_group_detail", { projectId, groupId });
  },
  runDesktopCvAssessment(projectId: string, limit = 1000): Promise<DesktopCvAssessmentResponse> {
    return invoke("run_desktop_cv_assessment", { request: { project_id: projectId, limit } });
  },
  getSubjectAssessments(projectId: string, assetGroupIds: string[]): Promise<SubjectAssessment[]> {
    return invoke("get_subject_assessments_for_asset_groups", {
      request: { project_id: projectId, asset_group_ids: assetGroupIds },
    });
  },
  deleteAssetGroup(projectId: string, groupId: string): Promise<boolean> {
    return invoke("delete_project_asset_group", { projectId, groupId });
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
  getModelProviderSettingsList(): Promise<ModelProviderSettings[]> {
    return invoke("get_model_provider_settings_list");
  },
  saveModelProviderSettings(request: SaveModelProviderSettingsRequest): Promise<ModelProviderSettings> {
    return invoke("save_model_provider_settings", { request });
  },
  deleteModelProviderSettings(settingsId: string): Promise<boolean> {
    return invoke("delete_model_provider_settings", { settingsId });
  },
  getProjectEvaluationSettings(projectId: string): Promise<ProjectEvaluationSettings> {
    return invoke("get_project_evaluation_settings", { projectId });
  },
  saveProjectEvaluationSettings(settings: ProjectEvaluationSettings): Promise<ProjectEvaluationSettings> {
    return invoke("save_project_evaluation_settings", { settings });
  },
  getGlobalPromptPacks(): Promise<PromptPack[]> {
    return invoke("get_global_prompt_packs");
  },
  getProjectPromptPacks(projectId: string): Promise<PromptPack[]> {
    return invoke("get_project_prompt_packs", { projectId });
  },
  createGlobalPromptPack(request: CreatePromptPackRequest): Promise<PromptPack> {
    return invoke("create_global_prompt_pack", { request });
  },
  forkGlobalPromptPack(request: ForkPromptPackRequest): Promise<PromptPack> {
    return invoke("fork_global_prompt_pack", { request });
  },
  saveGlobalPromptPack(request: SavePromptPackRequest): Promise<PromptPack> {
    return invoke("save_global_prompt_pack", { request });
  },
  deleteGlobalPromptPack(promptPackId: string): Promise<boolean> {
    return invoke("delete_global_prompt_pack", { promptPackId });
  },
  enqueueModelEvaluation(projectId: string, assetGroupIds: string[]): Promise<EnqueueModelEvaluationResponse> {
    return invoke("enqueue_model_evaluation_for_asset_groups", {
      request: { project_id: projectId, asset_group_ids: assetGroupIds },
    });
  },
  drainAnalysisJobs(limit: number): Promise<AnalysisDrainSummary> {
    return invoke("drain_analysis_jobs", { limit });
  },
  recommendBurstGroup(burstGroupId: string): Promise<SelectionRecommendation> {
    return invoke("recommend_burst_group", { burstGroupId });
  },
  splitBurstMember(burstGroupId: string, memberGroupId: string): Promise<unknown> {
    return invoke("split_burst_member", { burstGroupId, memberGroupId });
  },
  generateProjectRecommendation(projectId: string): Promise<SelectionRecommendation> {
    return invoke("generate_project_recommendation", { projectId });
  },
  discoverLanProjectSnapshots(): Promise<LanProjectSnapshotSource[]> {
    return invoke("discover_lan_project_snapshots", { request: {} });
  },
  syncProjectSnapshotFromUrl(projectId: string, snapshotUrl: string): Promise<SyncProjectSnapshotResponse> {
    return invoke("sync_project_snapshot_from_url", { request: { project_id: projectId, snapshot_url: snapshotUrl } });
  },
};

const app = document.querySelector<HTMLDivElement>("#app");
if (!app) {
  throw new Error("app root not found");
}
const appRoot = app;

void bootstrap().catch((error) => {
  setStatus("启动失败", errorMessage(error));
});

async function bootstrap() {
  await loadProjects();
  await refreshCurrentProject(false);
  await listen<boolean>("desktop-scan-finished", async (event) => {
    setStatus(event.payload ? "扫描完成" : "扫描失败");
    await refreshCurrentProject(false);
    if (event.payload) {
      await syncLanProjectContext();
    }
  });
  window.addEventListener("resize", () => {
    lastVirtualSignature = "";
    updateActiveVirtualBoard();
    syncAllFaceRiskLayers();
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
    state.assetPageLoading = false;
    state.selectedGroupId = null;
    state.selectedGroup = null;
    state.groupDetail = [];
    state.subjectAssessments = {};
    state.intelligenceProviders = [];
    state.intelligenceSettings = null;
    state.promptPacks = [];
    state.globalPromptPacks = [];
    state.settingsPanel = null;
    state.providerDraft = null;
    state.promptDraft = null;
    resetBoardViewport();
    render();
    return;
  }

  const currentLimit = Math.max(ASSET_PAGE_LIMIT, state.assetPage?.groups.length ?? 0);
  const [
    scan,
    assetPage,
    intelligenceProviders,
    promptPacks,
    globalPromptPacks,
    intelligenceSettings,
  ] = await Promise.all([
    api.getScanStatus(projectId),
    api.getAssetPage(projectId, 0, currentLimit),
    api.getModelProviderSettingsList(),
    api.getProjectPromptPacks(projectId),
    api.getGlobalPromptPacks(),
    api.getProjectEvaluationSettings(projectId),
  ]);
  state.scan = scan;
  state.intelligenceProviders = intelligenceProviders;
  state.promptPacks = promptPacks;
  state.globalPromptPacks = globalPromptPacks;
  state.intelligenceSettings = intelligenceSettings;
  if (!state.rootPath && scan?.root_path) {
    state.rootPath = scan.root_path;
  }
  state.assetPage = assetPage;
  state.assetPageLoading = false;
  warmThumbnailsForGroups(assetPage.groups.slice(0, THUMBNAIL_INITIAL_WARMUP_LIMIT));
  syncSelectedGroup();
  await loadSubjectAssessmentsForGroups(assetPage.groups);
  if (state.selectedGroupId) {
    state.groupDetail = await api.getGroupDetail(projectId, state.selectedGroupId);
  }
  if (showLoadedStatus) {
    state.status = "项目已载入";
  }
  render();
}

async function syncLanProjectContext(showNoSourceStatus = false) {
  const projectId = state.selectedProjectId;
  if (!projectId) return;
  state.lanSyncPhase = "discovering";
  state.lanSyncError = null;
  render();
  try {
    const sources = await api.discoverLanProjectSnapshots();
    state.lanSyncSources = sources;
    const source = selectLanProjectSnapshotSource(sources, selectedProject()?.name);
    if (!source) {
      state.lanSyncPhase = "idle";
      if (showNoSourceStatus) {
        state.status = "project-sync no source";
      }
      render();
      return;
    }

    state.lanSyncPhase = "syncing";
    render();
    const summary = await api.syncProjectSnapshotFromUrl(projectId, source.snapshot_url);
    state.lanSyncSummary = summary;
    state.lanSyncPhase = "done";
    state.status = `project-sync ${summary.matched_groups} groups`;
    await refreshCurrentProject(false);
  } catch (error) {
    state.lanSyncPhase = "failed";
    state.lanSyncError = errorMessage(error);
    render();
  }
}

function clearLanSyncState() {
  state.lanSyncPhase = "idle";
  state.lanSyncSources = [];
  state.lanSyncSummary = null;
  state.lanSyncError = null;
}

async function loadSubjectAssessmentsForGroups(groups: ReceivedAssetGroup[]) {
  const projectId = state.selectedProjectId;
  if (!projectId) return;
  const groupIds = groups
    .map((group) => group.group_id)
    .filter((groupId): groupId is string => Boolean(groupId));
  if (!groupIds.length) return;
  let assessments: SubjectAssessment[] = [];
  try {
    assessments = await api.getSubjectAssessments(projectId, [...new Set(groupIds)]);
  } catch (error) {
    console.warn("subject assessments unavailable", error);
    return;
  }
  const next = { ...state.subjectAssessments };
  for (const groupId of groupIds) {
    next[groupId] = [];
  }
  for (const assessment of assessments) {
    const bucket = next[assessment.asset_group_id] ?? [];
    bucket.push(assessment);
    next[assessment.asset_group_id] = bucket;
  }
  state.subjectAssessments = next;
}

async function refreshPromptPackLists() {
  const globalPromptPacks = await api.getGlobalPromptPacks();
  state.globalPromptPacks = globalPromptPacks;
  if (state.selectedProjectId) {
    state.promptPacks = await api.getProjectPromptPacks(state.selectedProjectId);
  } else {
    state.promptPacks = [];
  }
}

async function openSettingsPanel(panel: SettingsPanel) {
  state.providerDraft = null;
  state.promptDraft = null;
  if (panel === "global") {
    const [providers, packs] = await Promise.all([
      api.getModelProviderSettingsList(),
      api.getGlobalPromptPacks(),
    ]);
    state.intelligenceProviders = providers;
    state.globalPromptPacks = packs;
  }
  state.settingsPanel = panel;
  render();
}

function resetBoardViewport() {
  state.boardScrollTop = 0;
  state.boardWidth = 0;
  lastVirtualSignature = "";
}

async function loadMoreAssetGroups() {
  const projectId = state.selectedProjectId;
  const page = state.assetPage;
  if (!projectId || !page?.has_more || state.assetPageLoading) {
    return;
  }
  state.assetPageLoading = true;
  try {
    const nextPage = await api.getAssetPage(projectId, page.groups.length, ASSET_PAGE_LIMIT);
    if (state.selectedProjectId !== projectId || state.assetPage !== page) {
      return;
    }
    state.assetPage = {
      ...nextPage,
      offset: 0,
      groups: [...page.groups, ...nextPage.groups],
    };
    warmThumbnailsForGroups(nextPage.groups.slice(0, THUMBNAIL_PAGE_WARMUP_LIMIT));
    syncSelectedGroup();
    updateActiveVirtualBoard();
  } catch (error) {
    setStatus("载入照片组", errorMessage(error));
  } finally {
    state.assetPageLoading = false;
  }
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
  const rootPath = state.projectFolderDraft.trim();
  if (!name) {
    setStatus("创建项目", "请输入项目名称。");
    return;
  }
  if (!rootPath) {
    setStatus("创建项目", "请选择照片文件夹。");
    return;
  }
  const project = await withBusy("创建项目", () => api.createProject(name));
  if (!project) {
    return;
  }
  state.projectNameDraft = "";
  state.projectFolderDraft = "";
  state.projectCreatorOpen = false;
  state.projectMenuOpen = false;
  await loadProjects();
  await selectProject(project.project_id);
  state.rootPath = rootPath;
  state.status = "文件夹已绑定";
  render();
  await startScan();
}

async function selectProject(projectId: string) {
  await withBusy("选择项目", async () => {
    await api.selectProject(projectId);
    state.selectedProjectId = projectId;
    state.rootPath = "";
    state.projectFolderDraft = "";
    state.projectMenuOpen = false;
    state.projectCreatorOpen = false;
    state.selectedGroupId = null;
    state.selectedGroup = null;
    state.groupDetail = [];
    state.providerDraft = null;
    state.promptDraft = null;
    clearLanSyncState();
    resetBoardViewport();
    await refreshCurrentProject(false);
  });
}

async function chooseProjectFolderDraft() {
  const selected = await open({ directory: true, multiple: false });
  if (typeof selected === "string") {
    state.projectFolderDraft = selected;
    state.status = "已选择照片文件夹";
    render();
  }
}

async function chooseFolder() {
  if (!state.selectedProjectId) {
    await chooseProjectFolderDraft();
    return;
  }
  const selected = await open({ directory: true, multiple: false });
  if (typeof selected === "string") {
    state.rootPath = selected;
    state.status = "文件夹已绑定";
    render();
    if (!getScanStartBlocker()) {
      await startScan();
    }
  }
}

async function startScan() {
  const projectId = state.selectedProjectId;
  if (!projectId) {
    setStatus("开始扫描", "请先创建或选择一个项目。");
    return;
  }
  if (!state.rootPath) {
    setStatus("开始扫描", "请先选择文件夹。");
    return;
  }
  const scan = await withBusy("开始扫描", () => api.startProjectScan(projectId, state.rootPath));
  if (scan) {
    state.scan = scan;
    clearLanSyncState();
    state.selectedGroupId = null;
    state.selectedGroup = null;
    state.groupDetail = [];
    resetBoardViewport();
    state.status = "扫描已排队";
    render();
  }
}

async function selectGroup(group: ReceivedAssetGroup, options: SelectGroupOptions = {}) {
  const keepViewerTransform = shouldPreserveViewerTransformForSelection(
    state.selectedGroup,
    group,
    Boolean(options.preserveViewerTransform),
  );
  const carryoverImage = keepViewerTransform ? currentViewerMainImageCarryover() : null;
  state.selectedGroupId = group.group_id ?? null;
  state.selectedGroup = group;
  state.groupDetail = [];
  state.loupe = null;
  if (!keepViewerTransform) {
    state.viewerTransform = resetViewerTransform();
    state.viewerCarryoverImage = null;
  } else {
    state.viewerCarryoverImage = carryoverImage;
  }
  viewerDragState = null;
  render();
  const projectId = state.selectedProjectId;
  if (projectId && group.group_id) {
    const loaded = await withBusy("载入照片组", () =>
      Promise.all([
        api.getGroupDetail(projectId, group.group_id as string),
        api.getSubjectAssessments(projectId, [group.group_id as string]).catch(() => []),
      ]),
    );
    if (!loaded) return;
    const [detail, assessments] = loaded;
    state.groupDetail = detail ?? [];
    state.subjectAssessments = {
      ...state.subjectAssessments,
      [group.group_id]: assessments ?? [],
    };
    render();
  }
}

function currentViewerMainImageCarryover(): ViewerCarryoverImage | null {
  const preview = document.querySelector<HTMLElement>(".viewer-main-preview");
  if (!preview) {
    return state.viewerCarryoverImage;
  }
  const candidates = [
    ...Array.from(preview.querySelectorAll<HTMLImageElement>(":scope > img.viewer-carryover-image")).map((image) => ({
      url: image.currentSrc || image.src,
      loaded: image.complete && image.naturalWidth > 0,
      role: "carryover" as const,
    })),
    ...Array.from(preview.querySelectorAll<HTMLImageElement>(":scope > img.preview-image")).map((image) => ({
      url: image.currentSrc || image.src,
      loaded: image.complete && image.naturalWidth > 0,
      role: "preview" as const,
    })),
  ];
  const url = viewerCarryoverSource(candidates, state.viewerCarryoverImage?.url ?? null);
  return url ? { url } : null;
}

async function toggleFavorite(targetGroup = state.selectedGroup) {
  const projectId = state.selectedProjectId;
  const group = targetGroup;
  if (!projectId || !group?.group_id) {
    return;
  }
  await withBusy("保存收藏标记", () =>
    api.saveGroupUserMarks(projectId, group.group_id as string, !group.user_marks.favorite, null),
  );
  await refreshCurrentProject(false);
}

async function toggleMarked(targetGroup = state.selectedGroup) {
  const projectId = state.selectedProjectId;
  const group = targetGroup;
  if (!projectId || !group?.group_id) {
    return;
  }
  await withBusy("保存标记", () =>
    api.saveGroupUserMarks(projectId, group.group_id as string, null, !group.user_marks.marked),
  );
  await refreshCurrentProject(false);
}

async function deleteAssetGroup(targetGroup = state.selectedGroup) {
  const projectId = state.selectedProjectId;
  const group = targetGroup;
  if (!projectId || !group?.group_id) {
    return;
  }
  const fileCount = membersOf(group).length;
  const confirmed = window.confirm(
    `删除照片组 ${group.group_key}？\n\n这会从项目中移除记录，并删除 ${fileCount} 个原图文件。此操作不可撤销。`,
  );
  if (!confirmed) {
    return;
  }
  const deleted = await withBusy("删除照片组", () => api.deleteAssetGroup(projectId, group.group_id as string));
  if (!deleted) {
    await refreshCurrentProject(false);
    return;
  }
  purgePreviewCachesForGroup(group);
  if (group.group_id) {
    const remaining = { ...state.subjectAssessments };
    delete remaining[group.group_id];
    state.subjectAssessments = remaining;
  }
  state.selectedGroupId = null;
  state.selectedGroup = null;
  state.groupDetail = [];
  state.loupe = null;
  state.status = `已删除 ${group.group_key}`;
  await refreshCurrentProject(false);
}

function purgePreviewCachesForGroup(group: ReceivedAssetGroup) {
  const localPaths = membersOf(group)
    .map((asset) => localPreviewablePath(asset))
    .filter((path): path is string => Boolean(path));
  for (const localPath of localPaths) {
    const suffix = `:${localPath}`;
    for (const key of [...thumbnailUrlCache.keys()]) {
      if (key.endsWith(suffix)) {
        thumbnailUrlCache.delete(key);
      }
    }
    for (const key of [...thumbnailQueued.keys()]) {
      if (!key.endsWith(suffix)) {
        continue;
      }
      const item = thumbnailQueued.get(key);
      if (!item) {
        continue;
      }
      thumbnailQueued.delete(key);
      const index = thumbnailQueue.indexOf(item);
      if (index >= 0) {
        thumbnailQueue.splice(index, 1);
      }
      item.resolve(null);
    }
    for (const key of [...thumbnailBatchPending]) {
      if (key.endsWith(suffix)) {
        thumbnailBatchPending.delete(key);
      }
    }
    originalImageWarmCache.delete(convertFileSrc(localPath));
  }
}

async function runAnalysisJobs() {
  const projectId = state.selectedProjectId;
  if (!projectId) return;
  const summary = await withBusy("运行全局质检", () => api.runDesktopCvAssessment(projectId, 2000));
  if (summary) {
    state.status = `全局质检：${summary.assessed_count} 个照片组完成，${summary.failed_count} 失败，${summary.skipped_count} 跳过`;
    await refreshCurrentProject(false);
  }
}

async function recommendBurst(targetGroup = state.selectedGroup) {
  const burstId = targetGroup?.burst?.burst_group_id;
  if (!burstId) {
    setStatus("推荐连拍", "当前照片组不属于连拍。");
    return;
  }
  const recommendation = await withBusy("推荐连拍", () => api.recommendBurstGroup(burstId));
  if (recommendation) {
    state.status = `连拍推荐：${readable(recommendation.status)}`;
    await refreshCurrentProject(false);
  }
}

async function recommendProject() {
  const projectId = state.selectedProjectId;
  if (!projectId) {
    return;
  }
  const recommendation = await withBusy("生成全局推荐", () => api.generateProjectRecommendation(projectId));
  if (recommendation) {
    state.status = `全局推荐：${readable(recommendation.status)}`;
    await refreshCurrentProject(false);
  }
}

async function saveIntelligenceSettings(patch: Partial<ProjectEvaluationSettings>) {
  if (!state.selectedProjectId || !state.intelligenceSettings) return;
  const settings: ProjectEvaluationSettings = {
    ...state.intelligenceSettings,
    ...patch,
    project_id: state.selectedProjectId,
    project_recommendation_mode: "manual",
  };
  const saved = await withBusy("保存 AI 辅助设置", () => api.saveProjectEvaluationSettings(settings));
  if (saved) {
    state.intelligenceSettings = saved;
    state.status = "AI 辅助设置已保存";
    render();
  }
}

async function evaluateGroupWithModel(targetGroup = state.selectedGroup) {
  const projectId = state.selectedProjectId;
  const groupId = targetGroup?.group_id;
  if (!projectId || !groupId) return;
  const result = await withBusy("AI 评价", async () => {
    const enqueued = await api.enqueueModelEvaluation(projectId, [groupId]);
    await api.drainAnalysisJobs(12);
    return enqueued;
  });
  if (result) {
    state.status = result.enqueued_count ? `已加入 AI 评价：${result.enqueued_count} 个照片组` : "AI 评价已加入队列";
    await refreshCurrentProject(false);
  }
}

async function evaluateLoadedGroupsWithModel() {
  const projectId = state.selectedProjectId;
  const setup = currentIntelligenceSetup();
  const groupIds = filteredGroups()
    .map((group) => group.group_id)
    .filter((groupId): groupId is string => Boolean(groupId));
  if (!projectId || !setup.modelReady || !groupIds.length) return;
  const result = await withBusy("AI 评价", async () => {
    const enqueued = await api.enqueueModelEvaluation(projectId, groupIds);
    await api.drainAnalysisJobs(50);
    return enqueued;
  });
  if (result) {
    state.status = `已加入 AI 评价：${result.enqueued_count}/${groupIds.length} 个照片组`;
    await refreshCurrentProject(false);
  }
}

async function saveProviderDraft() {
  const draft = state.providerDraft;
  if (!draft || !providerDraftIsSaveable(draft)) return;
  const saved = await withBusy("保存 AI 服务", () => api.saveModelProviderSettings(draft));
  if (!saved) return;
  state.providerDraft = providerDraftFromSettings(saved);
  state.intelligenceProviders = await api.getModelProviderSettingsList();
  if (state.settingsPanel === "project" && state.intelligenceSettings?.model_provider_settings_id !== saved.settings_id) {
    await saveIntelligenceSettings({ model_provider_settings_id: saved.settings_id });
  } else {
    state.status = "AI 服务已保存";
    render();
  }
}

async function deleteProvider(settingsId: string) {
  const deleted = await withBusy("删除 AI 服务", () => api.deleteModelProviderSettings(settingsId));
  if (!deleted) return;
  state.providerDraft = null;
  state.intelligenceProviders = await api.getModelProviderSettingsList();
  if (state.intelligenceSettings?.model_provider_settings_id === settingsId) {
    await saveIntelligenceSettings({ model_provider_settings_id: null });
  } else {
    state.status = "AI 服务已删除";
    render();
  }
}

async function savePromptDraft() {
  const draft = state.promptDraft;
  if (!draft || !promptDraftIsSaveable(draft)) return;
  const styleTags = promptStyleTagsFromText(draft.style_tags_text);
  let saved: PromptPack | null = null;
  if (draft.mode === "create") {
    saved = await withBusy("创建选片规则包", () =>
      api.createGlobalPromptPack({
        name: draft.name,
        distribution_folder: draft.distribution_folder,
        style_tags: styleTags,
        scene_profile: draft.scene_profile,
        shared_preference: draft.shared_preference,
      }),
    );
  } else if (draft.mode === "fork" && draft.source_prompt_pack_id) {
    const forked = await withBusy("复制选片规则包", () =>
      api.forkGlobalPromptPack({
        source_prompt_pack_id: draft.source_prompt_pack_id as string,
        name: draft.name,
        distribution_folder: draft.distribution_folder,
      }),
    );
    if (forked) {
      saved = await withBusy("保存选片规则包", () =>
        api.saveGlobalPromptPack({
          prompt_pack_id: forked.prompt_pack_id,
          name: draft.name,
          style_tags: styleTags,
          scene_profile: draft.scene_profile,
          shared_preference: draft.shared_preference,
        }),
      );
    }
  } else if (draft.mode === "edit" && draft.prompt_pack_id) {
    saved = await withBusy("保存选片规则包", () =>
      api.saveGlobalPromptPack({
        prompt_pack_id: draft.prompt_pack_id as string,
        name: draft.name,
        style_tags: styleTags,
        scene_profile: draft.scene_profile,
        shared_preference: draft.shared_preference,
      }),
    );
  }
  if (!saved) return;
  await refreshPromptPackLists();
  state.promptDraft = promptDraftFromPack(saved);
  if (state.settingsPanel === "project" && state.intelligenceSettings?.prompt_pack_id !== saved.prompt_pack_id) {
    await saveIntelligenceSettings({ prompt_pack_id: saved.prompt_pack_id });
  } else {
    state.status = "选片规则包已保存";
    render();
  }
}

async function deletePromptPack(promptPackId: string) {
  const deleted = await withBusy("移除选片规则包", () => api.deleteGlobalPromptPack(promptPackId));
  if (!deleted) return;
  state.promptDraft = null;
  await refreshPromptPackLists();
  if (state.intelligenceSettings?.prompt_pack_id === promptPackId) {
    await saveIntelligenceSettings({ prompt_pack_id: null });
  } else {
    state.status = "选片规则包已移除";
    render();
  }
}

async function removeFromBurst(group: ReceivedAssetGroup) {
  const burstId = group.burst?.burst_group_id;
  if (!burstId || !group.group_id) {
    setStatus("移出连拍", "当前照片组不属于连拍。");
    return;
  }
  await withBusy("移出连拍", () => api.splitBurstMember(burstId, group.group_id as string));
  state.selectedGroupId = group.group_id;
  await refreshCurrentProject(false);
}

function render() {
  appRoot.replaceChildren(renderShell());
}

function renderShell() {
  return append(el("div", state.layoutMode === "viewer" ? "app-shell is-viewer-focus" : "app-shell"), renderTopBar(), renderWorkflow());
}

function renderTopBar() {
  if (state.layoutMode === "viewer") {
    const top = el("header", "topbar viewer-microbar");
    const summary = state.assetPage?.summary;
    const project = selectedProject();
    const source = state.rootPath ? state.rootPath.split(/[\\/]/).filter(Boolean).at(-1) : "未绑定文件夹";
    const status = el("div", state.error ? "viewer-status-pill is-error" : "viewer-status-pill", state.error ?? state.busy ?? state.status);
    append(
      top,
      append(
        el("div", "viewer-microbar-left"),
        commandButton("网格", "viewer-micro-button", () => {
          state.layoutMode = "grid";
          state.loupe = null;
          render();
        }),
        append(
          el("div", "viewer-micro-project"),
          el("span", "", project?.name ?? "未选择项目"),
          el("strong", "", source),
        ),
      ),
      append(
        el("div", "viewer-micro-context"),
        el("span", "", source),
        el("strong", "", `${summary?.group_count ?? 0} 组`),
        el("span", "", `${summary?.asset_count ?? 0} 文件`),
        renderScanProgressPill(),
        renderPreviewProgressPill(),
      ),
      status,
    );
    return top;
  }
  const top = el("header", "topbar");
  const status = el("div", state.error ? "status is-error" : "status", state.error ?? state.busy ?? state.status);
  append(top, renderProjectSwitcher(), renderTopContext(), status);
  return top;
}

function renderViewerLeftRail() {
  const side = el("aside", "project-sidebar viewer-left-rail");
  const summary = state.assetPage?.summary;
  const filters: Array<[SourceFilter, string]> = [
    ["all", "全部"],
    ["available", "可用"],
    ["changed", "已变化"],
    ["missing", "缺失"],
  ];
  append(
    side,
    commandButton("网格", "viewer-left-button", () => {
      state.layoutMode = "grid";
      state.loupe = null;
      render();
    }),
    commandButton("文件夹", "viewer-left-button", () => void chooseFolder(), Boolean(state.busy || !state.selectedProjectId)),
    append(el("div", "viewer-left-separator")),
  );
  for (const [filter, label] of filters) {
    const count =
      filter === "all"
        ? displayGroupsFor(allGroups()).length
        : displayGroupsFor(allGroups().filter((group) => sourceStatus(group) === filter)).length;
    const button = commandButton("", "viewer-left-filter", () => {
      state.sourceFilter = filter;
      state.viewFilter = filter === "missing" ? "missing" : "light-table";
      render();
    });
    button.title = `${label} ${count}`;
    if (state.sourceFilter === filter) button.classList.add("is-active");
    append(button, statusDot(filter === "all" ? "neutral" : filter), el("strong", "", String(count)));
    append(side, button);
  }
  append(side, append(el("div", "viewer-left-total"), el("strong", "", String(summary?.group_count ?? 0)), el("span", "", "组")));
  return side;
}

function renderProjectSwitcher() {
  const project = selectedProject();
  const wrap = el("div", "project-switcher-wrap");
  const chooser = el("div", "project-switcher");
  const trigger = commandButton("", "project-menu-trigger", () => {
    state.projectMenuOpen = !state.projectMenuOpen;
    state.projectCreatorOpen = false;
    render();
  });
  append(
    trigger,
    append(
      el("div", "switcher-copy"),
      el("span", "product-name", "相机连接器"),
      el("strong", "", project ? project.name : "未选择项目"),
    ),
    el("span", "switcher-caret", ""),
  );
  append(chooser, trigger);
  append(
    chooser,
    commandButton("新建", "new-project-button", () => {
      state.projectCreatorOpen = !state.projectCreatorOpen;
      state.projectMenuOpen = false;
      render();
    }),
    commandButton("设置", "global-settings-button", () => void openSettingsPanel("global")),
  );
  append(wrap, chooser);
  if (state.projectMenuOpen) {
    append(wrap, renderProjectMenu());
  }
  if (state.projectCreatorOpen) {
    append(wrap, append(el("div", "project-create-popover"), renderProjectCreate("compact")));
  }
  return wrap;
}

function renderProjectMenu() {
  const menu = el("div", "project-menu-popover");
  if (!state.projects.length) {
    append(menu, el("div", "project-menu-empty", "暂无项目"));
    return menu;
  }
  const list = el("div", "project-menu-list");
  for (const project of state.projects) {
    const item = commandButton("", "project-menu-item", () => void selectProject(project.project_id));
    if (project.project_id === state.selectedProjectId) item.classList.add("is-active");
    append(
      item,
      append(el("span", ""), el("strong", "", project.name), el("small", "", project.slug)),
      project.project_id === state.selectedProjectId ? el("span", "project-menu-meta", "当前") : null,
    );
    append(list, item);
  }
  append(menu, list);
  return menu;
}

function renderTopContext() {
  const summary = state.assetPage?.summary;
  const source = state.rootPath ? state.rootPath.split(/[\\/]/).filter(Boolean).at(-1) : "未绑定文件夹";
  const context = el("div", "top-context");
  append(
    context,
    el("span", "", source),
    el("strong", "", `${summary?.group_count ?? 0} 组`),
    el("span", "", `${summary?.asset_count ?? 0} 文件`),
    renderScanProgressPill(),
    renderPreviewProgressPill(),
  );
  return context;
}

function renderScanProgressPill() {
  const scan = scanProgressDisplay();
  const pill = el("span", `progress-pill ${scan.tone}`, scan.label);
  pill.title = scan.title;
  return pill;
}

function renderPreviewProgressPill() {
  const progress = currentPreviewProgress();
  const pill = el("span", "progress-pill preview-progress-pill", progress.label);
  pill.title = "高清预览进度。低清表示已经可看，高清或原图表示当前预览完成。";
  pill.dataset.previewProgress = "true";
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
    return { label: "扫描待开始", title: "绑定文件夹后开始扫描", tone: "idle" };
  }
  if (scanIsActive(scan?.phase)) {
    return {
      label: `扫描 ${scan?.files_seen ?? 0} 文件 / ${scan?.groups_updated ?? 0} 组`,
      title: `扫描阶段：${readable(scan?.phase ?? "pending")}，已发现 ${scan?.files_seen ?? 0} 个文件，已索引 ${scan?.assets_indexed ?? 0} 个照片文件`,
      tone: "working",
    };
  }
  if (scan?.phase === "failed") {
    return {
      label: `扫描失败，保留 ${summary?.group_count ?? scan.groups_updated} 组`,
      title: compactError(scan.error ?? null) ?? "扫描失败，当前索引仍可用",
      tone: "failed",
    };
  }
  return {
    label: `扫描完成 ${summary?.group_count ?? scan?.groups_updated ?? 0} 组`,
    title: `已索引 ${summary?.asset_count ?? scan?.assets_indexed ?? 0} 个照片文件`,
    tone: "ready",
  };
}

function currentPreviewProgress() {
  return previewProgress(displayGroupsFor(allGroups()).map((group) => previewStageForGroup(group, currentThumbnailMaxEdge())));
}

function renderWorkflow() {
  const layout = el("main", state.layoutMode === "viewer" ? "workflow-layout is-viewer-focus" : "workflow-layout");
  append(layout, renderWorkbenchSidebar(), renderWorkbenchSurface());
  if (state.settingsPanel) {
    append(layout, renderSettingsDrawer(state.settingsPanel));
  }
  return layout;
}

function renderWorkbenchSidebar() {
  if (state.layoutMode === "viewer") {
    return renderViewerLeftRail();
  }
  const side = el("aside", "project-sidebar");
  append(side, renderSourcePanel(), renderIntelligencePanel(), renderTransferPanel(), renderViewsPanel(), renderFiltersPanel());
  return side;
}

function renderProjectCreate(variant: "compact" | "hero" = "compact") {
  const row = el("form", `project-create project-create-${variant}`);
  row.addEventListener("submit", (event) => {
    event.preventDefault();
    void createProject();
  });
  const folderName = state.projectFolderDraft ? folderBasename(state.projectFolderDraft) : "选择照片文件夹";
  const folderPicker = commandButton("", state.projectFolderDraft ? "project-folder-picker has-folder" : "project-folder-picker", () => void chooseProjectFolderDraft(), Boolean(state.busy));
  folderPicker.title = state.projectFolderDraft || "选择本地照片文件夹";
  append(
    folderPicker,
    el("span", "project-folder-kicker", "照片文件夹"),
    el("strong", "", folderName),
    el("small", "", state.projectFolderDraft || "递归包含子文件夹"),
  );
  append(
    row,
    textInput(state.projectNameDraft, "项目名称", (value) => {
      state.projectNameDraft = value;
    }),
    folderPicker,
    append(
      el("div", "project-create-actions"),
      commandButton("创建并索引", variant === "hero" ? "primary large" : "primary", () => void createProject(), Boolean(state.busy)),
    ),
  );
  return row;
}

function renderSourcePanel() {
  const box = el("section", "side-section source-section");
  const canChoose = Boolean(state.selectedProjectId && !state.busy);
  const blocker = getScanStartBlocker();
  append(
    box,
    append(
      el("div", "side-section-head"),
      el("h3", "", "文件夹"),
      commandButton(state.rootPath ? "更换" : "选择", "side-link", () => void chooseFolder(), !canChoose),
    ),
    append(
      el("div", "source-path-row"),
      el("span", "source-folder-icon", ""),
      el("div", "path-readout", state.rootPath || "未选择文件夹"),
    ),
    el("p", "side-note", state.rootPath ? "递归索引，包含所有子文件夹。" : "为当前项目绑定本地照片文件夹。"),
  );
  if (state.rootPath) {
    append(
      box,
      commandButton(state.scan?.assets_indexed ? "重新扫描" : "扫描文件夹", "source-action", () => void startScan(), Boolean(blocker)),
      commandButton("同步局域网项目", "source-action", () => void syncLanProjectContext(true), Boolean(blocker)),
    );
  }
  return box;
}

function renderIntelligencePanel() {
  if (!state.selectedProjectId) return null;
  const setup = currentIntelligenceSetup();
  const box = el("section", "side-section intelligence-section");
  append(
    box,
    append(
      el("div", "side-section-head"),
      el("h3", "", "AI 辅助"),
      commandButton("设置", "side-link", () => {
        void openSettingsPanel("project");
      }),
    ),
    append(
      el("div", `intelligence-status ${setup.modelReady ? "is-ready" : "needs-setup"}`),
      statusDot(setup.modelReady ? "available" : "changed"),
      el("strong", "", readable(intelligenceStatusLabel(setup))),
    ),
    append(
      el("div", "intelligence-lines"),
      intelligenceLine("AI 服务", setup.selectedProvider?.provider_label ?? "未选择"),
      intelligenceLine("选片规则", setup.selectedPrompt?.name ?? "未选择"),
      intelligenceLine("场景", readable(state.intelligenceSettings?.scene_profile ?? "general")),
      intelligenceLine("自动 AI 评价", setup.autoEvaluate ? "开启" : "关闭"),
    ),
  );
  if (state.selectedGroup?.group_id) {
    append(
      box,
      commandButton("AI 评价当前组", "source-action", () => void evaluateGroupWithModel(), Boolean(state.busy || !setup.modelReady)),
    );
  }
  return box;
}

function renderSettingsDrawer(panelKind: SettingsPanel) {
  const backdrop = el("div", "drawer-backdrop");
  backdrop.addEventListener("click", (event) => {
    if (event.target === backdrop) {
      state.settingsPanel = null;
      render();
    }
  });
  return panelKind === "global" ? renderGlobalSettingsDrawer(backdrop) : renderProjectSettingsDrawer(backdrop);
}

function renderProjectSettingsDrawer(backdrop: HTMLElement) {
  const settings = state.intelligenceSettings;
  const setup = currentIntelligenceSetup();
  const panel = el("section", "intelligence-drawer");
  append(
    panel,
    append(
      el("div", "drawer-head"),
      append(
        el("div", ""),
        el("p", "eyebrow", "项目设置"),
        el("h2", "", "AI 辅助选片"),
      ),
      commandButton("关闭", "ghost", () => {
        state.settingsPanel = null;
        render();
      }),
    ),
  );

  if (!settings) {
    append(panel, el("p", "side-note", "先选择一个项目，再配置 AI 辅助选片。"));
    append(backdrop, panel);
    return backdrop;
  }

  append(
    panel,
    append(
      el("section", "settings-section"),
      settingsSectionHead("基础配置"),
      append(
        el("div", "settings-field-grid"),
        renderIntelligenceField(
          "AI 服务",
          selectControl(
            settings.model_provider_settings_id ?? "",
            [["", "不使用 AI 服务"], ...state.intelligenceProviders.map((provider) => [provider.settings_id, provider.provider_label] as [string, string])],
            (value) => void saveIntelligenceSettings({ model_provider_settings_id: value || null }),
          ),
        ),
        renderIntelligenceField(
          "选片规则",
          selectControl(
            settings.prompt_pack_id ?? "",
            [["", "不绑定选片规则"], ...state.promptPacks.map((prompt) => [prompt.prompt_pack_id, prompt.name] as [string, string])],
            (value) => void saveIntelligenceSettings({ prompt_pack_id: value || null }),
          ),
        ),
        renderIntelligenceField(
          "拍摄场景",
          selectControl(
            settings.scene_profile,
            [
              ["general", "通用"],
              ["portrait", "人像"],
              ["action", "运动"],
              ["landscape", "风光"],
              ["custom", "自定义"],
            ],
            (value) => void saveIntelligenceSettings({ scene_profile: value }),
          ),
        ),
      ),
    ),
    append(
      el("section", "settings-section"),
      settingsSectionHead("质量风险"),
      renderCvThresholdSettings(settings),
    ),
    append(
      el("section", "settings-section"),
      settingsSectionHead("自动化"),
      append(
        el("div", "settings-toggle-list"),
        renderToggleRow("扫描后自动 AI 评价", settings.auto_evaluate_on_upload, (checked) =>
          void saveIntelligenceSettings({ auto_evaluate_on_upload: checked }),
        ),
        renderToggleRow("自动生成连拍推荐", settings.auto_burst_recommendation_enabled, (checked) =>
          void saveIntelligenceSettings({ auto_burst_recommendation_enabled: checked }),
        ),
        renderToggleRow("允许 AI 选中有风险照片", settings.allow_risky_model_selects, (checked) =>
          void saveIntelligenceSettings({ allow_risky_model_selects: checked }),
        ),
      ),
    ),
    renderLoadedModelEvaluationPanel(setup),
  );

  append(backdrop, panel);
  return backdrop;
}

function renderCvThresholdSettings(settings: ProjectEvaluationSettings) {
  const mode = selectedCvThresholdMode(settings);
  const panel = el("section", "cv-threshold-panel");
  append(
    panel,
    renderIntelligenceField(
      "阈值方案",
      selectControl(
        mode,
        [
          ["loose", "宽松"],
          ["standard", "标准"],
          ["strict", "严格"],
          ["custom", "自定义"],
        ],
        (value) => {
          const next = settingsForCvThresholdMode(settings, value as CvThresholdMode);
          void saveIntelligenceSettings({
            cv_policy: next.cv_policy,
            cv_policy_overrides: next.cv_policy_overrides ?? null,
          });
        },
      ),
    ),
  );
  if (!settings.cv_policy_overrides) {
    return panel;
  }
  const policy = settings.cv_policy_overrides ?? technicalPolicyForCvPolicy(settings.cv_policy);
  const controls = cvThresholdControlSpecs(policy, settings.scene_profile);
  append(
    panel,
    append(
      el("div", "cv-threshold-head"),
      el("strong", "", "自定义参数"),
      commandButton("恢复预设", "micro-button", () =>
        void saveIntelligenceSettings({
          cv_policy_overrides: technicalPolicyForCvPolicy(settings.cv_policy),
        }),
      ),
    ),
  );
  for (const control of controls) {
    append(panel, renderCvThresholdControl(settings, policy, control));
  }
  return panel;
}

function renderCvThresholdControl(
  settings: ProjectEvaluationSettings,
  policy: NonNullable<ProjectEvaluationSettings["cv_policy_overrides"]>,
  control: ReturnType<typeof cvThresholdControlSpecs>[number],
) {
  const input = el("input", "cv-threshold-slider") as HTMLInputElement;
  input.type = "range";
  input.min = "0";
  input.max = "1";
  input.step = "0.01";
  input.value = String(control.sliderValue);
  input.addEventListener("change", () => {
    const nextPolicy = updateCvThresholdControl(policy, control.key, Number(input.value));
    void saveIntelligenceSettings({
      cv_policy: settings.cv_policy,
      cv_policy_overrides: nextPolicy,
    });
  });
  return append(
    el("label", "cv-threshold-row"),
    append(
      el("div", "cv-threshold-copy"),
      append(el("span", ""), el("strong", "", control.title), el("em", "", control.displayLabel)),
    ),
    input,
  );
}

function renderGlobalSettingsDrawer(backdrop: HTMLElement) {
  const panel = el("section", "intelligence-drawer is-global-settings");
  append(
    panel,
    append(
      el("div", "drawer-head"),
      append(
        el("div", ""),
        el("p", "eyebrow", "全局设置"),
        el("h2", "", "AI 与选片规则"),
      ),
      commandButton("关闭", "ghost", () => {
        state.settingsPanel = null;
        render();
      }),
    ),
    append(el("div", "global-settings-grid"), renderProviderManagement("global"), renderPromptManagement("global")),
  );
  append(backdrop, panel);
  return backdrop;
}

function renderLoadedModelEvaluationPanel(setup = currentIntelligenceSetup()) {
  const groupIds = filteredGroups()
    .map((group) => group.group_id)
    .filter((groupId): groupId is string => Boolean(groupId));
  return append(
    el("section", "drawer-section"),
    append(el("div", "drawer-section-head"), el("h3", "", "执行"), el("span", "", `${groupIds.length} 组`)),
    commandButton("评价当前视图", "source-action", () => void evaluateLoadedGroupsWithModel(), Boolean(state.busy || !setup.modelReady || !groupIds.length)),
  );
}

function renderProviderManagement(scope: SettingsPanel = "project") {
  const projectScope = scope === "project";
  const section = el("section", "drawer-section");
  append(
    section,
    append(
      el("div", "drawer-section-head"),
      el("h3", "", "AI 服务"),
      commandButton("新建服务", "side-link", () => {
        state.promptDraft = null;
        state.providerDraft = providerDraftFromSettings(null);
        render();
      }),
    ),
  );
  const list = el("div", "management-list");
  if (!state.intelligenceProviders.length) {
    append(list, el("p", "side-note", "暂无服务"));
  }
  for (const provider of state.intelligenceProviders) {
    const selected = state.intelligenceSettings?.model_provider_settings_id === provider.settings_id;
    const row = el("div", selected ? "management-row is-selected" : "management-row");
    append(
      row,
      append(
        el("div", "management-copy"),
        el("strong", "", provider.provider_label || provider.settings_id),
        el("span", "", `${readable(provider.provider_kind)} / ${provider.default_model || "未填写模型名称"}${provider.api_key_configured ? "" : " / 未设置密钥"}`),
      ),
      projectScope
        ? commandButton(selected ? "已选" : "选用", "micro-button", () => void saveIntelligenceSettings({ model_provider_settings_id: provider.settings_id }), Boolean(state.busy || selected))
        : selected
          ? el("span", "management-tag", "当前项目")
          : null,
      commandButton("编辑", "micro-button", () => {
        state.promptDraft = null;
        state.providerDraft = providerDraftFromSettings(provider);
        render();
      }),
    );
    append(list, row);
  }
  append(section, list);
  if (state.providerDraft) {
    append(section, renderProviderDraftForm(state.providerDraft));
  }
  return section;
}

function renderProviderDraftForm(draft: SaveModelProviderSettingsRequest) {
  const form = el("div", "management-editor");
  let saveButton: HTMLButtonElement | null = null;
  const updateSaveState = () => {
    if (!saveButton) return;
    saveButton.disabled = Boolean(state.busy || !state.providerDraft || !providerDraftIsSaveable(state.providerDraft));
  };
  const updateDraft = (patch: Partial<SaveModelProviderSettingsRequest>) => {
    state.providerDraft = { ...(state.providerDraft ?? draft), ...patch };
    updateSaveState();
  };
  append(
    form,
    renderIntelligenceField("ID", textInput(draft.settings_id, "global", (value) => {
      updateDraft({ settings_id: value });
    })),
    renderIntelligenceField("名称", textInput(draft.provider_label, "OpenAI", (value) => {
      updateDraft({ provider_label: value });
    })),
    renderIntelligenceField(
      "服务类型",
      selectControl(
        draft.provider_kind,
        [
          ["openai", "OpenAI"],
          ["custom", "兼容 OpenAI 接口"],
          ["none", "不启用"],
        ],
        (value) => {
          updateDraft({ provider_kind: value });
        },
      ),
    ),
    renderIntelligenceField("Base URL", textInput(draft.base_url, "https://api.openai.com/v1", (value) => {
      updateDraft({ base_url: value });
    })),
    renderIntelligenceField("模型", textInput(draft.default_model, "gpt-5-mini", (value) => {
      updateDraft({ default_model: value });
    })),
    renderIntelligenceField("API Key", passwordInput(draft.api_key ?? "", "留空表示保留已有密钥", (value) => {
      updateDraft({ api_key: value || null });
    })),
    renderIntelligenceField("密钥别名", textInput(draft.key_alias ?? "", "OPENAI_API_KEY", (value) => {
      updateDraft({ key_alias: value || null });
    })),
    renderIntelligenceField("图片长边", numberInput(draft.default_max_image_side, 256, 4096, (value) => {
      updateDraft({ default_max_image_side: value });
    })),
    renderIntelligenceField("批量数量", numberInput(draft.default_batch_size, 1, 32, (value) => {
      updateDraft({ default_batch_size: value });
    })),
    renderToggleRow("启用该服务", draft.configured, (checked) => {
      updateDraft({ configured: checked });
    }),
    append(
      el("div", "editor-actions"),
      (saveButton = commandButton("保存服务", "primary", () => void saveProviderDraft(), Boolean(state.busy || !providerDraftIsSaveable(draft)))),
      state.intelligenceProviders.some((provider) => provider.settings_id === draft.settings_id)
        ? commandButton("删除服务", "secondary danger-text", () => void deleteProvider(draft.settings_id), Boolean(state.busy))
        : null,
      commandButton("取消", "secondary", () => {
        state.providerDraft = null;
        render();
      }),
    ),
  );
  return form;
}

function renderPromptManagement(scope: SettingsPanel = "project") {
  const projectScope = scope === "project";
  const packs = projectScope ? state.promptPacks : state.globalPromptPacks;
  const section = el("section", "drawer-section");
  append(
    section,
    append(
      el("div", "drawer-section-head"),
      el("h3", "", "选片规则"),
      commandButton("新建", "side-link", () => {
        state.providerDraft = null;
        state.promptDraft = newPromptDraft();
        render();
      }),
    ),
  );
  const list = el("div", "management-list prompt-list");
  if (!packs.length) {
    append(list, el("p", "side-note", "还没有可用选片规则。"));
  }
  for (const prompt of packs) {
    const selected = state.intelligenceSettings?.prompt_pack_id === prompt.prompt_pack_id;
    const row = el("div", selected ? "management-row is-selected" : "management-row");
    append(
      row,
      append(
        el("div", "management-copy"),
        el("strong", "", prompt.name),
        el("span", "", `${prompt.built_in ? "内置" : "用户"} / ${readable(prompt.scene_profile)} / ${prompt.distribution_folder}`),
      ),
      projectScope
        ? commandButton(selected ? "已选" : "选用", "micro-button", () => void saveIntelligenceSettings({ prompt_pack_id: prompt.prompt_pack_id }), Boolean(state.busy || selected))
        : selected
          ? el("span", "management-tag", "当前项目")
          : null,
      commandButton(prompt.built_in ? "复制" : "编辑", "micro-button", () => {
        state.providerDraft = null;
        state.promptDraft = promptDraftFromPack(prompt);
        render();
      }),
    );
    append(list, row);
  }
  append(section, list);
  if (state.promptDraft) {
    append(section, renderPromptDraftForm(state.promptDraft));
  }
  return section;
}

function renderPromptDraftForm(draft: PromptDraft) {
  const form = el("div", "management-editor prompt-editor");
  let saveButton: HTMLButtonElement | null = null;
  const updateSaveState = () => {
    if (!saveButton) return;
    saveButton.disabled = Boolean(state.busy || !state.promptDraft || !promptDraftIsSaveable(state.promptDraft));
  };
  const updateDraft = (patch: Partial<PromptDraft>) => {
    state.promptDraft = { ...(state.promptDraft ?? draft), ...patch };
    updateSaveState();
  };
  append(
    form,
    append(el("div", "editor-kicker"), el("strong", "", promptDraftModeLabel(draft.mode))),
    renderIntelligenceField("名称", textInput(draft.name, "例如：婚礼纪实精选", (value) => {
      updateDraft({ name: value });
    })),
    renderIntelligenceField("文件夹", textInput(draft.distribution_folder, "user", (value) => {
      updateDraft({ distribution_folder: value });
    })),
    renderIntelligenceField(
      "拍摄场景",
      selectControl(
        draft.scene_profile,
        [
          ["general", "通用"],
          ["portrait", "人像"],
          ["action", "运动"],
          ["landscape", "风光"],
          ["custom", "自定义"],
        ],
        (value) => {
          updateDraft({ scene_profile: value });
        },
      ),
    ),
    renderIntelligenceField("风格标签", textInput(draft.style_tags_text, "婚礼 人像 纪实", (value) => {
      updateDraft({ style_tags_text: value });
    })),
    renderIntelligenceField("选片偏好", textAreaInput(draft.shared_preference, "写下选片偏好、优先级和淘汰规则。", (value) => {
      updateDraft({ shared_preference: value });
    })),
    append(
      el("div", "editor-actions"),
      (saveButton = commandButton(draft.mode === "edit" ? "保存" : draft.mode === "fork" ? "复制" : "创建", "primary", () => void savePromptDraft(), Boolean(state.busy || !promptDraftIsSaveable(draft)))),
      draft.mode === "edit" && draft.prompt_pack_id
        ? commandButton("移除本地包", "secondary danger-text", () => void deletePromptPack(draft.prompt_pack_id as string), Boolean(state.busy))
        : null,
      commandButton("取消", "secondary", () => {
        state.promptDraft = null;
        render();
      }),
    ),
  );
  return form;
}

function renderTransferPanel() {
  const scan = state.scan;
  const summary = state.assetPage?.summary;
  const box = el("section", "side-section");
  append(box, el("h3", "", "扫描记录"));
  if (!scan && !summary?.asset_count) {
    append(box, el("p", "side-note", "当前项目还没有索引记录。"));
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
    compactMetric("文件", String(transfer.files)),
    compactMetric("照片组", String(transfer.groups)),
    compactMetric("照片文件", String(transfer.assets)),
  );
  if (state.lanSyncPhase !== "idle" || state.lanSyncSummary) {
    append(
      box,
      append(
        el("div", "transfer-title"),
        el("strong", "", "project-sync"),
        statusDot(lanSyncTransferDot()),
        el("span", "", lanSyncTransferLabel()),
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

function renderViewsPanel() {
  const box = el("section", "side-section");
  append(box, el("h3", "", "视图"));
  const views: Array<[ViewFilter, string, number]> = [
    ["light-table", "选片台", displayGroupsFor(allGroups()).length],
    ["needs-work", "待处理", displayGroupsFor(allGroups().filter(needsWork)).length],
    ["missing", "缺失", displayGroupsFor(allGroups().filter((group) => sourceStatus(group) === "missing")).length],
  ];
  for (const [view, label, count] of views) {
    const item = commandButton("", "view-item", () => {
      state.viewFilter = view;
      if (view === "missing") state.sourceFilter = "missing";
      if (view === "light-table") state.sourceFilter = "all";
      render();
    });
    if (state.viewFilter === view) {
      item.classList.add("is-active");
    }
    append(item, el("span", "view-dot", ""), el("span", "", label), el("strong", "", String(count)));
    append(box, item);
  }
  return box;
}

function renderFiltersPanel() {
  const box = el("section", "side-section filters-section");
  append(box, el("h3", "", "文件状态"));
  const filters: Array<[SourceFilter, string]> = [
    ["all", "全部状态"],
    ["available", "可用"],
    ["changed", "已变化"],
    ["missing", "缺失"],
  ];
  for (const [filter, label] of filters) {
    const item = commandButton("", "filter-row", () => {
      state.sourceFilter = filter;
      state.viewFilter = filter === "missing" ? "missing" : "light-table";
      render();
    });
    if (state.sourceFilter === filter) item.classList.add("is-active");
    append(
      item,
      statusDot(filter === "all" ? "neutral" : filter),
      el("span", "", label),
      el("strong", "", String(filter === "all" ? displayGroupsFor(allGroups()).length : displayGroupsFor(allGroups().filter((group) => sourceStatus(group) === filter)).length)),
    );
    append(box, item);
  }
  return box;
}

function renderWorkbenchSurface() {
  const surface = el("section", "stage-surface review-surface");
  append(surface, renderReviewStage());
  return surface;
}

function renderReviewStage() {
  const inspectorGroup = state.layoutMode === "grid" ? state.selectedGroup : null;
  const showInspector = Boolean(inspectorGroup);
  const wrap = el("div", showInspector ? "review-stage has-inspector" : "review-stage");
  append(wrap, renderReviewMain());
  if (inspectorGroup) {
    append(wrap, renderInspector(inspectorGroup));
  }
  return wrap;
}

function renderReviewMain() {
  const main = el("section", "review-main");
  if (state.layoutMode !== "viewer") {
    append(main, renderReviewHeader());
  }
  append(main, renderLightTable());
  return main;
}

function renderReviewHeader() {
  const header = el("div", "review-header");
  const summary = state.assetPage?.summary;
  const hasProject = Boolean(state.selectedProjectId);
  const hasSource = Boolean(state.rootPath);
  const hasGroups = Boolean(summary?.group_count);
  const copy = append(
    el("div", "review-title"),
    el("h1", "", hasProject ? "选片台" : "创建选片项目"),
  );
  const actions = el("div", "review-actions");
  if (!hasProject) {
    // The project creation card owns the first-run action.
  } else if (!hasSource) {
    append(actions, commandButton("选择文件夹", "primary", () => void chooseFolder(), Boolean(state.busy)));
  } else {
    append(
      actions,
      commandButton("全局质检", "secondary", () => void runAnalysisJobs(), Boolean(state.busy || !hasGroups)),
      commandButton("全局推荐", "primary", () => void recommendProject(), Boolean(state.busy || !hasGroups)),
    );
  }
  append(header, copy, actions);
  return header;
}

function renderLightTable() {
  const shell = el("section", state.layoutMode === "viewer" ? "light-table is-viewer" : "light-table");
  shell.addEventListener("wheel", handleLightTableWheel, { passive: false });
  if (!state.selectedProjectId || !state.rootPath) {
    append(shell, renderWorkbenchEmptyState());
  } else if (state.layoutMode === "viewer") {
    append(shell, renderViewerMode());
  } else {
    append(shell, renderLightTableToolbar(), renderGroupBoard());
  }
  if (state.loupe) {
    append(shell, renderLoupeOverlay());
  }
  return shell;
}

function handleLightTableWheel(event: WheelEvent) {
  const target = event.target as HTMLElement | null;
  if (target?.closest("button, input, select, textarea, .size-control")) {
    return;
  }
  const board = (event.currentTarget as HTMLElement).querySelector<HTMLElement>(".group-board");
  if (!board) {
    return;
  }

  const deltaY = normalizedWheelDelta(event.deltaY, event.deltaMode, board.clientHeight);
  const deltaX = normalizedWheelDelta(event.deltaX, event.deltaMode, board.clientWidth);
  const maxTop = Math.max(0, board.scrollHeight - board.clientHeight);
  const maxLeft = Math.max(0, board.scrollWidth - board.clientWidth);
  const nextTop = clamp(board.scrollTop + deltaY, 0, maxTop);
  const nextLeft = clamp(board.scrollLeft + deltaX, 0, maxLeft);

  if (nextTop !== board.scrollTop || nextLeft !== board.scrollLeft) {
    board.scrollTop = nextTop;
    board.scrollLeft = nextLeft;
    handleGroupBoardScroll(board);
    event.preventDefault();
  }
}

function normalizedWheelDelta(delta: number, mode: number, pageSize: number) {
  if (mode === WheelEvent.DOM_DELTA_LINE) return delta * 18;
  if (mode === WheelEvent.DOM_DELTA_PAGE) return delta * pageSize;
  return delta;
}

function renderLightTableToolbar() {
  const toolbar = el("div", "light-table-toolbar");
  append(
    toolbar,
    renderSourceTabs(),
    append(
      el("div", "table-controls"),
      el("span", "", "排序：拍摄时间"),
      commandButton("网格", `tool-toggle${state.layoutMode === "grid" ? " is-active" : ""}`, () => {
        state.layoutMode = "grid";
        state.loupe = null;
        render();
      }),
      commandButton("查看器", `tool-toggle${state.layoutMode === "viewer" ? " is-active" : ""}`, () => {
        state.layoutMode = "viewer";
        state.loupe = null;
        state.viewerTransform = resetViewerTransform();
        viewerDragState = null;
        render();
      }),
      state.layoutMode === "grid"
        ? append(el("label", "size-control"), el("span", "", "尺寸"), renderSizeRange())
        : null,
    ),
  );
  return toolbar;
}

function renderSizeRange() {
  const input = el("input", "size-range") as HTMLInputElement;
  input.type = "range";
  input.min = "220";
  input.max = "420";
  input.step = "10";
  input.value = String(state.thumbSize);
  input.addEventListener("input", () => {
    state.thumbSize = Number(input.value);
    lastVirtualSignature = "";
    const board = document.querySelector<HTMLElement>(".group-board");
    if (board) {
      board.style.setProperty("--thumb-size", `${state.thumbSize}px`);
      updateVirtualBoard(board);
    }
  });
  return input;
}

function renderSourceTabs() {
  const tabs = el("div", "source-tabs");
  const filters: Array<[SourceFilter, string]> = [
    ["all", "全部"],
    ["available", "可用"],
    ["changed", "已变化"],
    ["missing", "缺失"],
  ];
  for (const [filter, label] of filters) {
    const count =
      filter === "all"
        ? displayGroupsFor(allGroups()).length
        : displayGroupsFor(allGroups().filter((group) => sourceStatus(group) === filter)).length;
    const tab = commandButton(`${label} ${count}`, "source-tab", () => {
      state.sourceFilter = filter;
      state.selectedGroupId = null;
      state.selectedGroup = null;
      state.groupDetail = [];
      resetBoardViewport();
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
  board.style.setProperty("--thumb-size", `${state.thumbSize}px`);
  if (!groups.length) {
    append(board, renderWorkbenchEmptyState());
    return board;
  }
  board.addEventListener("scroll", () => handleGroupBoardScroll(board), { passive: true });
  const spacer = el("div", "virtual-board-spacer");
  const windowNode = el("div", "virtual-board-window");
  append(spacer, windowNode);
  append(board, spacer);
  renderVirtualWindow(board, groups, virtualMetricsForBoard(board, groups.length));
  requestAnimationFrame(() => {
    if (!board.isConnected) return;
    board.scrollTop = state.boardScrollTop;
    state.boardWidth = board.clientWidth;
    lastVirtualSignature = "";
    updateVirtualBoard(board);
  });
  return board;
}

function renderViewerMode() {
  const groups = filteredGroups();
  const current = viewerCurrentGroup(groups, state.selectedGroupId);
  const viewer = el("section", `viewer-mode${state.viewerInspectorOpen ? " inspector-open" : ""}${state.viewerFilmstripCollapsed ? " filmstrip-collapsed" : ""}`);
  if (!current) {
    append(viewer, renderWorkbenchEmptyState());
    return viewer;
  }
  const hasBurstQueue = burstMembersOf(current).length > 1;
  if (!hasBurstQueue) {
    viewer.classList.add("no-burst");
  }

  const queue = viewerQueueWindow(groups, current, 10);
  const burstWarmQueue = viewerBurstWarmWindow(allGroups(), current);
  const warmQueue = uniqueGroupsByIdentity([...burstWarmQueue, ...queue]);
  warmThumbnailsForGroups(warmQueue);
  warmOriginalsForGroups(warmQueue);
  append(
    viewer,
    renderViewerStage(current, groups),
    state.viewerInspectorOpen ? renderViewerInspector(current, groups) : renderViewerRightRail(current),
    renderViewerFilmstrip(groups, current),
  );
  return viewer;
}

function renderViewerScorebar(group: ReceivedAssetGroup, groups: ReceivedAssetGroup[]) {
  const index = Math.max(0, groups.findIndex((candidate) => groupIdentity(candidate) === groupIdentity(group))) + 1;
  const rail = el("section", "viewer-scorebar");
  append(
    rail,
    append(
      el("div", "viewer-identity"),
      append(
        el("div", "viewer-kicker"),
        el("span", "", `${index} / ${groups.length}`),
        el("span", "", formatPairLabel(group)),
        group.burst ? el("span", "", `${group.burst.member_count} 张连拍`) : null,
      ),
      el("h2", "", group.group_key),
    ),
    append(
      el("div", "viewer-score-grid"),
      viewerScoreMetric("文件", readable(sourceStatus(group)), sourceStatus(group)),
      viewerScoreMetric("质量", readable(group.technical_gate_status ?? group.technical_status ?? "pending"), group.technical_gate_status ?? group.technical_status ?? "pending"),
      viewerScoreMetric("AI", modelLabel(group), group.model_status ?? "pending"),
      viewerScoreMetric("推荐", readable(group.burst?.recommendation_status ?? "not generated"), group.burst?.recommendation_status ?? "pending"),
    ),
  );
  return rail;
}

function renderViewerRightRail(group: ReceivedAssetGroup) {
  const rail = el("aside", "viewer-right-rail");
  const metrics: Array<[string, string]> = [
    ["文件", sourceStatus(group)],
    ["质量", group.technical_gate_status ?? group.technical_status ?? "pending"],
    ["AI", group.model_status ?? "pending"],
    ["推荐", group.burst?.recommendation_status ?? "pending"],
  ];
  append(
    rail,
    commandButton("详情", "viewer-right-toggle", () => {
      state.viewerInspectorOpen = true;
      render();
    }),
  );
  for (const [label, status] of metrics) {
    const item = el("div", "viewer-right-dot");
    item.title = `${label}: ${readable(status)}`;
    append(item, statusDot(status), el("span", "", label.slice(0, 1)));
    append(rail, item);
  }
  return rail;
}

function renderViewerInspector(group: ReceivedAssetGroup, groups: ReceivedAssetGroup[]) {
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
      commandButton("关闭", "viewer-inspector-close", () => {
        state.viewerInspectorOpen = false;
        render();
      }),
    ),
    append(
      el("div", "viewer-inspector-status"),
      viewerScoreMetric("文件", readable(sourceStatus(group)), sourceStatus(group)),
      viewerScoreMetric("质量", readable(group.technical_gate_status ?? group.technical_status ?? "pending"), group.technical_gate_status ?? group.technical_status ?? "pending"),
      viewerScoreMetric("AI", modelLabel(group), group.model_status ?? "pending"),
      viewerScoreMetric("推荐", readable(group.burst?.recommendation_status ?? "not generated"), group.burst?.recommendation_status ?? "pending"),
    ),
    renderViewerBurstSummary(group),
    renderEvaluationPanel(group),
  );
  return panel;
}

function renderViewerBurstSummary(group: ReceivedAssetGroup) {
  const members = burstMembersOf(group);
  const panel = el("section", "viewer-inspector-panel");
  append(panel, append(el("div", "viewer-section-label"), el("strong", "", "连拍"), el("span", "", `${selectedBurstIndex(group, members)} / ${members.length}`)));
  if (members.length <= 1) {
    append(panel, el("div", "empty-note", "单张拍摄"));
    return panel;
  }
  const frames = el("div", "viewer-inspector-burst");
  members.slice(0, 8).forEach((member, index) => {
    const frame = commandButton("", "viewer-inspector-frame", () =>
      void selectGroup(member, { preserveViewerTransform: true }),
    );
    frame.title = previewTooltipForGroup(member);
    appendPreviewImage(frame, member, { maxEdge: currentThumbnailMaxEdge() });
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

function renderViewerBurstQueue(group: ReceivedAssetGroup) {
  const members = burstMembersOf(group);
  if (members.length <= 1) {
    return null;
  }
  const strip = el("section", "viewer-burst-strip");
  strip.title = `连拍 ${selectedBurstIndex(group, members)} / ${members.length}`;
  const frames = el("div", "viewer-burst-frames");
  members.forEach((member, index) => {
    const frame = commandButton("", "viewer-burst-frame", () =>
      void selectGroup(member, { preserveViewerTransform: true }),
    );
    frame.title = previewTooltipForGroup(member);
    const media = el("span", "viewer-burst-media");
    appendPreviewImage(media, member, { maxEdge: currentThumbnailMaxEdge() });
    append(frame, media, el("span", "viewer-frame-index", String(index + 1)));
    if (groupIdentity(member) === groupIdentity(group)) {
      frame.classList.add("is-current");
    }
    append(frames, frame);
  });
  append(strip, frames);
  return strip;
}

function renderViewerStage(group: ReceivedAssetGroup, groups: ReceivedAssetGroup[]) {
  const stage = el("section", "viewer-stage");
  const previous = adjacentViewerGroup(groups, group, -1);
  const next = adjacentViewerGroup(groups, group, 1);
  const preview = el("div", "viewer-main-preview");
  if (state.viewerTransform.zoom > 1) {
    preview.classList.add("is-zoomed");
  }
  appendPreviewImage(preview, group, { maxEdge: VIEWER_PREVIEW_MAX_EDGE, original: true, eager: true });
  appendViewerCarryoverImage(preview);
  append(preview, renderPreviewStatusBadge(group, VIEWER_PREVIEW_MAX_EDGE, true), renderFaceRiskOverlay(group));
  preview.addEventListener("wheel", (event) => handleViewerWheel(event, preview), { passive: false });
  preview.addEventListener("dblclick", (event) => handleViewerDoubleClick(event, preview));
  preview.addEventListener("pointerdown", (event) => handleViewerPointerDown(event, preview));
  preview.addEventListener("pointermove", (event) => handleViewerPointerMove(event, preview));
  preview.addEventListener("pointerup", (event) => endViewerDrag(preview, event));
  preview.addEventListener("pointercancel", (event) => endViewerDrag(preview, event));
  preview.addEventListener("pointerleave", (event) => endViewerDrag(preview, event));
  window.requestAnimationFrame(() => applyViewerTransformToNode(preview));
  append(
    preview,
    append(
      el("div", "viewer-main-caption"),
      append(el("span", ""), el("strong", "", group.group_key), el("span", "", formatPairLabel(group))),
      append(el("span", ""), statusDot(sourceStatus(group)), el("span", "", compactEvaluationLabel(group))),
    ),
    previous ? commandButton("上一张", "viewer-nav previous", () => void selectGroup(previous)) : null,
    next ? commandButton("下一张", "viewer-nav next", () => void selectGroup(next)) : null,
    renderViewerActionDock(group),
    renderViewerBurstQueue(group),
  );
  append(stage, preview);
  return stage;
}

function appendViewerCarryoverImage(preview: HTMLElement) {
  const carryover = state.viewerCarryoverImage;
  if (!carryover?.url) {
    return;
  }
  const image = el("img", "viewer-carryover-image") as HTMLImageElement;
  image.src = carryover.url;
  image.alt = "";
  image.decoding = "async";
  image.draggable = false;
  append(preview, image);
}

function clearViewerCarryover(preview: HTMLElement) {
  state.viewerCarryoverImage = null;
  preview.querySelectorAll(":scope > img.viewer-carryover-image").forEach((image) => image.remove());
}

function handleViewerWheel(event: WheelEvent, preview: HTMLElement) {
  const point = viewerImagePointFromEvent(event, preview);
  if (!point) {
    return;
  }
  event.preventDefault();
  event.stopPropagation();
  const multiplier = event.deltaY < 0 ? 1.18 : 1 / 1.18;
  state.viewerTransform = zoomViewerTransformAtPoint(
    state.viewerTransform,
    point,
    state.viewerTransform.zoom * multiplier,
  );
  applyViewerTransformToNode(preview);
}

function handleViewerDoubleClick(event: MouseEvent, preview: HTMLElement) {
  const point = viewerImagePointFromEvent(event, preview);
  if (!point && state.viewerTransform.zoom <= 1) {
    return;
  }
  event.preventDefault();
  event.stopPropagation();
  state.viewerTransform = toggleViewerDoubleClickZoom(
    state.viewerTransform,
    point ?? { x: 0, y: 0 },
  );
  viewerDragState = null;
  applyViewerTransformToNode(preview);
}

function handleViewerPointerDown(event: PointerEvent, preview: HTMLElement) {
  if (event.button !== 0 || state.viewerTransform.zoom <= 1 || isViewerChromeTarget(event.target)) {
    return;
  }
  event.preventDefault();
  preview.setPointerCapture?.(event.pointerId);
  viewerDragState = { x: event.clientX, y: event.clientY };
  preview.classList.add("is-dragging");
  applyViewerTransformToNode(preview);
}

function isViewerChromeTarget(target: EventTarget | null) {
  return target instanceof HTMLElement && Boolean(target.closest("button, input, select, textarea"));
}

function handleViewerPointerMove(event: PointerEvent, preview: HTMLElement) {
  if (!viewerDragState) {
    return;
  }
  event.preventDefault();
  const delta = {
    x: event.clientX - viewerDragState.x,
    y: event.clientY - viewerDragState.y,
  };
  viewerDragState = { x: event.clientX, y: event.clientY };
  state.viewerTransform = dragViewerTransform(state.viewerTransform, delta);
  applyViewerTransformToNode(preview);
}

function endViewerDrag(preview: HTMLElement, event?: PointerEvent) {
  if (event && preview.hasPointerCapture?.(event.pointerId)) {
    preview.releasePointerCapture?.(event.pointerId);
  }
  viewerDragState = null;
  preview.classList.remove("is-dragging");
  applyViewerTransformToNode(preview);
}

function viewerImagePointFromEvent(event: MouseEvent, preview: HTMLElement) {
  const image = preview.querySelector<HTMLImageElement>(":scope > img.preview-image");
  if (!image) {
    return null;
  }
  const previewRect = preview.getBoundingClientRect();
  const naturalSize = {
    width: image.naturalWidth || previewRect.width || 1,
    height: image.naturalHeight || previewRect.height || 1,
  };
  const fit = containedImageRect(
    { left: previewRect.left, top: previewRect.top, width: previewRect.width, height: previewRect.height },
    naturalSize,
  );
  const point = normalizedContainedImagePoint(
    {
      left: previewRect.left,
      top: previewRect.top,
      width: previewRect.width,
      height: previewRect.height,
    },
    naturalSize,
    { x: event.clientX, y: event.clientY },
  );
  if (!point.inside && state.viewerTransform.zoom <= 1) {
    return null;
  }
  const x = clamp(event.clientX - fit.left, 0, fit.width);
  const y = clamp(event.clientY - fit.top, 0, fit.height);
  if (!Number.isFinite(x) || !Number.isFinite(y)) {
    return null;
  }
  return {
    x: clamp(x, 0, fit.width),
    y: clamp(y, 0, fit.height),
  };
}

function applyViewerTransformToNode(preview: HTMLElement) {
  const image = preview.querySelector<HTMLImageElement>(":scope > img.preview-image");
  if (!image) {
    return;
  }
  const previewRect = preview.getBoundingClientRect();
  const naturalSize = {
    width: image.naturalWidth || previewRect.width || 1,
    height: image.naturalHeight || previewRect.height || 1,
  };
  const fit = containedImageRect(
    { left: 0, top: 0, width: previewRect.width, height: previewRect.height },
    naturalSize,
  );
  const transform = `translate3d(${state.viewerTransform.panX}px, ${state.viewerTransform.panY}px, 0) scale(${state.viewerTransform.zoom})`;

  image.style.left = `${fit.left}px`;
  image.style.top = `${fit.top}px`;
  image.style.right = "auto";
  image.style.bottom = "auto";
  image.style.width = `${fit.width}px`;
  image.style.height = `${fit.height}px`;
  image.style.objectFit = "fill";
  image.style.transformOrigin = "0 0";
  image.style.transform = transform;
  image.style.cursor = viewerDragState ? "grabbing" : "";
  preview.style.cursor = viewerDragState ? "grabbing" : "";

  preview.querySelectorAll<HTMLImageElement>(":scope > img.viewer-carryover-image").forEach((carryoverImage) => {
    const carryoverSize = {
      width: carryoverImage.naturalWidth || naturalSize.width,
      height: carryoverImage.naturalHeight || naturalSize.height,
    };
    const carryoverFit = containedImageRect(
      { left: 0, top: 0, width: previewRect.width, height: previewRect.height },
      carryoverSize,
    );
    carryoverImage.style.left = `${carryoverFit.left}px`;
    carryoverImage.style.top = `${carryoverFit.top}px`;
    carryoverImage.style.right = "auto";
    carryoverImage.style.bottom = "auto";
    carryoverImage.style.width = `${carryoverFit.width}px`;
    carryoverImage.style.height = `${carryoverFit.height}px`;
    carryoverImage.style.objectFit = "fill";
    carryoverImage.style.transformOrigin = "0 0";
    carryoverImage.style.transform = transform;
    carryoverImage.style.cursor = viewerDragState ? "grabbing" : "";
  });

  const faceLayer = preview.querySelector<HTMLElement>(":scope > .face-risk-layer");
  if (faceLayer && !faceLayer.hidden) {
    faceLayer.style.left = `${fit.left}px`;
    faceLayer.style.top = `${fit.top}px`;
    faceLayer.style.width = `${fit.width}px`;
    faceLayer.style.height = `${fit.height}px`;
    faceLayer.style.transformOrigin = "0 0";
    faceLayer.style.transform = transform;
  }

  preview.classList.toggle("is-zoomed", state.viewerTransform.zoom > 1);
  preview.classList.toggle("is-dragging", Boolean(viewerDragState));
}

function renderViewerActionDock(group: ReceivedAssetGroup) {
  const dock = el("section", "viewer-action-dock");
  const setup = currentIntelligenceSetup();
  const keep = commandButton(group.user_marks.favorite ? "已收藏" : "收藏", "viewer-action", () => void toggleFavorite(group), Boolean(state.busy));
  const mark = commandButton(group.user_marks.marked ? "已标记" : "标记", "viewer-action", () => void toggleMarked(group), Boolean(state.busy));
  if (group.user_marks.favorite) keep.classList.add("is-active");
  if (group.user_marks.marked) mark.classList.add("is-active");
  append(
    dock,
    keep,
    mark,
    commandButton("质量检查", "viewer-action", () => void runAnalysisJobs(), Boolean(state.busy)),
    commandButton("AI 评价", "viewer-action", () => void evaluateGroupWithModel(group), Boolean(state.busy || !setup.modelReady || !group.group_id)),
    commandButton("推荐连拍", "viewer-action primary-action", () => void recommendBurst(group), Boolean(state.busy || !group.burst)),
    commandButton("移出连拍", "viewer-action", () => void removeFromBurst(group), Boolean(state.busy || !group.burst)),
    commandButton("删除", "viewer-action danger-action", () => void deleteAssetGroup(group), Boolean(state.busy || !group.group_id)),
  );
  return dock;
}

function renderViewerFilmstrip(groups: ReceivedAssetGroup[], current: ReceivedAssetGroup) {
  if (state.viewerFilmstripCollapsed) {
    const collapsed = el("section", "viewer-filmstrip-rail");
    append(
      collapsed,
      commandButton(`队列 / ${groups.length}`, "viewer-filmstrip-toggle", () => {
        state.viewerFilmstripCollapsed = false;
        render();
      }),
    );
    return collapsed;
  }
  const filmstrip = el("section", "viewer-filmstrip");
  const queue = viewerQueueWindow(groups, current, 10);
  append(
    filmstrip,
    append(
      el("div", "viewer-section-label"),
      el("strong", "", "队列"),
      append(
        el("span", "viewer-filmstrip-actions"),
        el("span", "", `${groups.length} 组`),
        commandButton("收起", "viewer-filmstrip-toggle", () => {
          state.viewerFilmstripCollapsed = true;
          render();
        }),
      ),
    ),
  );
  const frames = el("div", "viewer-filmstrip-frames");
  for (const group of queue) {
    const frame = commandButton("", "viewer-filmstrip-card", () => void selectGroup(group));
    appendPreviewImage(frame, group, { maxEdge: currentThumbnailMaxEdge() });
    append(
      frame,
      renderPreviewStatusBadge(group),
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

function handleGroupBoardScroll(board: HTMLElement) {
  state.boardScrollTop = board.scrollTop;
  state.boardWidth = board.clientWidth;
  markThumbnailScrolling();
  if (virtualBoardFrame !== null) {
    return;
  }
  virtualBoardFrame = requestAnimationFrame(() => {
    virtualBoardFrame = null;
    if (board.isConnected) {
      updateVirtualBoard(board);
    }
  });
}

function markThumbnailScrolling() {
  thumbnailScrolling = true;
  if (thumbnailScrollIdleTimer !== null) {
    window.clearTimeout(thumbnailScrollIdleTimer);
  }
  thumbnailScrollIdleTimer = window.setTimeout(() => {
    thumbnailScrolling = false;
    thumbnailScrollIdleTimer = null;
    pumpThumbnailQueue();
  }, THUMBNAIL_SCROLL_IDLE_MS);
}

function updateActiveVirtualBoard() {
  const board = document.querySelector<HTMLElement>(".group-board");
  if (board) {
    lastVirtualSignature = "";
    updateVirtualBoard(board);
  }
}

function updateVirtualBoard(board: HTMLElement) {
  const groups = filteredGroups();
  if (!groups.length) {
    board.replaceChildren(renderWorkbenchEmptyState());
    return;
  }
  if (!board.querySelector(".virtual-board-spacer")) {
    const spacer = el("div", "virtual-board-spacer");
    append(spacer, el("div", "virtual-board-window"));
    board.replaceChildren(spacer);
  }
  const metrics = virtualMetricsForBoard(board, groups.length);
  renderVirtualWindow(board, groups, metrics);
  if (shouldLoadMoreGroups(board, metrics)) {
    void loadMoreAssetGroups();
  }
}

function renderVirtualWindow(board: HTMLElement, groups: ReceivedAssetGroup[], metrics: VisibleGridWindow) {
  const spacer = board.querySelector<HTMLElement>(".virtual-board-spacer");
  const windowNode = board.querySelector<HTMLElement>(".virtual-board-window");
  if (!spacer || !windowNode) {
    return;
  }
  spacer.style.height = `${metrics.totalHeight}px`;
  windowNode.style.transform = `translate3d(0, ${metrics.offsetY}px, 0)`;
  windowNode.style.gridTemplateColumns = `repeat(${metrics.columns}, minmax(0, var(--thumb-size, 320px)))`;
  const signature = [
    metrics.startIndex,
    metrics.endIndex,
    metrics.columns,
    groups.length,
    state.thumbSize,
    state.selectedGroupId ?? "",
    state.sourceFilter,
    state.viewFilter,
  ].join(":");
  if (signature === lastVirtualSignature && windowNode.childElementCount) {
    prefetchThumbnailsAroundWindow(groups, metrics);
    return;
  }
  lastVirtualSignature = signature;
  windowNode.replaceChildren(...groups.slice(metrics.startIndex, metrics.endIndex).map(renderGroupCard));
  prefetchThumbnailsAroundWindow(groups, metrics);
}

function virtualMetricsForBoard(board: HTMLElement, totalItems: number) {
  const viewportWidth = Math.max(1, board.clientWidth || state.boardWidth || window.innerWidth - 320);
  const viewportHeight = Math.max(1, board.clientHeight || window.innerHeight - 210);
  return visibleGridWindow({
    totalItems,
    viewportWidth,
    viewportHeight,
    scrollTop: board.scrollTop || state.boardScrollTop,
    itemWidth: state.thumbSize,
    rowHeight: estimatedGroupRowHeight(),
    gap: GRID_GAP,
    overscanRows: VIRTUAL_OVERSCAN_ROWS,
  });
}

function estimatedGroupRowHeight() {
  return Math.round(state.thumbSize * 0.75) + 96;
}

function shouldLoadMoreGroups(board: HTMLElement, metrics: VisibleGridWindow) {
  const page = state.assetPage;
  if (!page?.has_more || state.assetPageLoading || state.sourceFilter !== "all" || state.viewFilter !== "light-table") {
    return false;
  }
  return board.scrollTop + board.clientHeight >= metrics.totalHeight - metrics.rowHeight * 4;
}

function prefetchThumbnailsAroundWindow(groups: ReceivedAssetGroup[], metrics: VisibleGridWindow) {
  const preloadCount = Math.max(metrics.columns, metrics.columns * THUMBNAIL_PREFETCH_ROWS);
  const endIndex = Math.min(groups.length, metrics.endIndex + preloadCount);
  warmThumbnailsForGroups(groups.slice(metrics.endIndex, endIndex));
}

function warmThumbnailsForGroups(groups: ReceivedAssetGroup[]) {
  if (!groups.length) {
    return;
  }
  window.setTimeout(() => {
    const localPaths: string[] = [];
    for (const group of groups) {
      const localPath = previewLocalPathForGroup(group);
      if (localPath) {
        localPaths.push(localPath);
      }
    }
    void warmThumbnailBatch(localPaths, currentThumbnailMaxEdge(), "fast");
  }, THUMBNAIL_WARMUP_DELAY_MS);
}

function warmOriginalsForGroups(groups: ReceivedAssetGroup[]) {
  if (!groups.length) {
    return;
  }
  window.setTimeout(() => {
    for (const group of groups) {
      const asset = previewAssetForGroup(group);
      const localPath = asset ? localPreviewablePath(asset) : null;
      if (!asset || !localPath) {
        continue;
      }
      if (supportsBrowserOriginalAsset(asset)) {
        const url = convertFileSrc(localPath);
        if (originalImageWarmCache.has(url)) {
          continue;
        }
        originalImageWarmCache.add(url);
        const image = new Image();
        image.decoding = "async";
        image.src = url;
      } else if (shouldRequestOriginalPreviewAsset(asset)) {
        void originalPreviewUrlForPath(localPath, "upgrade");
      } else if (supportsFullThumbnailAsset(asset)) {
        void thumbnailUrlForPath(localPath, VIEWER_PREVIEW_MAX_EDGE, "upgrade", "full");
      }
    }
  }, 80);
}

function renderWorkbenchEmptyState() {
  const empty = el("div", "empty-workbench");
  if (!state.selectedProjectId) {
    append(
      empty,
      el("h2", "", "创建项目并绑定文件夹"),
      el("p", "", "选择本地照片目录后会立即递归索引，按拍摄名合并 RAW/JPG。"),
      renderProjectCreate("hero"),
    );
    return empty;
  }
  if (!state.rootPath) {
    append(
      empty,
      el("h2", "", "这个项目还没有文件夹"),
      el("p", "", "为旧项目补一个本地照片目录，随后会自动开始索引。"),
      commandButton("绑定文件夹", "primary large", () => void chooseFolder(), Boolean(state.busy)),
    );
    return empty;
  }
  if (scanIsActive(state.scan?.phase)) {
    append(empty, el("h2", "", "正在扫描"), el("p", "", "索引更新后，照片组会逐步出现在这里。"));
    return empty;
  }
  if (state.scan?.phase === "failed") {
    append(
      empty,
      el("h2", "", "扫描失败"),
      el("p", "", compactError(state.scan.error ?? null) ?? "无法索引这个文件夹。"),
      commandButton("重新扫描", "primary large", () => void startScan(), !canStartScan()),
    );
    return empty;
  }
  append(
    empty,
    el("h2", "", state.viewFilter === "light-table" && state.sourceFilter === "all" ? "还没有索引照片" : "当前筛选没有结果"),
    el("p", "", state.viewFilter === "light-table" && state.sourceFilter === "all" ? "开始扫描后，这里会显示按拍摄名整理好的照片组。" : "切换筛选条件，或显示全部照片组。"),
    state.viewFilter === "light-table" && state.sourceFilter === "all"
      ? commandButton("扫描文件夹", "primary large", () => void startScan(), !canStartScan())
      : commandButton("显示全部", "secondary large", () => {
          state.viewFilter = "light-table";
          state.sourceFilter = "all";
          render();
        }),
  );
  return empty;
}

function renderGroupCard(group: ReceivedAssetGroup) {
  const card = el("article", "group-card");
  const isExpanded = group.group_id === state.selectedGroupId;
  card.tabIndex = 0;
  card.addEventListener("click", () => void selectGroup(group));
  card.addEventListener("keydown", (event) => {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      void selectGroup(group);
    }
  });
  if (isExpanded) {
    card.classList.add("is-selected");
    card.classList.add("is-expanded");
  }
  const thumb = renderAssetThumb(group, "asset-thumb", isExpanded);
  append(thumb, renderThumbMeta(group));
  const body = el("div", "group-card-body");
  append(
    body,
    append(
      el("div", "group-card-title"),
      append(
        el("div", "capture-title-row"),
        el("strong", "", group.group_key),
        group.burst ? el("span", "burst-count", `${group.burst.member_count} 张`) : null,
      ),
    ),
    renderCardStatusLine(group),
    renderMarks(group),
  );
  append(card, thumb, body);
  if (isExpanded && group.burst) {
    append(card, renderBurstStrip(group));
  }
  return card;
}

function renderAssetThumb(group: ReceivedAssetGroup, className: string, original = false) {
  const thumb = el("div", `${className} ${sourceStatus(group)}`);
  appendPreviewImage(thumb, group, { original, eager: original });
  append(thumb, renderPreviewStatusBadge(group, currentThumbnailMaxEdge(), original), renderFaceRiskOverlay(group));
  thumb.addEventListener("pointermove", (event) => updateLoupeFromPointer(event, group, currentThumbnailMaxEdge(), true));
  thumb.addEventListener("pointerleave", () => clearLoupeIfFloating());
  thumb.addEventListener("wheel", (event) => handleLoupeWheel(event, group, currentThumbnailMaxEdge(), true), { passive: false });
  return thumb;
}

function renderThumbMeta(group: ReceivedAssetGroup) {
  const meta = el("div", "thumb-meta");
  append(meta, el("span", "pair-badge", formatPairLabel(group)));
  if (group.burst) {
    append(meta, el("span", "pair-badge burst", `${group.burst.member_count} 张`));
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

function renderBurstStrip(group: ReceivedAssetGroup) {
  const strip = el("div", "burst-strip");
  const members = burstMembersOf(group);
  append(strip, el("span", "burst-label", `连拍 ${selectedBurstIndex(group, members)} / ${group.burst?.member_count ?? members.length}`));
  const frameRow = el("div", "burst-frames");
  members.slice(0, 10).forEach((member, index) => {
    const frame = commandButton("", "burst-frame", (event?: Event) => {
      event?.stopPropagation();
      void selectGroup(member);
    });
    frame.title = previewTooltipForGroup(member);
    appendPreviewImage(frame, member);
    append(frame, el("span", "viewer-frame-index", String(index + 1)));
    if (member.group_id === group.group_id) frame.classList.add("is-current");
    append(frameRow, frame);
  });
  append(strip, frameRow);
  return strip;
}

function renderInspector(group: ReceivedAssetGroup) {
  const panel = el("aside", "inspector");
  append(
    panel,
    append(
      el("div", "inspector-head"),
      append(el("div", ""), el("p", "eyebrow", "照片组"), el("h2", "", group.group_key)),
      commandButton("关闭", "ghost", () => {
        state.selectedGroupId = null;
        state.selectedGroup = null;
        state.groupDetail = [];
        state.loupe = null;
        render();
      }),
    ),
    renderInspectorActions(group),
    renderEvaluationPanel(group),
    renderFilesPanel(),
  );
  return panel;
}

function renderInspectorActions(group: ReceivedAssetGroup) {
  const actions = el("div", "inspector-actions");
  const setup = currentIntelligenceSetup();
  append(
    actions,
    append(
      el("div", "inspector-action-row"),
      commandButton(group.user_marks.favorite ? "取消收藏" : "收藏", "primary", () => void toggleFavorite(), Boolean(state.busy)),
      commandButton(group.user_marks.marked ? "取消标记" : "标记", "secondary", () => void toggleMarked(), Boolean(state.busy)),
    ),
    append(
      el("div", "inspector-action-row"),
      commandButton("AI 评价", "secondary", () => void evaluateGroupWithModel(group), Boolean(state.busy || !setup.modelReady || !group.group_id)),
      commandButton("推荐连拍", "secondary", () => void recommendBurst(), Boolean(state.busy || !group.burst)),
    ),
    append(
      el("div", "inspector-danger-row"),
      commandButton("删除原图", "secondary danger-text", () => void deleteAssetGroup(group), Boolean(state.busy || !group.group_id)),
    ),
  );
  return actions;
}

function renderEvaluationPanel(group: ReceivedAssetGroup) {
  const panel = el("section", "detail-panel");
  append(
    panel,
    el("h3", "", "检查结果"),
    checkResultList([
      ["文件", readable(sourceStatus(group)), sourceStatus(group)],
      ["质量", readable(group.technical_gate_status ?? group.technical_status ?? "pending"), group.technical_gate_status ?? group.technical_status ?? "pending"],
      ["AI", modelLabel(group), group.model_status ?? "pending"],
      ["等级", readable(group.model_tier ?? "none"), group.model_tier ?? "none"],
      ["连拍", group.burst ? `${group.burst.member_count} 张` : "无", group.burst ? "available" : "none"],
      ["推荐", readable(group.burst?.recommendation_status ?? "none"), group.burst?.recommendation_status ?? "none"],
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

function renderFaceRiskOverlay(group: ReceivedAssetGroup) {
  const assessment = latestFaceAssessment(group);
  const regions = assessment ? subjectRegions(assessment) : [];
  const signals = assessment ? subjectSignals(assessment) : {};
  const imageWidth = signals.image_width ?? 0;
  const imageHeight = signals.image_height ?? 0;
  const layer = el("div", "face-risk-layer");
  if (!assessment || !regions.length || imageWidth <= 0 || imageHeight <= 0) {
    layer.hidden = true;
    return layer;
  }
  layer.dataset.imageWidth = String(imageWidth);
  layer.dataset.imageHeight = String(imageHeight);
  layer.title = assessment.summary;
  for (const region of regions) {
    const x = finiteNumber(region.x);
    const y = finiteNumber(region.y);
    const width = finiteNumber(region.width ?? region.w);
    const height = finiteNumber(region.height ?? region.h);
    if (x === null || y === null || width === null || height === null || width <= 0 || height <= 0) {
      continue;
    }
    const box = el("span", "face-risk-box");
    box.style.left = `${(x / imageWidth) * 100}%`;
    box.style.top = `${(y / imageHeight) * 100}%`;
    box.style.width = `${(width / imageWidth) * 100}%`;
    box.style.height = `${(height / imageHeight) * 100}%`;
    const label = faceRiskLabel(assessment, signals);
    if (label) append(box, el("span", "face-risk-label", label));
    append(layer, box);
  }
  if (!layer.childElementCount) {
    layer.hidden = true;
  }
  requestAnimationFrame(() => {
    const parent = layer.parentElement;
    if (parent) syncFaceRiskLayer(parent);
  });
  return layer;
}

function latestFaceAssessment(group: ReceivedAssetGroup) {
  if (!group.group_id) return null;
  return (
    state.subjectAssessments[group.group_id]
      ?.filter((assessment) => assessment.subject_type === "face")
      .find((assessment) => subjectRegions(assessment).length > 0) ?? null
  );
}

function subjectRegions(assessment: SubjectAssessment): SubjectRegion[] {
  const parsed = parseJson<unknown>(assessment.regions_json, []);
  return Array.isArray(parsed) ? (parsed as SubjectRegion[]) : [];
}

function subjectSignals(assessment: SubjectAssessment): SubjectSignals {
  const parsed = parseJson<unknown>(assessment.signals_json, {});
  return parsed && typeof parsed === "object" && !Array.isArray(parsed) ? (parsed as SubjectSignals) : {};
}

function parseJson<T>(source: string, fallback: T): T {
  try {
    return JSON.parse(source) as T;
  } catch {
    return fallback;
  }
}

function finiteNumber(value: unknown): number | null {
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

function faceRiskLabel(assessment: SubjectAssessment, signals: SubjectSignals) {
  if (cssToken(assessment.gate_status) === "pass") return "";
  if (signals.closed_eyes) return "闭眼";
  if (signals.face_exposure_risk) return "面部曝光";
  if (signals.face_color_cast_risk) return "偏色";
  return cssToken(assessment.gate_status) === "warn" ? "风险" : "";
}

function syncAllFaceRiskLayers() {
  document.querySelectorAll<HTMLElement>(".face-risk-layer").forEach((layer) => {
    const parent = layer.parentElement;
    if (parent) syncFaceRiskLayer(parent);
  });
}

function syncFaceRiskLayer(container: HTMLElement) {
  const layer = container.querySelector<HTMLElement>(":scope > .face-risk-layer");
  const image = container.querySelector<HTMLImageElement>(":scope > img.preview-image");
  if (!layer || !image || layer.hidden || !image.naturalWidth || !image.naturalHeight) {
    return;
  }
  const containerWidth = container.clientWidth;
  const containerHeight = container.clientHeight;
  if (containerWidth <= 0 || containerHeight <= 0) {
    return;
  }
  if (container.classList.contains("viewer-main-preview")) {
    applyViewerTransformToNode(container);
    return;
  }
  const imageRatio = image.naturalWidth / image.naturalHeight;
  const containerRatio = containerWidth / containerHeight;
  const objectFit = getComputedStyle(image).objectFit;
  const contain = objectFit === "contain";
  const scale = contain
    ? containerRatio > imageRatio
      ? containerHeight / image.naturalHeight
      : containerWidth / image.naturalWidth
    : containerRatio > imageRatio
      ? containerWidth / image.naturalWidth
      : containerHeight / image.naturalHeight;
  const renderedWidth = image.naturalWidth * scale;
  const renderedHeight = image.naturalHeight * scale;
  layer.style.left = `${(containerWidth - renderedWidth) / 2}px`;
  layer.style.top = `${(containerHeight - renderedHeight) / 2}px`;
  layer.style.width = `${renderedWidth}px`;
  layer.style.height = `${renderedHeight}px`;
}

function renderFilesPanel() {
  const panel = el("section", "detail-panel");
  append(panel, el("h3", "", "文件"));
  const list = el("div", "file-list");
  if (!state.groupDetail.length) {
    append(list, el("div", "empty-note", "选择一个照片组后查看文件明细。"));
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

function renderLoupeOverlay() {
  const loupe = state.loupe;
  const group = loupe ? groupByIdentity(loupe.groupId) : null;
  const overlay = el("div", "loupe-overlay");
  if (!loupe || !group) {
    return overlay;
  }
  positionLoupeOverlay(overlay, loupe);
  const crop = el("div", "loupe-crop");
  crop.dataset.loupeGroup = loupe.groupId;
  setPreviewBackground(crop, group, loupe.maxEdge, loupe.original);
  crop.style.backgroundPosition = `${loupe.x * 100}% ${loupe.y * 100}%`;
  crop.style.backgroundSize = `${loupe.zoom * 100}% auto`;
  append(
    overlay,
    crop,
    append(el("div", "loupe-caption"), el("span", "", group.group_key), el("strong", "", `${loupe.zoom.toFixed(1)}x`)),
  );
  return overlay;
}

function selectedProject() {
  return state.projects.find((project) => project.project_id === state.selectedProjectId) ?? null;
}

function folderBasename(path: string) {
  return path.split(/[\\/]/).filter(Boolean).at(-1) ?? path;
}

function allGroups() {
  return state.assetPage?.groups ?? [];
}

function filteredGroups() {
  let groups = allGroups();
  if (state.viewFilter === "needs-work") {
    groups = groups.filter(needsWork);
  } else if (state.viewFilter === "missing") {
    groups = groups.filter((group) => sourceStatus(group) === "missing");
  }
  if (state.sourceFilter !== "all") {
    groups = groups.filter((group) => sourceStatus(group) === state.sourceFilter);
  }
  return displayGroupsFor(groups);
}

function displayGroupsFor(groups: ReceivedAssetGroup[]) {
  return collapseBurstGroups(groups, state.selectedGroupId);
}

function needsWork(group: ReceivedAssetGroup) {
  const technical = cssToken(group.technical_gate_status ?? group.technical_status ?? "pending");
  const model = cssToken(group.model_status ?? "pending");
  return (
    sourceStatus(group) !== "available" ||
    Boolean(group.technical_defects.length) ||
    ["pending", "technical-pending", "failed"].includes(technical) ||
    ["pending", "failed"].includes(model)
  );
}

function burstMembersOf(group: ReceivedAssetGroup) {
  const burstId = group.burst?.burst_group_id;
  if (!burstId) return [group];
  const members = allGroups().filter((candidate) => candidate.burst?.burst_group_id === burstId);
  return members.length ? members : [group];
}

function selectedBurstIndex(group: ReceivedAssetGroup, members: ReceivedAssetGroup[]) {
  const index = members.findIndex((member) => member.group_id === group.group_id);
  return index >= 0 ? index + 1 : 1;
}

function selectAdjacentBurst(group: ReceivedAssetGroup, direction: number) {
  const members = burstMembersOf(group);
  const currentIndex = Math.max(0, members.findIndex((member) => member.group_id === group.group_id));
  const next = members[(currentIndex + direction + members.length) % members.length];
  void selectGroup(next, { preserveViewerTransform: true });
}

function groupIdentity(group: ReceivedAssetGroup) {
  return group.group_id ?? group.group_key;
}

function uniqueGroupsByIdentity(groups: ReceivedAssetGroup[]) {
  const seen = new Set<string>();
  return groups.filter((group) => {
    const identity = groupIdentity(group);
    if (seen.has(identity)) {
      return false;
    }
    seen.add(identity);
    return true;
  });
}

function groupByIdentity(groupId: string) {
  return allGroups().find((group) => groupIdentity(group) === groupId) ?? null;
}

function previewUrlForGroup(group: ReceivedAssetGroup, maxEdge = currentThumbnailMaxEdge(), original = false) {
  const asset = previewAssetForGroup(group);
  const localPath = asset ? localPreviewablePath(asset) : null;
  if (!asset || !localPath) return "";
  if (original && supportsBrowserOriginalAsset(asset)) {
    return convertFileSrc(localPath);
  }
  if (original && shouldRequestOriginalPreviewAsset(asset)) {
    return originalPreviewUrlCache.get(originalPreviewCacheKey(localPath)) ?? "";
  }
  return (
    thumbnailUrlCache.get(thumbnailCacheKey(localPath, maxEdge, "full")) ??
    thumbnailUrlCache.get(thumbnailCacheKey(localPath, maxEdge, "fast")) ??
    (supportsBrowserOriginalAsset(asset) ? convertFileSrc(localPath) : "")
  );
}

function previewLocalPathForGroup(group: ReceivedAssetGroup) {
  const asset = previewAssetForGroup(group);
  return asset ? localPreviewablePath(asset) : null;
}

function originalPreviewUrlForGroup(group: ReceivedAssetGroup) {
  const asset = previewAssetForGroup(group);
  const localPath = asset ? localPreviewablePath(asset) : null;
  if (!asset || !localPath) {
    return null;
  }
  if (supportsBrowserOriginalAsset(asset)) {
    return convertFileSrc(localPath);
  }
  if (shouldRequestOriginalPreviewAsset(asset)) {
    return originalPreviewUrlCache.get(originalPreviewCacheKey(localPath)) ?? null;
  }
  return null;
}

function setPreviewBackground(node: HTMLElement, group: ReceivedAssetGroup, maxEdge = currentThumbnailMaxEdge(), original = false) {
  const url = previewUrlForGroup(group, maxEdge, original);
  if (!url) {
    node.classList.add("no-preview");
    node.style.backgroundImage = "";
    return;
  }
  node.classList.remove("no-preview");
  node.style.backgroundImage = `url("${url}")`;
}

function appendPreviewImage(node: HTMLElement, group: ReceivedAssetGroup, options: PreviewImageOptions = {}) {
  const previewAsset = previewAssetForGroup(group);
  const localPath = previewAsset ? localPreviewablePath(previewAsset) : null;
  if (!previewAsset || !localPath) {
    node.classList.add("no-preview");
    syncPreviewStatusBadge(node, "idle");
    return;
  }
  const shouldUpgradeFull = shouldRequestFullPreviewAsset(previewAsset, Boolean(options.original));
  const maxEdge = options.maxEdge ?? currentThumbnailMaxEdge();
  if (options.original && supportsBrowserOriginalAsset(previewAsset)) {
    node.dataset.previewPath = localPath;
    node.dataset.previewMaxEdge = "original";
    node.classList.remove("no-preview", "is-loading");
    insertPreviewImage(node, convertFileSrc(localPath), "original", options.eager);
    syncPreviewStatusBadge(node, "original");
    refreshPreviewProgressDom();
    return;
  }
  if (options.original && shouldRequestOriginalPreviewAsset(previewAsset)) {
    appendOriginalPreviewImage(node, group, localPath, maxEdge, options.eager);
    return;
  }
  const fullKey = thumbnailCacheKey(localPath, maxEdge, "full");
  const fastKey = thumbnailCacheKey(localPath, maxEdge, "fast");
  node.dataset.previewPath = localPath;
  node.dataset.previewMaxEdge = String(maxEdge);
  node.classList.remove("no-preview");
  const cachedFullUrl = thumbnailUrlCache.get(fullKey);
  if (cachedFullUrl) {
    insertPreviewImage(node, cachedFullUrl, "full", options.eager);
    syncPreviewStatusBadge(node, "full");
    return;
  }
  const cachedFastUrl = thumbnailUrlCache.get(fastKey);
  if (cachedFastUrl) {
    insertPreviewImage(node, cachedFastUrl, "fast", options.eager);
    syncPreviewStatusBadge(node, "fast");
    if (shouldUpgradeFull) scheduleFullQualityUpgrade(node, localPath, maxEdge);
    return;
  }
  node.classList.add("is-loading");
  const request = thumbnailUrlForPath(localPath, maxEdge, "visible", "fast");
  syncPreviewStatusBadge(node, previewStageForGroup(group, maxEdge, Boolean(options.original)));
  void request.then((url) => {
    if (!node.isConnected || node.dataset.previewPath !== localPath || node.dataset.previewMaxEdge !== String(maxEdge)) {
      return;
    }
    node.classList.remove("is-loading");
    if (!url) {
      node.classList.add("no-preview");
      syncPreviewStatusBadge(node, "failed");
      return;
    }
    node.classList.remove("no-preview");
    insertPreviewImage(node, url, "fast", options.eager);
    syncPreviewStatusBadge(node, "fast");
    refreshPreviewProgressDom();
    if (shouldUpgradeFull) scheduleFullQualityUpgrade(node, localPath, maxEdge);
  });
}

function appendOriginalPreviewImage(
  node: HTMLElement,
  group: ReceivedAssetGroup,
  localPath: string,
  fallbackMaxEdge: number,
  eager = false,
) {
  const key = originalPreviewCacheKey(localPath);
  node.dataset.previewPath = localPath;
  node.dataset.previewMaxEdge = "original";
  node.classList.remove("no-preview");
  const cachedOriginal = originalPreviewUrlCache.get(key);
  if (cachedOriginal) {
    node.classList.remove("is-loading");
    insertPreviewImage(node, cachedOriginal, "original", eager);
    syncPreviewStatusBadge(node, "original");
    refreshPreviewProgressDom();
    return;
  }

  const fallback =
    thumbnailUrlCache.get(thumbnailCacheKey(localPath, fallbackMaxEdge, "full")) ??
    thumbnailUrlCache.get(thumbnailCacheKey(localPath, fallbackMaxEdge, "fast"));
  if (fallback) {
    insertPreviewImage(node, fallback, "full", eager);
  } else {
    node.classList.add("is-loading");
    void thumbnailUrlForPath(localPath, fallbackMaxEdge, "visible", "fast").then((url) => {
      if (!node.isConnected || node.dataset.previewPath !== localPath || node.dataset.previewMaxEdge !== "original") {
        return;
      }
      node.classList.remove("is-loading");
      if (url && !originalPreviewUrlCache.has(key)) {
        insertPreviewImage(node, url, "fast", eager);
      }
    });
  }

  syncPreviewStatusBadge(node, previewStageForGroup(group, fallbackMaxEdge, true));
  void originalPreviewUrlForPath(localPath, "visible").then((url) => {
    if (!node.isConnected || node.dataset.previewPath !== localPath || node.dataset.previewMaxEdge !== "original") {
      return;
    }
    node.classList.remove("is-loading");
    if (!url) {
      node.classList.add("no-preview");
      syncPreviewStatusBadge(node, "failed");
      return;
    }
    node.classList.remove("no-preview");
    insertPreviewImage(node, url, "original", eager);
    syncPreviewStatusBadge(node, "original");
    refreshPreviewProgressDom();
  });
}

function scheduleFullQualityUpgrade(node: HTMLElement, localPath: string, maxEdge: number) {
  const key = thumbnailCacheKey(localPath, maxEdge, "full");
  const cachedUrl = thumbnailUrlCache.get(key);
  if (cachedUrl) {
    insertPreviewImage(node, cachedUrl, "full");
    syncPreviewStatusBadge(node, "full");
    return;
  }
  if (node.dataset.previewFullPending === key) {
    return;
  }
  node.dataset.previewFullPending = key;
  void thumbnailUrlForPath(localPath, maxEdge, "upgrade", "full").then((url) => {
    if (node.dataset.previewFullPending === key) {
      delete node.dataset.previewFullPending;
    }
    if (
      !node.isConnected ||
      !url ||
      node.dataset.previewPath !== localPath ||
      node.dataset.previewMaxEdge !== String(maxEdge)
    ) {
      return;
    }
    insertPreviewImage(node, url, "full");
    syncPreviewStatusBadge(node, "full");
    refreshPreviewProgressDom();
  });
}

function insertPreviewImage(node: HTMLElement, url: string, quality: PreviewImageQuality, eager = false) {
  const current = node.querySelector<HTMLImageElement>(":scope > img.preview-image");
  if (current?.src === url && current.dataset.quality === quality) {
    return;
  }
  const image = el("img", "preview-image") as HTMLImageElement;
  image.src = url;
  image.alt = "";
  image.loading = eager ? "eager" : "lazy";
  image.decoding = "async";
  image.draggable = false;
  image.dataset.quality = quality;
  image.setAttribute("fetchpriority", eager ? "high" : "low");
  const settle = () => {
    node.querySelectorAll<HTMLImageElement>(":scope > img.preview-image").forEach((candidate) => {
      if (candidate !== image) {
        candidate.remove();
      }
    });
    syncFaceRiskLayer(node);
    if (node.classList.contains("viewer-main-preview")) {
      clearViewerCarryover(node);
      applyViewerTransformToNode(node);
    }
  };
  image.addEventListener("load", settle, { once: true });
  image.addEventListener("error", () => {
    image.remove();
    if (node.classList.contains("viewer-main-preview")) {
      clearViewerCarryover(node);
    }
  }, { once: true });
  node.prepend(image);
  if (image.complete) {
    settle();
  }
}

function thumbnailCacheKey(localPath: string, maxEdge = THUMBNAIL_MAX_EDGE, quality: ThumbnailQuality = "fast") {
  return `${quality}:${maxEdge}:${localPath}`;
}

function originalPreviewCacheKey(localPath: string) {
  return `original:${localPath}`;
}

function currentThumbnailMaxEdge() {
  const pixelRatio = Math.min(Math.max(window.devicePixelRatio || 1, 1), 2.25);
  const edge = Math.ceil(state.thumbSize * pixelRatio);
  return Math.min(THUMBNAIL_MAX_EDGE, Math.max(THUMBNAIL_MIN_EDGE, edge));
}

function previewStageForGroup(group: ReceivedAssetGroup, maxEdge = currentThumbnailMaxEdge(), original = false): PreviewStage {
  const asset = previewAssetForGroup(group);
  const localPath = asset ? localPreviewablePath(asset) : null;
  if (!asset || !localPath) {
    return "idle";
  }
  if (original && supportsBrowserOriginalAsset(asset)) {
    return "original";
  }
  if (original && shouldRequestOriginalPreviewAsset(asset)) {
    return originalPreviewStageForLocalPath(localPath);
  }
  return previewStageForLocalPath(localPath, maxEdge);
}

function originalPreviewStageForLocalPath(localPath: string): PreviewStage {
  const key = originalPreviewCacheKey(localPath);
  if (originalPreviewUrlCache.has(key)) {
    return "original";
  }
  if (originalPreviewActiveKeys.has(key)) {
    return "loading";
  }
  if (originalPreviewPending.has(key) || originalPreviewQueued.has(key)) {
    return "queued";
  }
  if (originalPreviewFailedKeys.has(key)) {
    return "failed";
  }
  return "idle";
}

function previewStageForLocalPath(localPath: string, maxEdge = currentThumbnailMaxEdge()): PreviewStage {
  const fullKey = thumbnailCacheKey(localPath, maxEdge, "full");
  const fastKey = thumbnailCacheKey(localPath, maxEdge, "fast");
  if (thumbnailUrlCache.has(fullKey)) {
    return "full";
  }
  if (thumbnailUrlCache.has(fastKey)) {
    return "fast";
  }
  if (thumbnailActiveKeys.has(fullKey) || thumbnailActiveKeys.has(fastKey)) {
    return "loading";
  }
  if (
    thumbnailPending.has(fullKey) ||
    thumbnailPending.has(fastKey) ||
    thumbnailQueued.has(fullKey) ||
    thumbnailQueued.has(fastKey) ||
    thumbnailBatchPending.has(fullKey) ||
    thumbnailBatchPending.has(fastKey)
  ) {
    return "queued";
  }
  if (thumbnailFailedKeys.has(fullKey) || thumbnailFailedKeys.has(fastKey)) {
    return "failed";
  }
  return "idle";
}

function renderPreviewStatusBadge(
  group: ReceivedAssetGroup,
  maxEdge = currentThumbnailMaxEdge(),
  original = false,
) {
  const badge = el("span", "preview-status-badge");
  badge.dataset.previewStatusBadge = "true";
  applyPreviewStatusBadge(badge, previewStageForGroup(group, maxEdge, original));
  return badge;
}

function previewTooltipForGroup(
  group: ReceivedAssetGroup,
  maxEdge = currentThumbnailMaxEdge(),
  original = false,
) {
  const display = previewBadge(previewStageForGroup(group, maxEdge, original));
  return `${group.group_key} · ${display.label}`;
}

function applyPreviewStatusBadge(badge: HTMLElement, stage: PreviewStage) {
  const display = previewBadge(stage);
  badge.className = `preview-status-badge ${display.tone}`;
  badge.textContent = display.label;
  badge.title = display.title;
  badge.dataset.previewStage = stage;
}

function syncPreviewStatusBadge(node: HTMLElement, stage: PreviewStage) {
  const badge = node.querySelector<HTMLElement>(":scope > .preview-status-badge");
  if (badge) {
    applyPreviewStatusBadge(badge, stage);
  }
}

function refreshPreviewProgressDom() {
  if (previewProgressFrame !== null) {
    return;
  }
  previewProgressFrame = window.requestAnimationFrame(() => {
    previewProgressFrame = null;
    const progress = currentPreviewProgress();
    document.querySelectorAll<HTMLElement>("[data-preview-progress='true']").forEach((node) => {
      node.textContent = progress.label;
    });
  });
}

function syncPreviewNodesForThumbnailItem(item: ThumbnailQueueItem) {
  document.querySelectorAll<HTMLElement>("[data-preview-path]").forEach((node) => {
    if (node.dataset.previewPath !== item.localPath || node.dataset.previewMaxEdge !== String(item.maxEdge)) {
      return;
    }
    const url = thumbnailUrlCache.get(item.key) ?? null;
    applyCachedPreviewToNode(node, {
      localPath: item.localPath,
      maxEdge: String(item.maxEdge),
      quality: item.quality,
      url,
    });
    syncPreviewStatusBadge(node, previewStageForLocalPath(item.localPath, item.maxEdge));
  });
}

async function thumbnailUrlForPath(
  localPath: string,
  maxEdge = THUMBNAIL_MAX_EDGE,
  priority: ThumbnailPriority = "visible",
  quality: ThumbnailQuality = "fast",
) {
  const key = thumbnailCacheKey(localPath, maxEdge, quality);
  const cached = thumbnailUrlCache.get(key);
  if (cached) return cached;
  const pending = thumbnailPending.get(key);
  if (pending) {
    if (priority === "visible") {
      promoteQueuedThumbnail(key);
    }
    return pending;
  }
  const request = enqueueThumbnailRequest(key, localPath, maxEdge, priority, quality).finally(() => {
    thumbnailPending.delete(key);
  });
  thumbnailPending.set(key, request);
  return request;
}

async function originalPreviewUrlForPath(localPath: string, priority: ThumbnailPriority = "visible") {
  const key = originalPreviewCacheKey(localPath);
  const cached = originalPreviewUrlCache.get(key);
  if (cached) return cached;
  const pending = originalPreviewPending.get(key);
  if (pending) {
    if (priority === "visible") {
      promoteQueuedOriginalPreview(key);
    }
    return pending;
  }
  const request = enqueueOriginalPreviewRequest(key, localPath, priority).finally(() => {
    originalPreviewPending.delete(key);
  });
  originalPreviewPending.set(key, request);
  return request;
}

async function warmThumbnailBatch(localPaths: string[], maxEdge: number, quality: ThumbnailQuality) {
  const batch: Array<{ key: string; localPath: string }> = [];
  const seen = new Set<string>();
  for (const localPath of localPaths) {
    const key = thumbnailCacheKey(localPath, maxEdge, quality);
    if (
      seen.has(key) ||
      thumbnailUrlCache.has(key) ||
      thumbnailPending.has(key) ||
      thumbnailQueued.has(key) ||
      thumbnailBatchPending.has(key)
    ) {
      continue;
    }
    seen.add(key);
    batch.push({ key, localPath });
  }

  for (let index = 0; index < batch.length; index += THUMBNAIL_BATCH_SIZE) {
    const chunk = batch.slice(index, index + THUMBNAIL_BATCH_SIZE);
    for (const item of chunk) {
      thumbnailBatchPending.add(item.key);
    }
    refreshPreviewProgressDom();
    try {
      const response = await api.getAssetThumbnails(
        chunk.map((item) => item.localPath),
        maxEdge,
        quality,
      );
      for (const item of response.thumbnails) {
        if (!item.path) {
          thumbnailFailedKeys.add(thumbnailCacheKey(item.source_path, maxEdge, quality));
          continue;
        }
        const key = thumbnailCacheKey(item.source_path, maxEdge, quality);
        thumbnailFailedKeys.delete(key);
        const url = convertFileSrc(item.path);
        thumbnailUrlCache.set(key, url);
        syncPreviewNodesForCachedThumbnail(item.source_path, maxEdge, quality, url);
      }
    } catch {
      // Visible thumbnails still use the priority queue; background warmup can fail quietly.
    } finally {
      for (const item of chunk) {
        thumbnailBatchPending.delete(item.key);
      }
      refreshPreviewProgressDom();
    }
  }
}

function enqueueOriginalPreviewRequest(key: string, localPath: string, priority: ThumbnailPriority) {
  const request = new Promise<string | null>((resolve) => {
    const item: OriginalPreviewQueueItem = { key, localPath, priority, resolve };
    originalPreviewFailedKeys.delete(key);
    originalPreviewQueued.set(key, item);
    if (priority === "visible") {
      originalPreviewQueue.unshift(item);
    } else if (priority === "upgrade") {
      const firstPrefetchIndex = originalPreviewQueue.findIndex((candidate) => candidate.priority === "prefetch");
      if (firstPrefetchIndex >= 0) {
        originalPreviewQueue.splice(firstPrefetchIndex, 0, item);
      } else {
        originalPreviewQueue.push(item);
      }
    } else {
      originalPreviewQueue.push(item);
    }
    pumpOriginalPreviewQueue();
    refreshPreviewProgressDom();
  });
  return request;
}

function promoteQueuedOriginalPreview(key: string) {
  const item = originalPreviewQueued.get(key);
  if (!item) {
    return;
  }
  const index = originalPreviewQueue.indexOf(item);
  if (index <= 0) {
    return;
  }
  originalPreviewQueue.splice(index, 1);
  originalPreviewQueue.unshift(item);
}

function pumpOriginalPreviewQueue() {
  while (originalPreviewQueue.length && originalPreviewActiveCount < ORIGINAL_PREVIEW_CONCURRENCY) {
    const [item] = originalPreviewQueue.splice(0, 1);
    originalPreviewQueued.delete(item.key);
    if (originalPreviewUrlCache.has(item.key)) {
      item.resolve(originalPreviewUrlCache.get(item.key) ?? null);
      syncPreviewNodesForOriginalItem(item);
      refreshPreviewProgressDom();
      continue;
    }
    startOriginalPreviewItem(item);
    void api
      .getAssetOriginalPreview(item.localPath)
      .then((response) => {
        const url = convertFileSrc(response.path);
        originalPreviewFailedKeys.delete(item.key);
        originalPreviewUrlCache.set(item.key, url);
        item.resolve(url);
      })
      .catch(() => {
        originalPreviewFailedKeys.add(item.key);
        item.resolve(null);
      })
      .finally(() => {
        finishOriginalPreviewItem(item);
        pumpOriginalPreviewQueue();
      });
  }
}

function startOriginalPreviewItem(item: OriginalPreviewQueueItem) {
  originalPreviewActiveKeys.add(item.key);
  originalPreviewActiveCount += 1;
  syncPreviewNodesForOriginalItem(item);
  refreshPreviewProgressDom();
}

function finishOriginalPreviewItem(item: OriginalPreviewQueueItem) {
  originalPreviewActiveKeys.delete(item.key);
  originalPreviewActiveCount = Math.max(0, originalPreviewActiveCount - 1);
  syncPreviewNodesForOriginalItem(item);
  refreshPreviewProgressDom();
}

function enqueueThumbnailRequest(
  key: string,
  localPath: string,
  maxEdge: number,
  priority: ThumbnailPriority,
  quality: ThumbnailQuality,
) {
  const request = new Promise<string | null>((resolve) => {
    const item: ThumbnailQueueItem = { key, localPath, maxEdge, quality, priority, resolve };
    thumbnailFailedKeys.delete(key);
    thumbnailQueued.set(key, item);
    if (priority === "visible") {
      thumbnailQueue.unshift(item);
    } else if (priority === "upgrade") {
      const firstPrefetchIndex = thumbnailQueue.findIndex((candidate) => candidate.priority === "prefetch");
      if (firstPrefetchIndex >= 0) {
        thumbnailQueue.splice(firstPrefetchIndex, 0, item);
      } else {
        thumbnailQueue.push(item);
      }
    } else {
      thumbnailQueue.push(item);
    }
    pumpThumbnailQueue();
    refreshPreviewProgressDom();
  });
  return request;
}

function syncPreviewNodesForOriginalItem(item: OriginalPreviewQueueItem) {
  document.querySelectorAll<HTMLElement>("[data-preview-path]").forEach((node) => {
    if (node.dataset.previewPath !== item.localPath || node.dataset.previewMaxEdge !== "original") {
      return;
    }
    const url = originalPreviewUrlCache.get(item.key) ?? null;
    applyCachedPreviewToNode(node, {
      localPath: item.localPath,
      maxEdge: "original",
      quality: "original",
      url,
    });
    syncPreviewStatusBadge(node, originalPreviewStageForLocalPath(item.localPath));
  });
}

function syncPreviewNodesForCachedThumbnail(
  localPath: string,
  maxEdge: number,
  quality: ThumbnailQuality,
  url: string | null,
) {
  document.querySelectorAll<HTMLElement>("[data-preview-path]").forEach((node) => {
    if (node.dataset.previewPath !== localPath || node.dataset.previewMaxEdge !== String(maxEdge)) {
      return;
    }
    applyCachedPreviewToNode(node, {
      localPath,
      maxEdge: String(maxEdge),
      quality,
      url,
    });
    syncPreviewStatusBadge(node, previewStageForLocalPath(localPath, maxEdge));
  });
}

function applyCachedPreviewToNode(
  node: HTMLElement,
  item: { localPath: string; maxEdge: string; quality: PreviewSyncQuality; url: string | null },
) {
  const current = node.querySelector<HTMLImageElement>(":scope > img.preview-image");
  if (
    !shouldApplyPreviewSync(
      {
        previewPath: node.dataset.previewPath,
        previewMaxEdge: node.dataset.previewMaxEdge,
        previewFullPending: node.dataset.previewFullPending,
        currentQuality: previewQualityFromImage(current),
      },
      item,
    )
  ) {
    return;
  }
  node.classList.remove("no-preview", "is-loading");
  insertPreviewImage(node, item.url as string, item.quality);
}

function previewQualityFromImage(image: HTMLImageElement | null): PreviewSyncQuality | undefined {
  const quality = image?.dataset.quality;
  if (quality === "fast" || quality === "full" || quality === "original") {
    return quality;
  }
  return undefined;
}

function promoteQueuedThumbnail(key: string) {
  const item = thumbnailQueued.get(key);
  if (!item) {
    return;
  }
  const index = thumbnailQueue.indexOf(item);
  if (index <= 0) {
    return;
  }
  thumbnailQueue.splice(index, 1);
  thumbnailQueue.unshift(item);
}

function pumpThumbnailQueue() {
  while (thumbnailQueue.length) {
    const index = thumbnailQueue.findIndex(canStartThumbnailItem);
    if (index < 0) {
      return;
    }
    const [item] = thumbnailQueue.splice(index, 1);
    thumbnailQueued.delete(item.key);
    if (thumbnailUrlCache.has(item.key)) {
      item.resolve(thumbnailUrlCache.get(item.key) ?? null);
      syncPreviewNodesForThumbnailItem(item);
      refreshPreviewProgressDom();
      continue;
    }
    startThumbnailItem(item);
    void api
      .getAssetThumbnail(item.localPath, item.maxEdge, item.quality)
      .then((response) => {
        const url = convertFileSrc(response.path);
        thumbnailFailedKeys.delete(item.key);
        thumbnailUrlCache.set(item.key, url);
        item.resolve(url);
      })
      .catch(() => {
        thumbnailFailedKeys.add(item.key);
        item.resolve(null);
      })
      .finally(() => {
        finishThumbnailItem(item);
        pumpThumbnailQueue();
      });
  }
}

function canStartThumbnailItem(item: ThumbnailQueueItem) {
  if (item.quality === "full") {
    return thumbnailFullActiveCount < fullThumbnailConcurrency(thumbnailScrolling);
  }
  return thumbnailActiveCount < THUMBNAIL_CONCURRENCY;
}

function startThumbnailItem(item: ThumbnailQueueItem) {
  thumbnailActiveKeys.add(item.key);
  if (item.quality === "full") {
    thumbnailFullActiveCount += 1;
  } else {
    thumbnailActiveCount += 1;
  }
  syncPreviewNodesForThumbnailItem(item);
  refreshPreviewProgressDom();
}

function finishThumbnailItem(item: ThumbnailQueueItem) {
  thumbnailActiveKeys.delete(item.key);
  if (item.quality === "full") {
    thumbnailFullActiveCount = Math.max(0, thumbnailFullActiveCount - 1);
  } else {
    thumbnailActiveCount = Math.max(0, thumbnailActiveCount - 1);
  }
  syncPreviewNodesForThumbnailItem(item);
  refreshPreviewProgressDom();
}

function previewAssetForGroup(group: ReceivedAssetGroup) {
  const candidates = [group.jpeg, group.primary, group.video, group.raw].filter(Boolean) as ReceivedAsset[];
  return candidates.find(isPreviewableAsset) ?? null;
}

function localPreviewablePath(asset: ReceivedAsset) {
  if (!isPreviewableAsset(asset)) return null;
  return localPathFromLocation(asset.storage_location) ?? absolutePathOrNull(asset.original_path);
}

function localPathFromLocation(location: StoredObjectLocation | null | undefined): string | null {
  if (!location) return null;
  if (typeof location === "string") return absolutePathOrNull(location);
  if (typeof location !== "object") return null;
  const record = location as Record<string, unknown>;
  const direct = record.path ?? record.local_path ?? record.localPath;
  if (typeof direct === "string") return absolutePathOrNull(direct);
  for (const value of Object.values(record)) {
    if (typeof value === "string") {
      const path = absolutePathOrNull(value);
      if (path) return path;
    }
  }
  return null;
}

function isPreviewableAsset(asset: ReceivedAsset) {
  const format = cssToken(asset.format || extensionOf(asset.filename));
  return isPreviewableFormat(format);
}

function supportsFullThumbnailAsset(asset: ReceivedAsset) {
  const format = cssToken(asset.format || extensionOf(asset.filename));
  return supportsFullThumbnailFormat(format);
}

function supportsBrowserOriginalAsset(asset: ReceivedAsset) {
  const format = cssToken(asset.format || extensionOf(asset.filename));
  return isBrowserPreviewFormat(format);
}

function shouldRequestFullPreviewAsset(asset: ReceivedAsset, original: boolean) {
  const format = cssToken(asset.format || extensionOf(asset.filename));
  return shouldRequestFullPreview(format, original);
}

function shouldRequestOriginalPreviewAsset(asset: ReceivedAsset) {
  const format = cssToken(asset.format || extensionOf(asset.filename));
  return shouldRequestOriginalPreview(format);
}

function extensionOf(path: string | null | undefined) {
  return path?.split(/[./\\]/).filter(Boolean).at(-1) ?? "";
}

function absolutePathOrNull(path: string | null | undefined) {
  if (!path) return null;
  return /^[a-zA-Z]:[\\/]/.test(path) || path.startsWith("\\\\") || path.startsWith("/") ? path : null;
}

function updateLoupeFromPointer(event: PointerEvent, group: ReceivedAssetGroup, maxEdge = currentThumbnailMaxEdge(), original = false) {
  if (original) {
    void ensureOriginalPreviewForGroup(group, "visible");
  }
  if (!previewUrlForGroup(group, maxEdge, original)) return;
  const next = loupeFromPointer(event, group, maxEdge, original);
  const previous = state.loupe;
  if (!previous || previous.groupId !== next.groupId) {
    scheduleLoupe(next, group);
    return;
  }
  state.loupe = next;
  applyLoupeDom(next, group);
}

function handleLoupeWheel(event: WheelEvent, group: ReceivedAssetGroup, maxEdge = currentThumbnailMaxEdge(), original = false) {
  const loupe = state.loupe;
  if (!loupe || loupe.groupId !== groupIdentity(group)) {
    return;
  }
  if (performance.now() - loupe.startedAtMs < LOUPE_ZOOM_READY_MS) {
    return;
  }
  event.preventDefault();
  event.stopPropagation();
  const nextZoom = clamp(loupe.zoom + (event.deltaY < 0 ? 0.2 : -0.2), 1.5, 4);
  const next = {
    ...loupeFromPointer(event, group, maxEdge, original),
    zoom: nextZoom,
    startedAtMs: loupe.startedAtMs,
  };
  state.loupe = next;
  applyLoupeDom(next, group);
}

function clearLoupeIfFloating() {
  clearPendingLoupe();
  if (state.loupe) {
    state.loupe = null;
    render();
  }
}

function scheduleLoupe(next: LoupeState, group: ReceivedAssetGroup) {
  if (state.loupe && state.loupe.groupId !== next.groupId) {
    state.loupe = null;
    render();
  }
  pendingLoupeState = next;
  pendingLoupeGroup = group;
  if (pendingLoupeGroupId === next.groupId && pendingLoupeTimer) {
    return;
  }
  clearPendingLoupe();
  pendingLoupeGroupId = next.groupId;
  pendingLoupeState = next;
  pendingLoupeGroup = group;
  pendingLoupeTimer = window.setTimeout(() => {
    if (!pendingLoupeState || !pendingLoupeGroup) return;
    state.loupe = {
      ...pendingLoupeState,
      startedAtMs: performance.now(),
    };
    clearPendingLoupe();
    render();
  }, LOUPE_SHOW_DELAY_MS);
}

function clearPendingLoupe() {
  if (pendingLoupeTimer) {
    window.clearTimeout(pendingLoupeTimer);
  }
  pendingLoupeTimer = null;
  pendingLoupeGroupId = null;
  pendingLoupeState = null;
  pendingLoupeGroup = null;
}

function applyLoupeDom(loupe: LoupeState, group: ReceivedAssetGroup) {
  const overlay = document.querySelector<HTMLElement>(".loupe-overlay");
  if (overlay) {
    positionLoupeOverlay(overlay, loupe);
  }

  const crop = document.querySelector<HTMLElement>(".loupe-crop");
  if (crop) {
    setPreviewBackground(crop, group, loupe.maxEdge, loupe.original);
    crop.style.backgroundPosition = `${loupe.x * 100}% ${loupe.y * 100}%`;
    crop.style.backgroundSize = `${loupe.zoom * 100}% auto`;
  }

  const zoomLabel = document.querySelector<HTMLElement>(".loupe-caption strong");
  if (zoomLabel) zoomLabel.textContent = `${loupe.zoom.toFixed(1)}x`;
}

function positionLoupeOverlay(overlay: HTMLElement, loupe: LoupeState) {
  const overlayWidth = Math.min(520, window.innerWidth - 16);
  const overlayHeight = 360;
  const gap = 18;
  const rightSide = loupe.clientX + gap + overlayWidth;
  const preferredLeft = rightSide > window.innerWidth - 8 ? loupe.clientX - overlayWidth - gap : loupe.clientX + gap;
  overlay.style.left = `${clamp(preferredLeft, 8, window.innerWidth - overlayWidth - 8)}px`;
  overlay.style.top = `${clamp(loupe.clientY - 96, 8, window.innerHeight - overlayHeight - 8)}px`;
}

function loupeFromPointer(event: PointerEvent | WheelEvent, group: ReceivedAssetGroup, maxEdge: number, original = false): LoupeState {
  const target = event.currentTarget as HTMLElement;
  const groupId = groupIdentity(group);
  const current = state.loupe;
  const point = normalizedPreviewPoint(target, event.clientX, event.clientY);
  return {
    groupId,
    x: point.x,
    y: point.y,
    clientX: event.clientX,
    clientY: event.clientY,
    zoom: state.loupe?.zoom ?? 2,
    maxEdge,
    original,
    startedAtMs: current?.groupId === groupId ? current.startedAtMs : performance.now(),
  };
}

function ensureOriginalPreviewForGroup(group: ReceivedAssetGroup, priority: ThumbnailPriority) {
  const asset = previewAssetForGroup(group);
  const localPath = asset ? localPreviewablePath(asset) : null;
  if (!asset || !localPath || !shouldRequestOriginalPreviewAsset(asset)) {
    return null;
  }
  return originalPreviewUrlForPath(localPath, priority);
}

function normalizedPreviewPoint(target: HTMLElement, clientX: number, clientY: number) {
  const rect = target.getBoundingClientRect();
  const image = target.querySelector<HTMLImageElement>(":scope > img.preview-image");
  const naturalWidth = image?.naturalWidth || rect.width || 1;
  const naturalHeight = image?.naturalHeight || rect.height || 1;
  const objectFit = image ? getComputedStyle(image).objectFit : "cover";
  const imageRect = objectFit === "cover"
    ? coverImageRect({ left: rect.left, top: rect.top, width: rect.width, height: rect.height }, { width: naturalWidth, height: naturalHeight })
    : containedImageRect({ left: rect.left, top: rect.top, width: rect.width, height: rect.height }, { width: naturalWidth, height: naturalHeight });
  return normalizedPointInRect(imageRect, clientX, clientY);
}

function coverImageRect(container: { left: number; top: number; width: number; height: number }, image: { width: number; height: number }) {
  const safeContainerWidth = Math.max(1, container.width);
  const safeContainerHeight = Math.max(1, container.height);
  const imageWidth = Math.max(1, image.width);
  const imageHeight = Math.max(1, image.height);
  const scale = Math.max(safeContainerWidth / imageWidth, safeContainerHeight / imageHeight);
  const width = imageWidth * scale;
  const height = imageHeight * scale;
  return {
    left: container.left + (safeContainerWidth - width) / 2,
    top: container.top + (safeContainerHeight - height) / 2,
    width,
    height,
  };
}

function normalizedPointInRect(rect: { left: number; top: number; width: number; height: number }, clientX: number, clientY: number) {
  return {
    x: clamp((clientX - rect.left) / Math.max(1, rect.width), 0, 1),
    y: clamp((clientY - rect.top) / Math.max(1, rect.height), 0, 1),
  };
}

function formatPairLabel(group: ReceivedAssetGroup) {
  if (group.raw && group.jpeg) return "RAW+JPG";
  if (group.raw) return "RAW";
  if (group.jpeg) return "JPG";
  if (group.video) return "MOV";
  return readable(group.primary.format);
}

function statusDot(value: string) {
  return el("span", `status-dot ${cssToken(value)}`);
}

function scanTransferDot(health: string) {
  switch (health) {
    case "ready":
      return "available";
    case "working":
      return "changed";
    case "failed":
      return "missing";
    default:
      return "neutral";
  }
}

function lanSyncTransferDot() {
  switch (state.lanSyncPhase) {
    case "done":
      return "available";
    case "discovering":
    case "syncing":
      return "changed";
    case "failed":
      return "missing";
    default:
      return "neutral";
  }
}

function lanSyncTransferLabel() {
  if (state.lanSyncPhase === "discovering") return "discovering";
  if (state.lanSyncPhase === "syncing") return "matching";
  if (state.lanSyncPhase === "failed") return "failed";
  const summary = state.lanSyncSummary;
  if (!summary) return state.lanSyncSources.length ? `${state.lanSyncSources.length} source` : "no source";
  const applied =
    summary.applied_user_marks +
    summary.applied_model_evaluations +
    summary.applied_selection_recommendations;
  return `${summary.matched_groups} matched / ${applied} applied / ${summary.unresolved_records} unresolved`;
}

function evaluationDot(group: ReceivedAssetGroup) {
  if (group.technical_defects.length) return "missing";
  if (typeof group.model_score === "number") return "available";
  const technical = cssToken(group.technical_gate_status ?? group.technical_status ?? "pending");
  const model = cssToken(group.model_status ?? "pending");
  if (["failed", "reject"].includes(technical) || ["failed", "reject"].includes(model)) return "missing";
  if (["pass", "ready", "completed"].includes(technical) || ["ready", "completed"].includes(model)) return "available";
  return "changed";
}

function compactEvaluationLabel(group: ReceivedAssetGroup) {
  if (group.technical_defects.length) return "需复核";
  if (typeof group.model_score === "number") return `${group.model_score} ${readable(group.model_tier ?? "score")}`;
  const technical = cssToken(group.technical_gate_status ?? group.technical_status ?? "pending");
  const model = cssToken(group.model_status ?? "pending");
  if (["failed", "reject"].includes(technical) || ["failed", "reject"].includes(model)) return "评价失败";
  if (["pass", "ready", "completed"].includes(technical) || ["ready", "completed"].includes(model)) return "已评价";
  return "待评价";
}

function clamp(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value));
}

function canStartScan() {
  return getScanStartBlocker() === null;
}

function getScanStartBlocker() {
  return scanStartBlocker({
    hasProject: Boolean(state.selectedProjectId),
    hasRootPath: Boolean(state.rootPath),
    busy: Boolean(state.busy),
    scanPhase: state.scan?.phase ?? null,
  });
}

function scanBlockerCopy(blocker: ScanStartBlocker) {
  switch (blocker) {
    case "project":
      return "先创建或选择一个项目。";
    case "folder":
      return "先为项目绑定照片文件夹。";
    case "busy":
      return `正在处理 ${state.busy}，稍后再试。`;
    case "active_scan":
      return "当前项目正在扫描。";
  }
}

function compactError(error: string | null) {
  if (!error) return null;
  return error.length > 130 ? `${error.slice(0, 127)}...` : error;
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

function renderFormatBadges(group: ReceivedAssetGroup) {
  const badges = el("div", "format-badges");
  if (group.jpeg) append(badges, el("span", "format-badge jpeg", "JPG"));
  if (group.raw) append(badges, el("span", "format-badge raw", "RAW"));
  if (group.video) append(badges, el("span", "format-badge video", "MOV"));
  return badges;
}

function renderMarks(group: ReceivedAssetGroup) {
  const marks = el("div", "marks");
  if (group.user_marks.favorite) append(marks, el("span", "mark", "已收藏"));
  if (group.user_marks.marked) append(marks, el("span", "mark", "已标记"));
  if (group.is_model_select) append(marks, el("span", "mark model", "AI 推荐"));
  if (!marks.childElementCount) return null;
  return marks;
}

function checkResultList(rows: Array<[string, string, string]>) {
  const list = el("div", "check-result-list");
  for (const [label, value, status] of rows) {
    append(
      list,
      append(
        el("div", "check-result-row"),
        statusDot(checkResultDot(status)),
        el("span", "check-label", label),
        el("strong", "", value),
      ),
    );
  }
  return list;
}

function checkResultDot(status: string) {
  const token = cssToken(status);
  if (["available", "pass", "ready", "completed", "evaluated"].includes(token)) return "available";
  if (["missing", "failed", "reject"].includes(token)) return "missing";
  if (["changed", "pending", "queued", "setup"].includes(token)) return "changed";
  return "neutral";
}

function kvGrid(rows: Array<[string, string]>) {
  const grid = el("div", "kv-grid");
  for (const [label, value] of rows) {
    append(grid, append(el("div", "kv"), el("span", "kv-label", label), el("strong", "", value)));
  }
  return grid;
}

function currentIntelligenceSetup() {
  return intelligenceSetupState(state.intelligenceProviders, state.promptPacks, state.intelligenceSettings);
}

function intelligenceLine(label: string, value: string) {
  return append(el("div", "intelligence-line"), el("span", "", label), el("strong", "", value));
}

function renderIntelligenceField(label: string, control: HTMLElement, note = "") {
  const field = append(el("label", "intelligence-field"), el("span", "", label), control);
  if (note) {
    append(field, el("small", "", note));
  }
  return field;
}

function settingsSectionHead(title: string, note = "") {
  const head = append(el("div", "settings-section-head"), el("h3", "", title));
  if (note) {
    append(head, el("p", "", note));
  }
  return head;
}

function renderToggleRow(label: string, checked: boolean, onChange: (checked: boolean) => void) {
  const input = el("input", "") as HTMLInputElement;
  input.type = "checkbox";
  input.checked = checked;
  input.addEventListener("change", () => onChange(input.checked));
  return append(el("label", "toggle-row"), append(el("span", ""), el("strong", "", label)), input);
}

function selectControl(value: string, options: Array<[string, string]>, onChange: (value: string) => void) {
  const select = el("select", "select-control") as HTMLSelectElement;
  select.value = value;
  for (const [optionValue, label] of options) {
    const option = el("option", "", label) as HTMLOptionElement;
    option.value = optionValue;
    option.selected = optionValue === value;
    append(select, option);
  }
  select.addEventListener("change", () => onChange(select.value));
  return select;
}

function compactMetric(label: string, value: string) {
  return append(el("div", "compact-metric"), el("span", "", label), el("strong", "", value));
}

function statusChip(value: string, kind: string) {
  return el("span", `status-chip ${kind} ${cssToken(value)}`, readable(value));
}

function commandButton(label: string, className: string, onClick: (event: MouseEvent) => void, disabled = false) {
  const node = el("button", className, label);
  node.type = "button";
  node.disabled = disabled;
  node.addEventListener("click", (event) => onClick(event));
  return node;
}

function textInput(value: string, placeholder: string, onInput: (value: string) => void) {
  const node = el("input", "text-input") as HTMLInputElement;
  node.value = value;
  node.placeholder = placeholder;
  node.addEventListener("input", () => onInput(node.value));
  return node;
}

function passwordInput(value: string, placeholder: string, onInput: (value: string) => void) {
  const node = el("input", "text-input") as HTMLInputElement;
  node.type = "password";
  node.value = value;
  node.placeholder = placeholder;
  node.addEventListener("input", () => onInput(node.value));
  return node;
}

function numberInput(value: number, min: number, max: number, onInput: (value: number) => void) {
  const node = el("input", "text-input") as HTMLInputElement;
  node.type = "number";
  node.min = String(min);
  node.max = String(max);
  node.value = String(value);
  node.addEventListener("input", () => {
    const parsed = Number(node.value);
    if (!Number.isFinite(parsed)) return;
    onInput(Math.min(max, Math.max(min, parsed)));
  });
  return node;
}

function textAreaInput(value: string, placeholder: string, onInput: (value: string) => void) {
  const node = el("textarea", "textarea-input") as HTMLTextAreaElement;
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
  const normalized = value.replace(/_/g, " ").trim().toLowerCase();
  const mapped = READABLE_LABELS[normalized];
  if (mapped) return mapped;
  return value
    .replace(/_/g, " ")
    .replace(/([a-z])([A-Z])/g, "$1 $2")
    .replace(/\b\w/g, (letter) => letter.toUpperCase());
}

const READABLE_LABELS: Record<string, string> = {
  action: "运动",
  all: "全部",
  available: "可用",
  auto: "自动",
  changed: "已变化",
  completed: "已完成",
  custom: "自定义",
  disabled: "未启用",
  evaluated: "已评价",
  failed: "失败",
  general: "通用",
  indexed: "已索引",
  landscape: "风光",
  loose: "宽松",
  missing: "缺失",
  none: "无",
  "not generated": "未生成",
  openai: "OpenAI",
  pass: "通过",
  pending: "待处理",
  portrait: "人像",
  model: "AI",
  prompt: "选片规则",
  provider: "AI 服务",
  queued: "已排队",
  ready: "就绪",
  reject: "淘汰",
  scanning: "扫描中",
  score: "分",
  setup: "待设置",
  standard: "标准",
  strict: "严格",
  "model select": "AI 推荐",
  "technical pending": "质量待查",
};

function promptDraftModeLabel(mode: PromptDraft["mode"]) {
  if (mode === "create") return "新建选片规则";
  if (mode === "fork") return "复制选片规则";
  return "编辑选片规则";
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
