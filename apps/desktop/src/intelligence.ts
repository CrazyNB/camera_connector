export type ModelProviderSettings = {
  settings_id: string;
  provider_kind: string;
  provider_label: string;
  base_url: string;
  default_model: string;
  default_max_image_side: number;
  default_send_mode: string;
  default_batch_size: number;
  configured: boolean;
  api_key_configured: boolean;
  key_alias?: string | null;
  updated_at_ms: number;
};

export type PromptPack = {
  prompt_pack_id: string;
  distribution_folder: string;
  name: string;
  version: string;
  author: string;
  style_tags: string[];
  scene_profile: string;
  schema: string;
  capabilities: string[];
  built_in: boolean;
  enabled: boolean;
  shared_preference?: string | null;
  prompt_hash: string;
  updated_at_ms: number;
};

export type TechnicalAssessmentPolicy = {
  blur_severe_edge_threshold: number;
  blur_severe_frequency_threshold: number;
  blur_high_edge_threshold: number;
  blur_high_frequency_threshold: number;
  highlight_clip_threshold: number;
  shadow_clip_threshold: number;
  clipping_high_ratio: number;
  clipping_high_connected_ratio: number;
  clipping_severe_ratio: number;
  clipping_severe_connected_ratio: number;
  color_cast_high_threshold: number;
  color_cast_severe_threshold: number;
  face_eye_open_warn_threshold: number;
  face_exposure_warn_ratio: number;
  face_color_cast_warn_threshold: number;
};

export type CvThresholdMode = "loose" | "standard" | "strict" | "custom";

export type CvThresholdControlKey =
  | "blur"
  | "clipping"
  | "shadow_clip"
  | "highlight_clip"
  | "color_cast"
  | "face_eyes"
  | "face_exposure"
  | "face_color_cast";

export type CvThresholdControlSpec = {
  key: CvThresholdControlKey;
  title: string;
  sliderValue: number;
  displayPercent: number;
  displayLabel: string;
  description: string;
};

export type ProjectEvaluationSettings = {
  project_id: string;
  auto_evaluate_on_upload: boolean;
  auto_burst_recommendation_enabled: boolean;
  project_recommendation_mode: "manual";
  prompt_pack_id?: string | null;
  model_provider_settings_id?: string | null;
  scene_profile: string;
  cv_policy: string;
  cv_policy_overrides?: TechnicalAssessmentPolicy | null;
  allow_risky_model_selects: boolean;
  max_image_side?: number | null;
  batch_size?: number | null;
  updated_at_ms: number;
};

export type SaveModelProviderSettingsRequest = Omit<
  ModelProviderSettings,
  "api_key_configured" | "updated_at_ms"
> & {
  api_key?: string | null;
};

export type CreatePromptPackRequest = {
  name: string;
  style_tags: string[];
  scene_profile: string;
  distribution_folder: string;
  shared_preference: string;
};

export type ForkPromptPackRequest = {
  source_prompt_pack_id: string;
  name: string;
  distribution_folder: string;
};

export type SavePromptPackRequest = {
  prompt_pack_id: string;
  name: string;
  style_tags: string[];
  scene_profile: string;
  shared_preference: string;
};

export type PromptDraftMode = "create" | "edit" | "fork";

export type PromptDraft = {
  mode: PromptDraftMode;
  prompt_pack_id: string | null;
  source_prompt_pack_id: string | null;
  name: string;
  distribution_folder: string;
  style_tags_text: string;
  scene_profile: string;
  shared_preference: string;
  built_in: boolean;
};

export type IntelligenceSetupState = {
  selectedProvider: ModelProviderSettings | null;
  selectedPrompt: PromptPack | null;
  providerReady: boolean;
  promptReady: boolean;
  modelReady: boolean;
  autoEvaluate: boolean;
  autoBurstRecommendation: boolean;
  allowsRiskySelects: boolean;
};

