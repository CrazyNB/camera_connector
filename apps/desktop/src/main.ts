import { convertFileSrc } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  resetViewerTransform,
  shouldPreserveViewerTransformForSelection,
  viewerBurstWarmWindow,
  viewerCurrentGroup,
  viewerQueueWindow,
  viewerReplacementAfterDelete,
} from "./viewerMode";
import {
  groupIdentity,
  uniqueGroupsByIdentity,
} from "./groupSelectors";
import { createAppSelectors } from "./appSelectors";
import { currentViewerMainImageCarryover as readCurrentViewerMainImageCarryover } from "./viewerCarryover";
import {
  renderFaceRiskOverlay as renderFaceRiskOverlayForGroup,
  syncAllFaceRiskLayers as syncAllFaceRiskLayersWith,
  syncFaceRiskLayer as syncFaceRiskLayerWith,
} from "./faceRiskOverlay";
import { createIntelligenceManagementRenderer } from "./intelligenceManagement";
import { createAppActions } from "./appActions";
import { createProjectController } from "./appProjectController";
import { previewProgress } from "./previewStatus";
import {
  configurePreviewQueue,
  originalPreviewUrlForPath,
} from "./previewQueue";
import { createPreviewRenderer } from "./previewRenderer";
import { createPreviewWarmup } from "./previewWarmup";
import { createLoupeInteraction } from "./loupeInteraction";
import { createViewerInteraction } from "./viewerInteraction";
import {
  renderViewerBurstQueue,
  renderViewerInspector,
  renderViewerRightRail,
  type ViewerChromeOptions,
} from "./viewerChrome";
import {
  renderViewerFilmstrip,
  renderViewerStage,
  type ViewerStageOptions,
} from "./viewerStage";
import {
  renderViewerActionDock as renderViewerActionDockComponent,
  type ViewerActionDockOptions,
} from "./viewerActionDock";
import {
  renderFiltersPanel,
  renderIntelligencePanel,
  renderSourcePanel,
  renderTransferPanel,
  renderViewerLeftRail,
  renderViewsPanel,
  type SidebarPanelOptions,
} from "./sidebarPanels";
import { renderTopBar, type TopBarOptions } from "./topBar";
import { renderSettingsDrawer, type SettingsDrawerOptions } from "./settingsDrawer";
import {
  getThumbnailScrolling,
  handleLightTableWheel,
  renderGroupBoard,
  renderLightTableToolbar,
  resetLightTableVirtualSignature,
  updateActiveVirtualBoard,
  type LightTableOptions,
} from "./lightTable";
import {
  renderGroupCard as renderAssetGroupCard,
  type AssetGroupCardOptions,
} from "./assetGroupCard";
import {
  renderInspector,
  type InspectorPanelOptions,
} from "./inspectorPanel";
import {
  renderWorkbenchEmptyState,
  renderWorkbenchSurface,
  type WorkbenchViewOptions,
} from "./workbenchView";
import {
  renderLoupeOverlay,
  type LoupeOverlayOptions,
} from "./loupeOverlay";
import { createWorkflowStatusController } from "./workflowStatus";
import { renderProjectCreateForm } from "./projectCreateForm";
import {
  originalImageWarmCache,
  previewStageForGroup,
  thumbnailBatchPending,
  thumbnailQueue,
  thumbnailQueued,
  thumbnailUrlCache,
  type ThumbnailPriority,
} from "./previewCache";
import {
  localPreviewablePath,
  previewAssetForGroup,
  shouldRequestOriginalPreviewAsset,
} from "./previewAssets";
import { api } from "./desktopApi";
import {
  ASSET_PAGE_LIMIT,
  THUMBNAIL_PAGE_WARMUP_LIMIT,
  VIEWER_PREVIEW_MAX_EDGE,
  state,
} from "./appState";
import type {
  DesktopCvAssessmentProgress,
  LoupeState,
  ReceivedAssetGroup,
  SelectGroupOptions,
} from "./appTypes";
import {
  errorMessage,
  membersOf,
  scanIsActive,
} from "./presentation";
import {
  append,
  el,
} from "./domHelpers";
import "./styles.css";

const app = document.querySelector<HTMLDivElement>("#app");
if (!app) {
  throw new Error("app root not found");
}
const appRoot = app;

