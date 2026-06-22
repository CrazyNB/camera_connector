import type { LanProjectSnapshotSource } from "./lanProjectSync";
import type {
  ModelProviderSettings,
  PromptDraft,
  ProjectEvaluationSettings,
  PromptPack,
  SaveModelProviderSettingsRequest,
} from "./intelligence";
import type { ViewerTransform } from "./viewerMode";

export type Project = {
  project_id: string;
  name: string;
  slug: string;
  status: string;
  created_at_ms: number;
  updated_at_ms: number;
};

export type DesktopScanRun = {
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

export type StoredObjectLocation = unknown;

export type ReceivedAsset = {
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

export type AssetUserMarks = {
  favorite: boolean;
  marked: boolean;
};

export type ReceivedAssetBurstSummary = {
  burst_group_id: string;
  member_count: number;
  recommendation_status: string;
  best_asset_group_id?: string | null;
  best_score?: number | null;
};

export type ReceivedAssetTechnicalDefectSummary = {
  defect_type: string;
  severity: string;
  confidence: number;
  reason?: string | null;
};

export type ReceivedAssetGroup = {
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

export type AssetGroupSummary = {
  group_count: number;
  asset_count: number;
  groups_with_jpeg: number;
  groups_with_raw: number;
  groups_with_video: number;
};

export type AssetGroupPage = {
  groups: ReceivedAssetGroup[];
  summary: AssetGroupSummary;
  offset: number;
  limit: number;
  total_groups: number;
  has_more: boolean;
};

export type ThumbnailResponse = {
  path: string;
  cached: boolean;
  quality?: ThumbnailQuality;
};

export type OriginalPreviewResponse = {
  path: string;
  cached: boolean;
  direct_source: boolean;
};

export type ThumbnailQuality = "fast" | "full";
export type PreviewImageQuality = ThumbnailQuality | "original";

export type SyncProjectSnapshotResponse = {
  matched_assets: number;
  matched_groups: number;
  applied_user_marks: number;
  applied_model_evaluations: number;
  applied_selection_recommendations: number;
  unresolved_records: number;
  ambiguous_records: number;
};

export type LanSyncPhase = "idle" | "discovering" | "syncing" | "done" | "failed";

export type PreviewImageOptions = {
  maxEdge?: number;
  original?: boolean;
  eager?: boolean;
};

export type SelectGroupOptions = {
  preserveViewerTransform?: boolean;
};

export type ThumbnailBatchItem = {
  source_path: string;
  path?: string | null;
  cached: boolean;
  error?: string | null;
};

export type ThumbnailBatchResponse = {
  thumbnails: ThumbnailBatchItem[];
};

export type StoredAsset = {
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

export type AnalysisDrainSummary = {
  claimed_count: number;
  completed_count: number;
  failed_count: number;
};

export type SelectionRecommendation = {
  status: string;
};

export type EnqueueModelEvaluationResponse = {
  enqueued_count: number;
};

export type DesktopCvAssessmentResponse = {
  assessed_count: number;
  failed_count: number;
  skipped_count: number;
  subject_count: number;
};

export type DesktopCvAssessmentProgress = DesktopCvAssessmentResponse & {
  project_id: string;
  scope: "project" | "group";
  total_count: number;
  current_group_id?: string | null;
};

export type SubjectAssessment = {
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

export type SubjectRegion = {
  kind?: string;
  x?: number;
  y?: number;
  width?: number;
  height?: number;
  w?: number;
  h?: number;
};

export type SubjectSignals = {
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

export type DesktopError = {
  code?: string;
  message?: string;
};

export type SourceFilter = "all" | "available" | "changed" | "missing";

export type ViewFilter = "light-table" | "needs-work" | "missing";

export type LayoutMode = "grid" | "viewer";

export type SettingsPanel = "project" | "global";

export type LoupeState = {
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

export type ViewerCarryoverImage = {
  url: string;
};

export type AppState = {
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
  cvProgress: DesktopCvAssessmentProgress | null;
  boardScrollTop: number;
  boardWidth: number;
  assetPageLoading: boolean;
};