const BLUR_HIGH_MIN = 0.06;
const BLUR_HIGH_MAX = 0.22;
const CLIPPING_HIGH_MIN = 0.04;
const CLIPPING_HIGH_MAX = 0.3;
const CLIPPING_HIGH_CONNECTED_MIN = 0.04;
const CLIPPING_HIGH_CONNECTED_MAX = 0.3;
const CLIPPING_SEVERE_MIN = 0.35;
const CLIPPING_SEVERE_MAX = 0.75;
const SHADOW_CLIP_THRESHOLD_MIN = 0;
const SHADOW_CLIP_THRESHOLD_MAX = 15;
const HIGHLIGHT_CLIP_THRESHOLD_MIN = 235;
const HIGHLIGHT_CLIP_THRESHOLD_MAX = 255;
const COLOR_CAST_HIGH_MIN = 0.28;
const COLOR_CAST_HIGH_MAX = 0.65;
const COLOR_CAST_SEVERE_MIN = 0.5;
const COLOR_CAST_SEVERE_MAX = 0.9;
const FACE_EYE_OPEN_WARN_MIN = 0.2;
const FACE_EYE_OPEN_WARN_MAX = 0.55;
const FACE_EXPOSURE_WARN_MIN = 0.12;
const FACE_EXPOSURE_WARN_MAX = 0.4;
const FACE_COLOR_CAST_WARN_MIN = 0.28;
const FACE_COLOR_CAST_WARN_MAX = 0.65;

export function selectedProjectProvider(
  providers: ModelProviderSettings[],
  settings: ProjectEvaluationSettings | null,
): ModelProviderSettings | null {
  if (!settings?.model_provider_settings_id) return null;
  return providers.find((provider) => provider.settings_id === settings.model_provider_settings_id) ?? null;
}

export function selectedProjectPrompt(
  prompts: PromptPack[],
  settings: ProjectEvaluationSettings | null,
): PromptPack | null {
  if (!settings?.prompt_pack_id) return null;
  return prompts.find((prompt) => prompt.prompt_pack_id === settings.prompt_pack_id) ?? null;
}

export function intelligenceSetupState(
  providers: ModelProviderSettings[],
  prompts: PromptPack[],
  settings: ProjectEvaluationSettings | null,
): IntelligenceSetupState {
  const selectedProvider = selectedProjectProvider(providers, settings);
  const selectedPrompt = selectedProjectPrompt(prompts, settings);
  const providerReady = Boolean(selectedProvider?.configured && selectedProvider.api_key_configured);
  const promptReady = Boolean(selectedPrompt?.enabled);
  return {
    selectedProvider,
    selectedPrompt,
    providerReady,
    promptReady,
    modelReady: providerReady && promptReady,
    autoEvaluate: Boolean(settings?.auto_evaluate_on_upload),
    autoBurstRecommendation: Boolean(settings?.auto_burst_recommendation_enabled),
    allowsRiskySelects: Boolean(settings?.allow_risky_model_selects),
  };
}

export function intelligenceStatusLabel(setup: IntelligenceSetupState): string {
  if (setup.modelReady && setup.autoEvaluate) return "Auto";
  if (setup.modelReady) return "Manual";
  if (!setup.providerReady) return "Provider";
  if (!setup.promptReady) return "Prompt";
  return "Setup";
}