const previewRenderer = createPreviewRenderer({
  clearViewerCarryover,
  currentPreviewProgressLabel: () => currentPreviewProgress().label,
  getThumbSize: () => state.thumbSize,
  onPreviewImageSettled: (node) => {
    syncFaceRiskLayer(node);
    if (node.classList.contains("viewer-main-preview")) {
      applyViewerTransformToNode(node);
    }
  },
  requestAnimationFrame: (callback) => window.requestAnimationFrame(callback),
});
const {
  appendPreviewImage,
  currentThumbnailMaxEdge,
  insertPreviewImage,
  previewLocalPathForGroup,
  previewTooltipForGroup,
  previewUrlForGroup,
  refreshPreviewProgressDom,
  renderPreviewStatusBadge,
  setPreviewBackground,
  syncPreviewStatusBadge,
} = previewRenderer;

const { warmThumbnailsForGroups, warmOriginalsForGroups } = createPreviewWarmup({
  currentThumbnailMaxEdge,
  previewLocalPathForGroup,
});

configurePreviewQueue({
  insertPreviewImage,
  refreshPreviewProgressDom,
  syncPreviewStatusBadge,
  getThumbnailScrolling,
});

const loupeInteraction = createLoupeInteraction({
  applyPreviewBackground: setPreviewBackground,
  ensureOriginalPreviewForGroup,
  getLoupe: () => state.loupe,
  groupIdentity,
  hasPreviewUrlForGroup: (group, maxEdge, original) => Boolean(previewUrlForGroup(group, maxEdge, original)),
  render,
  setLoupe: (loupe) => {
    state.loupe = loupe;
  },
});

const viewerInteraction = createViewerInteraction({
  getCarryoverImage: () => state.viewerCarryoverImage,
  getTransform: () => state.viewerTransform,
  setCarryoverImage: (image) => {
    state.viewerCarryoverImage = image;
  },
  setTransform: (transform) => {
    state.viewerTransform = transform;
  },
});

const {
  allGroups,
  selectedProject,
  filteredGroups,
  displayGroupsFor,
  burstMembersOf,
  groupByIdentity,
} = createAppSelectors(state, selectGroup);

const {
  lanSyncTransferDot,
  lanSyncTransferLabel,
  canStartScan,
  getScanStartBlocker,
  setStatus,
  withBusy,
  currentIntelligenceSetup,
} = createWorkflowStatusController(render);

const projectController = createProjectController({
  state,
  render,
  selectedProject,
  setStatus,
  withBusy,
  getScanStartBlocker,
  warmThumbnailsForGroups,
});
const {
  loadProjects,
  refreshCurrentProject,
  syncLanProjectContext,
  refreshPromptPackLists,
  openSettingsPanel,
  resetBoardViewport,
  syncSelectedGroup,
  createProject,
  selectProject,
  chooseProjectFolderDraft,
  chooseFolder,
  startScan,
} = projectController;

const actions = createAppActions({
  state,
  render,
  refreshCurrentProject,
  refreshPromptPackLists,
  filteredGroups,
  allGroups,
  setStatus,
  withBusy,
  currentIntelligenceSetup,
});
const {
  runAnalysisJobs,
  runGroupAnalysis,
  recommendBurst,
  recommendProject,
  saveIntelligenceSettings,
  evaluateGroupWithModel,
  evaluateLoadedGroupsWithModel,
  saveProviderDraft,
  deleteProvider,
  savePromptDraft,
  deletePromptPack,
  removeFromBurst,
} = actions;

const intelligenceManagement = createIntelligenceManagementRenderer({
  state,
  render,
  saveIntelligenceSettings,
  saveProviderDraft,
  deleteProvider,
  savePromptDraft,
  deletePromptPack,
});
const { renderProviderManagement, renderPromptManagement } = intelligenceManagement;

void bootstrap().catch((error) => {
  setStatus("鍚姩澶辫触", errorMessage(error));
});

