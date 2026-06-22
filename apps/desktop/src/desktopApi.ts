import { invoke } from "@tauri-apps/api/core";
import type {
  AnalysisDrainSummary,
  AssetGroupPage,
  AssetUserMarks,
  DesktopCvAssessmentResponse,
  DesktopScanRun,
  EnqueueModelEvaluationResponse,
  OriginalPreviewResponse,
  Project,
  SelectionRecommendation,
  StoredAsset,
  SubjectAssessment,
  SyncProjectSnapshotResponse,
  ThumbnailBatchResponse,
  ThumbnailQuality,
  ThumbnailResponse,
} from "./appTypes";
import type {
  CreatePromptPackRequest,
  ForkPromptPackRequest,
  ModelProviderSettings,
  ProjectEvaluationSettings,
  PromptPack,
  SaveModelProviderSettingsRequest,
  SavePromptPackRequest,
} from "./intelligence";
import type { LanProjectSnapshotSource } from "./lanProjectSync";

const DEFAULT_ASSET_PAGE_LIMIT = 96;
const DEFAULT_THUMBNAIL_MAX_EDGE = 1280;

export const api = {
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
  getAssetPage(projectId: string, offset = 0, limit = DEFAULT_ASSET_PAGE_LIMIT): Promise<AssetGroupPage> {
    return invoke("get_project_asset_page", { request: { project_id: projectId, offset, limit } });
  },
  getAssetThumbnail(
    sourcePath: string,
    maxEdge = DEFAULT_THUMBNAIL_MAX_EDGE,
    quality: ThumbnailQuality = "fast",
  ): Promise<ThumbnailResponse> {
    return invoke("get_asset_thumbnail", { request: { source_path: sourcePath, max_edge: maxEdge, quality } });
  },
  getAssetThumbnails(
    sourcePaths: string[],
    maxEdge = DEFAULT_THUMBNAIL_MAX_EDGE,
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
  runDesktopCvAssessment(
    projectId: string,
    limit = 1000,
    assetGroupIds?: string[],
  ): Promise<DesktopCvAssessmentResponse> {
    return invoke("run_desktop_cv_assessment", {
      request: { project_id: projectId, limit, asset_group_ids: assetGroupIds },
    });
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