export function technicalPolicyForCvPolicy(value: string): TechnicalAssessmentPolicy {
  switch (value.trim().toLowerCase()) {
    case "loose":
      return {
        blur_severe_edge_threshold: 0.025,
        blur_severe_frequency_threshold: 0.025,
        blur_high_edge_threshold: 0.09,
        blur_high_frequency_threshold: 0.09,
        highlight_clip_threshold: 250,
        shadow_clip_threshold: 2,
        clipping_high_ratio: 0.18,
        clipping_high_connected_ratio: 0.25,
        clipping_severe_ratio: 0.65,
        clipping_severe_connected_ratio: 0.65,
        color_cast_high_threshold: 0.55,
        color_cast_severe_threshold: 0.85,
        face_eye_open_warn_threshold: 0.25,
        face_exposure_warn_ratio: 0.35,
        face_color_cast_warn_threshold: 0.55,
      };
    case "strict":
      return {
        blur_severe_edge_threshold: 0.06,
        blur_severe_frequency_threshold: 0.06,
        blur_high_edge_threshold: 0.16,
        blur_high_frequency_threshold: 0.16,
        highlight_clip_threshold: 242,
        shadow_clip_threshold: 8,
        clipping_high_ratio: 0.08,
        clipping_high_connected_ratio: 0.12,
        clipping_severe_ratio: 0.4,
        clipping_severe_connected_ratio: 0.4,
        color_cast_high_threshold: 0.32,
        color_cast_severe_threshold: 0.55,
        face_eye_open_warn_threshold: 0.45,
        face_exposure_warn_ratio: 0.16,
        face_color_cast_warn_threshold: 0.32,
      };
    default:
      return {
        blur_severe_edge_threshold: 0.04,
        blur_severe_frequency_threshold: 0.04,
        blur_high_edge_threshold: 0.12,
        blur_high_frequency_threshold: 0.12,
        highlight_clip_threshold: 245,
        shadow_clip_threshold: 5,
        clipping_high_ratio: 0.12,
        clipping_high_connected_ratio: 0.18,
        clipping_severe_ratio: 0.5,
        clipping_severe_connected_ratio: 0.5,
        color_cast_high_threshold: 0.42,
        color_cast_severe_threshold: 0.7,
        face_eye_open_warn_threshold: 0.35,
        face_exposure_warn_ratio: 0.25,
        face_color_cast_warn_threshold: 0.42,
      };
  }
}

export function selectedCvThresholdMode(settings: ProjectEvaluationSettings): CvThresholdMode {
  if (settings.cv_policy_overrides) return "custom";
  return normalizeCvThresholdMode(settings.cv_policy);
}

export function settingsForCvThresholdMode(
  settings: ProjectEvaluationSettings,
  mode: CvThresholdMode,
): ProjectEvaluationSettings {
  if (mode === "custom") {
    return {
      ...settings,
      cv_policy_overrides: settings.cv_policy_overrides ?? technicalPolicyForCvPolicy(settings.cv_policy),
    };
  }
  return {
    ...settings,
    cv_policy: mode,
    cv_policy_overrides: null,
  };
}