async function bootstrap() {
  await loadProjects();
  await refreshCurrentProject(false);
  await listen<boolean>("desktop-scan-finished", async (event) => {
    setStatus(event.payload ? "鎵弿瀹屾垚" : "鎵弿澶辫触");
    await refreshCurrentProject(false);
    if (event.payload) {
      await syncLanProjectContext();
    }
  });
  await listen<DesktopCvAssessmentProgress>("desktop-cv-assessment-progress", (event) => {
    if (event.payload.project_id !== state.selectedProjectId) {
      return;
    }
    state.cvProgress = event.payload;
    render();
  });
  window.addEventListener("resize", () => {
    resetLightTableVirtualSignature();
    updateActiveVirtualBoard(lightTableOptions());
    syncAllFaceRiskLayers();
  });
  window.setInterval(() => {
    if (state.scan && scanIsActive(state.scan.phase)) {
      void refreshCurrentProject(false);
    }
  }, 1400);
  render();
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
    updateActiveVirtualBoard(lightTableOptions());
  } catch (error) {
    setStatus("杞藉叆鐓х墖缁?", errorMessage(error));
  } finally {
    state.assetPageLoading = false;
  }
}

async function selectGroup(group: ReceivedAssetGroup, options: SelectGroupOptions = {}) {
  const keepViewerTransform = shouldPreserveViewerTransformForSelection(
    state.selectedGroup,
    group,
    Boolean(options.preserveViewerTransform),
  );
  const carryoverImage = keepViewerTransform ? readCurrentViewerMainImageCarryover(state.viewerCarryoverImage) : null;
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
  viewerInteraction.clearDrag();
  render();
  const projectId = state.selectedProjectId;
  if (projectId && group.group_id) {
    const loaded = await withBusy("杞藉叆鐓х墖缁?", () =>
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

async function toggleFavorite(targetGroup = state.selectedGroup) {
  const projectId = state.selectedProjectId;
  const group = targetGroup;
  if (!projectId || !group?.group_id) {
    return;
  }
  await withBusy("淇濆瓨鏀惰棌鏍囪", () =>
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
  await withBusy("淇濆瓨鏍囪", () =>
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
    `鍒犻櫎鐓х墖缁?${group.group_key}锛焅n\n杩欎細浠庨」鐩腑绉婚櫎璁板綍锛屽苟鍒犻櫎 ${fileCount} 涓師鍥炬枃浠躲€傛鎿嶄綔涓嶅彲鎾ら攢銆俙`,
  );
  if (!confirmed) {
    return;
  }
  const replacementGroup = viewerReplacementAfterDelete(filteredGroups(), group);
  const deleted = await withBusy("鍒犻櫎鐓х墖缁?", () => api.deleteAssetGroup(projectId, group.group_id as string));
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
  state.selectedGroupId = replacementGroup?.group_id ?? null;
  state.selectedGroup = replacementGroup;
  state.groupDetail = [];
  state.loupe = null;
  state.status = `宸插垹闄?${group.group_key}`;
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

function render() {
  appRoot.replaceChildren(renderShell());
}

function renderShell() {
  return append(el("div", state.layoutMode === "viewer" ? "app-shell is-viewer-focus" : "app-shell"), renderTopBar(topBarOptions()), renderWorkflow());
}

function currentPreviewProgress() {
  return previewProgress(displayGroupsFor(allGroups()).map((group) => previewStageForGroup(group, currentThumbnailMaxEdge())));
}

function topBarOptions(): TopBarOptions {
  return {
    selectedProject,
    renderProjectCreate,
    selectProject,
    openSettingsPanel,
    currentPreviewProgress,
    render,
  };
}

function renderWorkflow() {
  const layout = el("main", state.layoutMode === "viewer" ? "workflow-layout is-viewer-focus" : "workflow-layout");
  append(layout, renderWorkbenchSidebar(), renderWorkbenchSurface(workbenchViewOptions()));
  if (state.settingsPanel) {
    append(layout, renderSettingsDrawer(state.settingsPanel, settingsDrawerOptions()));
  }
  return layout;
}

function workbenchViewOptions(): WorkbenchViewOptions {
  return {
    renderInspector: (group) => renderInspector(group, inspectorPanelOptions()),
    renderLightTable,
    renderProjectCreate,
    chooseFolder,
    runAnalysisJobs,
    recommendProject,
    startScan,
    canStartScan,
    render,
  };
}

function renderWorkbenchSidebar() {
  const sidebar = sidebarOptions();
  if (state.layoutMode === "viewer") {
    return renderViewerLeftRail(sidebar);
  }
  const side = el("aside", "project-sidebar");
  append(side, renderSourcePanel(sidebar), renderIntelligencePanel(sidebar), renderTransferPanel(sidebar), renderViewsPanel(sidebar), renderFiltersPanel(sidebar));
  return side;
}

function sidebarOptions(): SidebarPanelOptions {
  return {
    allGroups,
    displayGroupsFor,
    chooseFolder,
    startScan,
    syncLanProjectContext,
    openSettingsPanel,
    evaluateGroupWithModel,
    currentIntelligenceSetup,
    getScanStartBlocker,
    renderProjectCreate,
    lanSyncTransferDot,
    lanSyncTransferLabel,
    render,
  };
}

function settingsDrawerOptions(): SettingsDrawerOptions {
  return {
    render,
    saveIntelligenceSettings,
    renderProviderManagement,
    renderPromptManagement,
    currentIntelligenceSetup,
    filteredGroups,
    evaluateLoadedGroupsWithModel,
  };
}

function renderProjectCreate(variant: "compact" | "hero" = "compact") {
  return renderProjectCreateForm({ createProject, chooseProjectFolderDraft }, variant);
}

function renderLightTable() {
  const shell = el("section", state.layoutMode === "viewer" ? "light-table is-viewer" : "light-table");
  const lightTable = lightTableOptions();
  shell.addEventListener("wheel", (event) => handleLightTableWheel(event, lightTable), { passive: false });
  if (!state.selectedProjectId || !state.rootPath) {
    append(shell, renderWorkbenchEmptyState(workbenchViewOptions()));
  } else if (state.layoutMode === "viewer") {
    append(shell, renderViewerMode());
  } else {
    append(shell, renderLightTableToolbar(lightTable), renderGroupBoard(lightTable));
  }
  if (state.loupe) {
    append(shell, renderLoupeOverlay(loupeOverlayOptions()));
  }
  return shell;
}

function loupeOverlayOptions(): LoupeOverlayOptions {
  return {
    loupe: state.loupe,
    groupByIdentity,
    positionLoupeOverlay,
    setPreviewBackground,
  };
}

function lightTableOptions(): LightTableOptions {
  return {
    allGroups,
    displayGroupsFor,
    filteredGroups,
    render,
    resetBoardViewport,
    clearViewerDrag: () => viewerInteraction.clearDrag(),
    renderWorkbenchEmptyState: () => renderWorkbenchEmptyState(workbenchViewOptions()),
    renderGroupCard: (group, gridColumns) => renderAssetGroupCard(group, gridColumns, assetGroupCardOptions()),
    loadMoreAssetGroups,
    warmThumbnailsForGroups,
  };
}

function assetGroupCardOptions(): AssetGroupCardOptions {
  return {
    selectedGroupId: state.selectedGroupId,
    selectGroup,
    appendPreviewImage,
    renderPreviewStatusBadge,
    renderFaceRiskOverlay,
    currentThumbnailMaxEdge,
    updateLoupeFromPointer,
    clearLoupeIfFloating,
    handleLoupeWheel,
    burstMembersOf,
    previewTooltipForGroup,
  };
}

function inspectorPanelOptions(): InspectorPanelOptions {
  return {
    busy: state.busy,
    subjectAssessments: state.subjectAssessments,
    groupDetail: state.groupDetail,
    closeInspector: () => {
      state.selectedGroupId = null;
      state.selectedGroup = null;
      state.groupDetail = [];
      state.loupe = null;
      render();
    },
    currentIntelligenceSetup,
    toggleFavorite: () => toggleFavorite(),
    toggleMarked: () => toggleMarked(),
    evaluateGroupWithModel,
    recommendBurst: () => recommendBurst(),
    deleteAssetGroup,
  };
}

function renderViewerMode() {
  const groups = filteredGroups();
  const current = viewerCurrentGroup(groups, state.selectedGroupId);
  const viewer = el("section", `viewer-mode${state.viewerInspectorOpen ? " inspector-open" : ""}${state.viewerFilmstripCollapsed ? " filmstrip-collapsed" : ""}`);
  if (!current) {
    append(viewer, renderWorkbenchEmptyState(workbenchViewOptions()));
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
  const viewerChrome = viewerChromeOptions();
  const viewerStage = viewerStageOptions(viewerChrome);
  append(
    viewer,
    renderViewerStage(current, groups, viewerStage),
    state.viewerInspectorOpen ? renderViewerInspector(current, groups, viewerChrome) : renderViewerRightRail(current, viewerChrome),
    renderViewerFilmstrip(groups, current, viewerStage),
  );
  return viewer;
}

function viewerChromeOptions(): ViewerChromeOptions {
  return {
    subjectAssessments: state.subjectAssessments,
    burstMembersOf,
    appendPreviewImage,
    currentThumbnailMaxEdge,
    previewTooltipForGroup,
    selectBurstMember: (group) => void selectGroup(group, { preserveViewerTransform: true }),
    openInspector: () => {
      state.viewerInspectorOpen = true;
      render();
    },
    closeInspector: () => {
      state.viewerInspectorOpen = false;
      render();
    },
  };
}

function viewerStageOptions(viewerChrome: ViewerChromeOptions): ViewerStageOptions {
  return {
    viewerPreviewMaxEdge: VIEWER_PREVIEW_MAX_EDGE,
    viewerTransformZoom: state.viewerTransform.zoom,
    viewerFilmstripCollapsed: state.viewerFilmstripCollapsed,
    appendPreviewImage,
    appendViewerCarryoverImage: (preview) => viewerInteraction.appendCarryoverImage(preview),
    renderPreviewStatusBadge,
    renderFaceRiskOverlay,
    handleWheel: (event, preview) => viewerInteraction.handleWheel(event, preview),
    handleDoubleClick: (event, preview) => viewerInteraction.handleDoubleClick(event, preview),
    handlePointerDown: (event, preview) => viewerInteraction.handlePointerDown(event, preview),
    handlePointerMove: (event, preview) => viewerInteraction.handlePointerMove(event, preview),
    endDrag: (preview, event) => viewerInteraction.endDrag(preview, event),
    applyTransformToNode: applyViewerTransformToNode,
    selectGroup: (group) => void selectGroup(group),
    renderActionDock: (group) => renderViewerActionDockComponent(group, viewerActionDockOptions()),
    renderBurstQueue: (group) => renderViewerBurstQueue(group, viewerChrome),
    currentThumbnailMaxEdge,
    setFilmstripCollapsed: (collapsed) => {
      state.viewerFilmstripCollapsed = collapsed;
      render();
    },
  };
}

function clearViewerCarryover(preview: HTMLElement) {
  viewerInteraction.clearCarryover(preview);
}

function applyViewerTransformToNode(preview: HTMLElement) {
  viewerInteraction.applyTransformToNode(preview);
}

function viewerActionDockOptions(): ViewerActionDockOptions {
  return {
    busy: state.busy,
    modelReady: currentIntelligenceSetup().modelReady,
    toggleFavorite,
    toggleMarked,
    runGroupAnalysis,
    evaluateGroupWithModel,
    recommendBurst,
    removeFromBurst,
    deleteAssetGroup,
  };
}

function renderFaceRiskOverlay(group: ReceivedAssetGroup) {
  return renderFaceRiskOverlayForGroup(group, state.subjectAssessments, {
    onViewerPreview: applyViewerTransformToNode,
  });
}

function syncAllFaceRiskLayers() {
  syncAllFaceRiskLayersWith({ onViewerPreview: applyViewerTransformToNode });
}

function syncFaceRiskLayer(container: HTMLElement) {
  syncFaceRiskLayerWith(container, { onViewerPreview: applyViewerTransformToNode });
}

function updateLoupeFromPointer(event: PointerEvent, group: ReceivedAssetGroup, maxEdge = currentThumbnailMaxEdge(), original = false) {
  loupeInteraction.updateFromPointer(event, group, maxEdge, original);
}

function handleLoupeWheel(event: WheelEvent, group: ReceivedAssetGroup, maxEdge = currentThumbnailMaxEdge(), original = false) {
  loupeInteraction.handleWheel(event, group, maxEdge, original);
}

function clearLoupeIfFloating() {
  loupeInteraction.clearIfFloating();
}

function positionLoupeOverlay(overlay: HTMLElement, loupe: LoupeState) {
  loupeInteraction.positionOverlay(overlay, loupe);
}

function ensureOriginalPreviewForGroup(group: ReceivedAssetGroup, priority: ThumbnailPriority) {
  const asset = previewAssetForGroup(group);
  const localPath = asset ? localPreviewablePath(asset) : null;
  if (!asset || !localPath || !shouldRequestOriginalPreviewAsset(asset)) {
    return null;
  }
  return originalPreviewUrlForPath(localPath, priority);
}
