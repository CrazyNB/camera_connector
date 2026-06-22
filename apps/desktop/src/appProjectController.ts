import { open } from "@tauri-apps/plugin-dialog";
import { api } from "./desktopApi";
import { selectLanProjectSnapshotSource } from "./lanProjectSync";
import { errorMessage } from "./presentation";
import { resetLightTableVirtualSignature } from "./lightTable";
import {
  ASSET_PAGE_LIMIT,
  THUMBNAIL_INITIAL_WARMUP_LIMIT,
  state,
} from "./appState";
import type {
  AppState,
  Project,
  ReceivedAssetGroup,
  SettingsPanel,
  SubjectAssessment,
} from "./appTypes";

type WithBusy = <T>(label: string, task: () => Promise<T>) => Promise<T | null>;

export function createProjectController(options: {
  state: AppState;
  render: () => void;
  selectedProject: () => Project | null;
  setStatus: (message: string, error?: string | null) => void;
  withBusy: WithBusy;
  getScanStartBlocker: () => unknown;
  warmThumbnailsForGroups: (groups: ReceivedAssetGroup[]) => void;
}) {
  const {
    render,
    selectedProject,
    setStatus,
    withBusy,
    getScanStartBlocker,
    warmThumbnailsForGroups,
  } = options;

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
      state.status = "椤圭洰宸茶浇鍏?";
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
    resetLightTableVirtualSignature();
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
      setStatus("鍒涘缓椤圭洰", "璇疯緭鍏ラ」鐩悕绉般€?");
      return;
    }
    if (!rootPath) {
      setStatus("鍒涘缓椤圭洰", "璇烽€夋嫨鐓х墖鏂囦欢澶广€?");
      return;
    }
    const project = await withBusy("鍒涘缓椤圭洰", () => api.createProject(name));
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
    state.status = "鏂囦欢澶瑰凡缁戝畾";
    render();
    await startScan();
  }

  async function selectProject(projectId: string) {
    await withBusy("閫夋嫨椤圭洰", async () => {
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
      state.status = "宸查€夋嫨鐓х墖鏂囦欢澶?";
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
      state.status = "鏂囦欢澶瑰凡缁戝畾";
      render();
      if (!getScanStartBlocker()) {
        await startScan();
      }
    }
  }

  async function startScan() {
    const projectId = state.selectedProjectId;
    if (!projectId) {
      setStatus("Start scan", "Create or select a project first.");
      return;
    }
    if (!state.rootPath) {
      setStatus("Start scan", "Choose a photo folder first.");
      return;
    }
    const scan = await withBusy("寮€濮嬫壂鎻?", () => api.startProjectScan(projectId, state.rootPath));
    if (scan) {
      state.scan = scan;
      clearLanSyncState();
      state.selectedGroupId = null;
      state.selectedGroup = null;
      state.groupDetail = [];
      resetBoardViewport();
      state.status = "鎵弿宸叉帓闃?";
      render();
    }
  }

  return {
    loadProjects,
    refreshCurrentProject,
    syncLanProjectContext,
    clearLanSyncState,
    loadSubjectAssessmentsForGroups,
    refreshPromptPackLists,
    openSettingsPanel,
    resetBoardViewport,
    syncSelectedGroup,
    createProject,
    selectProject,
    chooseProjectFolderDraft,
    chooseFolder,
    startScan,
  };
}