export function cvThresholdControlSpecs(
  policy: TechnicalAssessmentPolicy,
  sceneProfile = "general",
): CvThresholdControlSpec[] {
  const blur = blurSensitivity(policy);
  const clipping = clippingSensitivity(policy);
  const shadow = shadowClipThresholdValue(policy);
  const highlight = highlightClipThresholdValue(policy);
  const color = colorCastSensitivity(policy);
  const controls: CvThresholdControlSpec[] = [
    {
      key: "blur",
      title: "失焦灵敏度",
      sliderValue: blur,
      displayPercent: percentLabel(blur),
      displayLabel: `${percentLabel(blur)}%`,
      description: `边缘和高频细节低于 ${formatRatioPercent(policy.blur_high_edge_threshold)} 标记失焦，低于 ${formatRatioPercent(policy.blur_severe_edge_threshold)} 视为严重。`,
    },
    {
      key: "clipping",
      title: "死黑/死白灵敏度",
      sliderValue: clipping,
      displayPercent: percentLabel(clipping),
      displayLabel: `${percentLabel(clipping)}%`,
      description: `近黑 <=${policy.shadow_clip_threshold} / 近白 >=${policy.highlight_clip_threshold}，占比超过 ${formatRatioPercent(policy.clipping_high_ratio)} 或连片超过 ${formatRatioPercent(policy.clipping_high_connected_ratio)} 标记风险。`,
    },
    {
      key: "shadow_clip",
      title: "近黑边界",
      sliderValue: shadow,
      displayPercent: policy.shadow_clip_threshold,
      displayLabel: `<=${policy.shadow_clip_threshold}`,
      description: `亮度小于等于 ${policy.shadow_clip_threshold} 的像素计入暗部死黑。数值越低，误报越少。`,
    },
    {
      key: "highlight_clip",
      title: "近白边界",
      sliderValue: highlight,
      displayPercent: policy.highlight_clip_threshold,
      displayLabel: `>=${policy.highlight_clip_threshold}`,
      description: `亮度大于等于 ${policy.highlight_clip_threshold} 的像素计入高光溢出。数值越高，判断越保守。`,
    },
    {
      key: "color_cast",
      title: "偏色灵敏度",
      sliderValue: color,
      displayPercent: percentLabel(color),
      displayLabel: `${percentLabel(color)}%`,
      description: `RGB 通道相对亮度差异超过 ${formatDecimal(policy.color_cast_high_threshold, 2)} 标记偏色，超过 ${formatDecimal(policy.color_cast_severe_threshold, 2)} 视为严重。`,
    },
  ];
  if (sceneProfile.trim().toLowerCase() === "portrait") {
    const eyes = faceEyesSensitivity(policy);
    const exposure = faceExposureSensitivity(policy);
    const faceColor = faceColorCastSensitivity(policy);
    controls.push(
      {
        key: "face_eyes",
        title: "闭眼灵敏度",
        sliderValue: eyes,
        displayPercent: percentLabel(eyes),
        displayLabel: `${percentLabel(eyes)}%`,
        description: `检测到人脸时，任一眼睁开概率低于 ${formatDecimal(policy.face_eye_open_warn_threshold, 2)} 标记闭眼风险。`,
      },
      {
        key: "face_exposure",
        title: "面部死黑/死白灵敏度",
        sliderValue: exposure,
        displayPercent: percentLabel(exposure),
        displayLabel: `${percentLabel(exposure)}%`,
        description: `人脸区域近黑/近白像素占比超过 ${formatRatioPercent(policy.face_exposure_warn_ratio)} 标记面部曝光风险。`,
      },
      {
        key: "face_color_cast",
        title: "面部偏色灵敏度",
        sliderValue: faceColor,
        displayPercent: percentLabel(faceColor),
        displayLabel: `${percentLabel(faceColor)}%`,
        description: `人脸区域 RGB 相对亮度差异超过 ${formatDecimal(policy.face_color_cast_warn_threshold, 2)} 标记面部偏色。`,
      },
    );
  }
  return controls;
}

export function updateCvThresholdControl(
  policy: TechnicalAssessmentPolicy,
  key: CvThresholdControlKey,
  value: number,
): TechnicalAssessmentPolicy {
  const sliderValue = clamp01(value);
  switch (key) {
    case "blur": {
      const next = denormalize(sliderValue, BLUR_HIGH_MIN, BLUR_HIGH_MAX);
      return {
        ...policy,
        blur_high_edge_threshold: next,
        blur_high_frequency_threshold: next,
        blur_severe_edge_threshold: Math.min(policy.blur_severe_edge_threshold, next),
        blur_severe_frequency_threshold: Math.min(policy.blur_severe_frequency_threshold, next),
      };
    }
    case "clipping":
      return {
        ...policy,
        clipping_high_ratio: inverseDenormalize(sliderValue, CLIPPING_HIGH_MIN, CLIPPING_HIGH_MAX),
        clipping_high_connected_ratio: inverseDenormalize(
          sliderValue,
          CLIPPING_HIGH_CONNECTED_MIN,
          CLIPPING_HIGH_CONNECTED_MAX,
        ),
        clipping_severe_ratio: inverseDenormalize(sliderValue, CLIPPING_SEVERE_MIN, CLIPPING_SEVERE_MAX),
        clipping_severe_connected_ratio: inverseDenormalize(
          sliderValue,
          CLIPPING_SEVERE_MIN,
          CLIPPING_SEVERE_MAX,
        ),
      };
    case "shadow_clip":
      return {
        ...policy,
        shadow_clip_threshold: Math.round(
          denormalize(sliderValue, SHADOW_CLIP_THRESHOLD_MIN, SHADOW_CLIP_THRESHOLD_MAX),
        ),
      };
    case "highlight_clip":
      return {
        ...policy,
        highlight_clip_threshold: Math.round(
          denormalize(sliderValue, HIGHLIGHT_CLIP_THRESHOLD_MIN, HIGHLIGHT_CLIP_THRESHOLD_MAX),
        ),
      };
    case "color_cast":
      return {
        ...policy,
        color_cast_high_threshold: inverseDenormalize(sliderValue, COLOR_CAST_HIGH_MIN, COLOR_CAST_HIGH_MAX),
        color_cast_severe_threshold: inverseDenormalize(
          sliderValue,
          COLOR_CAST_SEVERE_MIN,
          COLOR_CAST_SEVERE_MAX,
        ),
      };
    case "face_eyes":
      return {
        ...policy,
        face_eye_open_warn_threshold: denormalize(sliderValue, FACE_EYE_OPEN_WARN_MIN, FACE_EYE_OPEN_WARN_MAX),
      };
    case "face_exposure":
      return {
        ...policy,
        face_exposure_warn_ratio: inverseDenormalize(sliderValue, FACE_EXPOSURE_WARN_MIN, FACE_EXPOSURE_WARN_MAX),
      };
    case "face_color_cast":
      return {
        ...policy,
        face_color_cast_warn_threshold: inverseDenormalize(
          sliderValue,
          FACE_COLOR_CAST_WARN_MIN,
          FACE_COLOR_CAST_WARN_MAX,
        ),
      };
  }
}

