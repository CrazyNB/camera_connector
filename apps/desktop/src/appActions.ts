import { viewerActionScope } from "./lightTableLayout";
import { api } from "./desktopApi";
import {
  promptDraftFromPack,
  promptDraftIsSaveable,
  promptStyleTagsFromText,
  providerDraftFromSettings,
  providerDraftIsSaveable,
  type IntelligenceSetupState,
  type ProjectEvaluationSettings,
  type PromptPack,
} from "./intelligence";
import type {
  AppState,
  DesktopCvAssessmentResponse,
  ReceivedAssetGroup,
} from "./appTypes";

type WithBusy = <T>(label: string, task: () => Promise<T>) => Promise<T | null>;

export function createAppActions(options: {
  state: AppState;
  render: () => void;
  refreshCurrentProject: (showLoadedStatus?: boolean) => Promise<void>;
  refreshPromptPackLists: () => Promise<void>;
  filteredGroups: () => ReceivedAssetGroup[];
  allGroups: () => ReceivedAssetGroup[];
  setStatus: (message: string, error?: string | null) => void;
  withBusy: WithBusy;
  currentIntelligenceSetup: () => IntelligenceSetupState;
}) {
  const {
    state,
    render,
    refreshCurrentProject,
    refreshPromptPackLists,
    filteredGroups,
    allGroups,
    setStatus,
    withBusy,
    currentIntelligenceSetup,
  } = options;

  async function runAnalysisJobs() {
    const projectId = state.selectedProjectId;
    if (!projectId) return;
    state.cvProgress = {
      project_id: projectId,
      scope: viewerActionScope("global-quality"),
      total_count: Math.max(1, state.assetPage?.summary.group_count ?? allGroups().length),
      assessed_count: 0,
      failed_count: 0,
      skipped_count: 0,
      subject_count: 0,
      current_group_id: null,
    };
    render();
    const summary = await withBusy("杩愯鍏ㄥ眬璐ㄦ", () => api.runDesktopCvAssessment(projectId, 2000));
    if (summary) {
      state.status = qualityAssessmentStatus("鍏ㄥ眬璐ㄦ", summary);
      state.cvProgress = null;
      await refreshCurrentProject(false);
    } else {
      state.cvProgress = null;
      render();
    }
  }

  async function runGroupAnalysis(targetGroup = state.selectedGroup) {
    const projectId = state.selectedProjectId;
    const groupId = targetGroup?.group_id;
    if (!projectId || !groupId) {
      return;
    }
    state.cvProgress = {
      project_id: projectId,
      scope: viewerActionScope("quality"),
      total_count: 1,
      assessed_count: 0,
      failed_count: 0,
      skipped_count: 0,
      subject_count: 0,
      current_group_id: groupId,
    };
    render();
    const summary = await withBusy("璐ㄩ噺妫€鏌?", () => api.runDesktopCvAssessment(projectId, 1, [groupId]));
    if (summary) {
      state.status = qualityAssessmentStatus(targetGroup.group_key, summary);
      state.cvProgress = null;
      await refreshCurrentProject(false);
    } else {
      state.cvProgress = null;
      render();
    }
  }

  function qualityAssessmentStatus(label: string, summary: DesktopCvAssessmentResponse) {
    const done = summary.assessed_count;
    const failed = summary.failed_count;
    const skipped = summary.skipped_count;
    const subject = summary.subject_count ? `锛屼汉鑴?${summary.subject_count}` : "";
    return `${label}锛氬畬鎴?${done}锛屽け璐?${failed}锛岃烦杩?${skipped}${subject}`;
  }

  async function recommendBurst(targetGroup = state.selectedGroup) {
    const burstId = targetGroup?.burst?.burst_group_id;
    if (!burstId) {
      setStatus("鎺ㄨ崘杩炴媿", "褰撳墠鐓х墖缁勪笉灞炰簬杩炴媿銆?");
      return;
    }
    const recommendation = await withBusy("鎺ㄨ崘杩炴媿", () => api.recommendBurstGroup(burstId));
    if (recommendation) {
      state.status = `杩炴媿鎺ㄨ崘锛?{readable(recommendation.status)}`;
      await refreshCurrentProject(false);
    }
  }

  async function recommendProject() {
    const projectId = state.selectedProjectId;
    if (!projectId) {
      return;
    }
    const recommendation = await withBusy("鐢熸垚鍏ㄥ眬鎺ㄨ崘", () => api.generateProjectRecommendation(projectId));
    if (recommendation) {
      state.status = `鍏ㄥ眬鎺ㄨ崘锛?{readable(recommendation.status)}`;
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
    const saved = await withBusy("淇濆瓨 AI 杈呭姪璁剧疆", () => api.saveProjectEvaluationSettings(settings));
    if (saved) {
      state.intelligenceSettings = saved;
      state.status = "AI 杈呭姪璁剧疆宸蹭繚瀛?";
      render();
    }
  }

  async function evaluateGroupWithModel(targetGroup = state.selectedGroup) {
    const projectId = state.selectedProjectId;
    const groupId = targetGroup?.group_id;
    if (!projectId || !groupId) return;
    const result = await withBusy("AI 璇勪环", async () => {
      const enqueued = await api.enqueueModelEvaluation(projectId, [groupId]);
      await api.drainAnalysisJobs(12);
      return enqueued;
    });
    if (result) {
      state.status = result.enqueued_count ? `宸插姞鍏?AI 璇勪环锛?{result.enqueued_count} 涓収鐗囩粍` : "AI 璇勪环宸插姞鍏ラ槦鍒?";
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
    const result = await withBusy("AI 璇勪环", async () => {
      const enqueued = await api.enqueueModelEvaluation(projectId, groupIds);
      await api.drainAnalysisJobs(50);
      return enqueued;
    });
    if (result) {
      state.status = `宸插姞鍏?AI 璇勪环锛?{result.enqueued_count}/${groupIds.length} 涓収鐗囩粍`;
      await refreshCurrentProject(false);
    }
  }

  async function saveProviderDraft() {
    const draft = state.providerDraft;
    if (!draft || !providerDraftIsSaveable(draft)) return;
    const saved = await withBusy("淇濆瓨 AI 鏈嶅姟", () => api.saveModelProviderSettings(draft));
    if (!saved) return;
    state.providerDraft = providerDraftFromSettings(saved);
    state.intelligenceProviders = await api.getModelProviderSettingsList();
    if (state.settingsPanel === "project" && state.intelligenceSettings?.model_provider_settings_id !== saved.settings_id) {
      await saveIntelligenceSettings({ model_provider_settings_id: saved.settings_id });
    } else {
      state.status = "AI 鏈嶅姟宸蹭繚瀛?";
      render();
    }
  }

  async function deleteProvider(settingsId: string) {
    const deleted = await withBusy("鍒犻櫎 AI 鏈嶅姟", () => api.deleteModelProviderSettings(settingsId));
    if (!deleted) return;
    state.providerDraft = null;
    state.intelligenceProviders = await api.getModelProviderSettingsList();
    if (state.intelligenceSettings?.model_provider_settings_id === settingsId) {
      await saveIntelligenceSettings({ model_provider_settings_id: null });
    } else {
      state.status = "AI 鏈嶅姟宸插垹闄?";
      render();
    }
  }

  async function savePromptDraft() {
    const draft = state.promptDraft;
    if (!draft || !promptDraftIsSaveable(draft)) return;
    const styleTags = promptStyleTagsFromText(draft.style_tags_text);
    let saved: PromptPack | null = null;
    if (draft.mode === "create") {
      saved = await withBusy("鍒涘缓閫夌墖瑙勫垯鍖?", () =>
        api.createGlobalPromptPack({
          name: draft.name,
          distribution_folder: draft.distribution_folder,
          style_tags: styleTags,
          scene_profile: draft.scene_profile,
          shared_preference: draft.shared_preference,
        }),
      );
    } else if (draft.mode === "fork" && draft.source_prompt_pack_id) {
      const forked = await withBusy("澶嶅埗閫夌墖瑙勫垯鍖?", () =>
        api.forkGlobalPromptPack({
          source_prompt_pack_id: draft.source_prompt_pack_id as string,
          name: draft.name,
          distribution_folder: draft.distribution_folder,
        }),
      );
      if (forked) {
        saved = await withBusy("淇濆瓨閫夌墖瑙勫垯鍖?", () =>
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
      saved = await withBusy("淇濆瓨閫夌墖瑙勫垯鍖?", () =>
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
      state.status = "閫夌墖瑙勫垯鍖呭凡淇濆瓨";
      render();
    }
  }

  async function deletePromptPack(promptPackId: string) {
    const deleted = await withBusy("绉婚櫎閫夌墖瑙勫垯鍖?", () => api.deleteGlobalPromptPack(promptPackId));
    if (!deleted) return;
    state.promptDraft = null;
    await refreshPromptPackLists();
    if (state.intelligenceSettings?.prompt_pack_id === promptPackId) {
      await saveIntelligenceSettings({ prompt_pack_id: null });
    } else {
      state.status = "閫夌墖瑙勫垯鍖呭凡绉婚櫎";
      render();
    }
  }

  async function removeFromBurst(group: ReceivedAssetGroup) {
    const burstId = group.burst?.burst_group_id;
    if (!burstId || !group.group_id) {
      setStatus("绉诲嚭杩炴媿", "褰撳墠鐓х墖缁勪笉灞炰簬杩炴媿銆?");
      return;
    }
    await withBusy("绉诲嚭杩炴媿", () => api.splitBurstMember(burstId, group.group_id as string));
    state.selectedGroupId = group.group_id;
    await refreshCurrentProject(false);
  }

  return {
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
  };
}
