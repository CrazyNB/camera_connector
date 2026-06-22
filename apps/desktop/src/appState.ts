import type { AppState } from "./appTypes";
import { resetViewerTransform } from "./viewerMode";

export const state: AppState = {
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
  cvProgress: null,
  boardScrollTop: 0,
  boardWidth: 0,
  assetPageLoading: false,
};

export const ASSET_PAGE_LIMIT = 96;
export const LOUPE_SHOW_DELAY_MS = 1000;
export const LOUPE_OVERLAY_MAX_WIDTH = 960;
export const LOUPE_OVERLAY_HEIGHT = 620;
export const GRID_GAP = 12;
export const VIRTUAL_OVERSCAN_ROWS = 2;
export const THUMBNAIL_MIN_EDGE = 640;
export const THUMBNAIL_MAX_EDGE = 1280;
export const VIEWER_PREVIEW_MAX_EDGE = 1280;
export const THUMBNAIL_CONCURRENCY = 3;
export const ORIGINAL_PREVIEW_CONCURRENCY = 4;
export const THUMBNAIL_PREFETCH_ROWS = 3;
export const THUMBNAIL_INITIAL_WARMUP_LIMIT = 48;
export const THUMBNAIL_PAGE_WARMUP_LIMIT = 24;
export const THUMBNAIL_WARMUP_DELAY_MS = 180;
export const THUMBNAIL_BATCH_SIZE = 8;
export const THUMBNAIL_SCROLL_IDLE_MS = 300;
export const LOUPE_ZOOM_READY_MS = 1000;