export function providerDraftFromSettings(
  settings: ModelProviderSettings | null,
): SaveModelProviderSettingsRequest {
  if (!settings) {
    return {
      settings_id: "global",
      provider_kind: "openai",
      provider_label: "OpenAI",
      base_url: "",
      default_model: "",
      default_max_image_side: 1536,
      default_send_mode: "preview_only",
      default_batch_size: 1,
      configured: false,
      key_alias: null,
      api_key: null,
    };
  }
  return {
    settings_id: settings.settings_id,
    provider_kind: settings.provider_kind,
    provider_label: settings.provider_label,
    base_url: settings.base_url,
    default_model: settings.default_model,
    default_max_image_side: settings.default_max_image_side,
    default_send_mode: settings.default_send_mode,
    default_batch_size: settings.default_batch_size,
    configured: settings.configured,
    key_alias: settings.key_alias ?? null,
    api_key: null,
  };
}

export function providerDraftIsSaveable(draft: SaveModelProviderSettingsRequest): boolean {
  if (!draft.settings_id.trim() || !draft.provider_label.trim()) return false;
  if (draft.provider_kind !== "none" && draft.configured && !draft.default_model.trim()) return false;
  if (draft.provider_kind === "custom" && draft.configured && !draft.base_url.trim()) return false;
  return draft.default_max_image_side > 0 && draft.default_batch_size > 0;
}

export function newPromptDraft(): PromptDraft {
  return {
    mode: "create",
    prompt_pack_id: null,
    source_prompt_pack_id: null,
    name: "",
    distribution_folder: "user",
    style_tags_text: "",
    scene_profile: "general",
    shared_preference: "",
    built_in: false,
  };
}

export function promptDraftFromPack(pack: PromptPack): PromptDraft {
  const sharedPreference = pack.shared_preference ?? "";
  if (pack.built_in) {
    return {
      mode: "fork",
      prompt_pack_id: null,
      source_prompt_pack_id: pack.prompt_pack_id,
      name: `Custom ${pack.name}`,
      distribution_folder: "user",
      style_tags_text: pack.style_tags.join(" "),
      scene_profile: pack.scene_profile,
      shared_preference: sharedPreference,
      built_in: true,
    };
  }
  return {
    mode: "edit",
    prompt_pack_id: pack.prompt_pack_id,
    source_prompt_pack_id: null,
    name: pack.name,
    distribution_folder: pack.distribution_folder,
    style_tags_text: pack.style_tags.join(" "),
    scene_profile: pack.scene_profile,
    shared_preference: sharedPreference,
    built_in: false,
  };
}

export function promptStyleTagsFromText(value: string): string[] {
  return value
    .split(/[,\s]+/g)
    .map((tag) => tag.trim())
    .filter(Boolean);
}

export function promptDraftIsSaveable(draft: PromptDraft): boolean {
  if (!draft.name.trim() || !draft.distribution_folder.trim() || !draft.shared_preference.trim()) return false;
  if (draft.mode === "edit") return Boolean(draft.prompt_pack_id);
  if (draft.mode === "fork") return Boolean(draft.source_prompt_pack_id);
  return true;
}

function normalizeCvThresholdMode(value: string): Exclude<CvThresholdMode, "custom"> {
  const token = value.trim().toLowerCase();
  if (token === "loose" || token === "strict") return token;
  return "standard";
}

function blurSensitivity(policy: TechnicalAssessmentPolicy): number {
  return normalize(policy.blur_high_edge_threshold, BLUR_HIGH_MIN, BLUR_HIGH_MAX);
}

function clippingSensitivity(policy: TechnicalAssessmentPolicy): number {
  return average([
    inverseNormalize(policy.clipping_high_ratio, CLIPPING_HIGH_MIN, CLIPPING_HIGH_MAX),
    inverseNormalize(policy.clipping_high_connected_ratio, CLIPPING_HIGH_CONNECTED_MIN, CLIPPING_HIGH_CONNECTED_MAX),
    inverseNormalize(policy.clipping_severe_ratio, CLIPPING_SEVERE_MIN, CLIPPING_SEVERE_MAX),
  ]);
}

function shadowClipThresholdValue(policy: TechnicalAssessmentPolicy): number {
  return normalize(policy.shadow_clip_threshold, SHADOW_CLIP_THRESHOLD_MIN, SHADOW_CLIP_THRESHOLD_MAX);
}

function highlightClipThresholdValue(policy: TechnicalAssessmentPolicy): number {
  return normalize(policy.highlight_clip_threshold, HIGHLIGHT_CLIP_THRESHOLD_MIN, HIGHLIGHT_CLIP_THRESHOLD_MAX);
}

function colorCastSensitivity(policy: TechnicalAssessmentPolicy): number {
  return average([
    inverseNormalize(policy.color_cast_high_threshold, COLOR_CAST_HIGH_MIN, COLOR_CAST_HIGH_MAX),
    inverseNormalize(policy.color_cast_severe_threshold, COLOR_CAST_SEVERE_MIN, COLOR_CAST_SEVERE_MAX),
  ]);
}

function faceEyesSensitivity(policy: TechnicalAssessmentPolicy): number {
  return normalize(policy.face_eye_open_warn_threshold, FACE_EYE_OPEN_WARN_MIN, FACE_EYE_OPEN_WARN_MAX);
}

function faceExposureSensitivity(policy: TechnicalAssessmentPolicy): number {
  return inverseNormalize(policy.face_exposure_warn_ratio, FACE_EXPOSURE_WARN_MIN, FACE_EXPOSURE_WARN_MAX);
}

function faceColorCastSensitivity(policy: TechnicalAssessmentPolicy): number {
  return inverseNormalize(
    policy.face_color_cast_warn_threshold,
    FACE_COLOR_CAST_WARN_MIN,
    FACE_COLOR_CAST_WARN_MAX,
  );
}

function percentLabel(value: number): number {
  return Math.round(clamp01(value) * 100);
}

function formatRatioPercent(value: number): string {
  return `${percentLabel(value)}%`;
}

function formatDecimal(value: number, digits: number): string {
  return value.toFixed(digits);
}

function normalize(value: number, min: number, max: number): number {
  return clamp01((value - min) / (max - min));
}

function inverseNormalize(value: number, min: number, max: number): number {
  return clamp01((max - value) / (max - min));
}

function denormalize(value: number, min: number, max: number): number {
  return min + (max - min) * clamp01(value);
}

function inverseDenormalize(value: number, min: number, max: number): number {
  return max - (max - min) * clamp01(value);
}

function average(values: number[]): number {
  if (!values.length) return 0;
  return clamp01(values.reduce((sum, value) => sum + value, 0) / values.length);
}

function clamp01(value: number): number {
  return Math.min(1, Math.max(0, value));
}
