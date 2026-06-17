import assert from "node:assert/strict";
import test from "node:test";
import {
  cvThresholdControlSpecs,
  newPromptDraft,
  providerDraftFromSettings,
  promptDraftFromPack,
  promptStyleTagsFromText,
  intelligenceSetupState,
  intelligenceStatusLabel,
  selectedCvThresholdMode,
  settingsForCvThresholdMode,
  technicalPolicyForCvPolicy,
  updateCvThresholdControl,
  type ModelProviderSettings,
  type ProjectEvaluationSettings,
  type PromptPack,
} from "../src/intelligence.js";

const provider = (overrides: Partial<ModelProviderSettings> = {}): ModelProviderSettings => ({
  settings_id: "provider-1",
  provider_kind: "openai",
  provider_label: "OpenAI",
  base_url: "",
  default_model: "gpt-5-mini",
  default_max_image_side: 1536,
  default_send_mode: "preview_only",
  default_batch_size: 1,
  configured: true,
  api_key_configured: true,
  key_alias: null,
  updated_at_ms: 0,
  ...overrides,
});

const prompt = (overrides: Partial<PromptPack> = {}): PromptPack => ({
  prompt_pack_id: "general-default",
  distribution_folder: "builtin",
  name: "General",
  version: "general-default-v1",
  author: "system",
  style_tags: ["general"],
  scene_profile: "general",
  schema: "model-evaluation-v1",
  capabilities: ["asset_evaluation"],
  built_in: true,
  enabled: true,
  shared_preference: "Prefer clean keepers.",
  prompt_hash: "hash",
  updated_at_ms: 0,
  ...overrides,
});

const settings = (overrides: Partial<ProjectEvaluationSettings> = {}): ProjectEvaluationSettings => ({
  project_id: "project-1",
  auto_evaluate_on_upload: true,
  auto_burst_recommendation_enabled: true,
  project_recommendation_mode: "manual",
  prompt_pack_id: "general-default",
  model_provider_settings_id: "provider-1",
  scene_profile: "general",
  cv_policy: "standard",
  cv_policy_overrides: null,
  allow_risky_model_selects: false,
  max_image_side: null,
  batch_size: null,
  updated_at_ms: 0,
  ...overrides,
});

test("intelligenceSetupState requires a configured provider with an API key", () => {
  const setup = intelligenceSetupState([provider({ api_key_configured: false })], [prompt()], settings());

  assert.equal(setup.providerReady, false);
  assert.equal(setup.promptReady, true);
  assert.equal(setup.modelReady, false);
  assert.equal(intelligenceStatusLabel(setup), "Provider");
});

test("intelligenceSetupState requires an enabled selected prompt", () => {
  const setup = intelligenceSetupState([provider()], [prompt({ enabled: false })], settings());

  assert.equal(setup.providerReady, true);
  assert.equal(setup.promptReady, false);
  assert.equal(setup.modelReady, false);
  assert.equal(intelligenceStatusLabel(setup), "Prompt");
});

test("intelligenceSetupState reports automatic model readiness", () => {
  const setup = intelligenceSetupState([provider()], [prompt()], settings());

  assert.equal(setup.providerReady, true);
  assert.equal(setup.promptReady, true);
  assert.equal(setup.modelReady, true);
  assert.equal(setup.autoEvaluate, true);
  assert.equal(setup.autoBurstRecommendation, true);
  assert.equal(setup.allowsRiskySelects, false);
  assert.equal(intelligenceStatusLabel(setup), "Auto");
});

test("providerDraftFromSettings preserves provider metadata without echoing an API key", () => {
  assert.deepEqual(providerDraftFromSettings(provider({ key_alias: "OPENAI_API_KEY" })), {
    settings_id: "provider-1",
    provider_kind: "openai",
    provider_label: "OpenAI",
    base_url: "",
    default_model: "gpt-5-mini",
    default_max_image_side: 1536,
    default_send_mode: "preview_only",
    default_batch_size: 1,
    configured: true,
    key_alias: "OPENAI_API_KEY",
    api_key: null,
  });
});

test("promptDraftFromPack edits user prompts and forks built-in prompts", () => {
  assert.deepEqual(promptDraftFromPack(prompt({ built_in: false, prompt_pack_id: "user-pack" })), {
    mode: "edit",
    prompt_pack_id: "user-pack",
    source_prompt_pack_id: null,
    name: "General",
    distribution_folder: "builtin",
    style_tags_text: "general",
    scene_profile: "general",
    shared_preference: "Prefer clean keepers.",
    built_in: false,
  });

  assert.deepEqual(promptDraftFromPack(prompt()), {
    mode: "fork",
    prompt_pack_id: null,
    source_prompt_pack_id: "general-default",
    name: "Custom General",
    distribution_folder: "user",
    style_tags_text: "general",
    scene_profile: "general",
    shared_preference: "Prefer clean keepers.",
    built_in: true,
  });
});

test("newPromptDraft and promptStyleTagsFromText normalize prompt editor input", () => {
  assert.deepEqual(newPromptDraft(), {
    mode: "create",
    prompt_pack_id: null,
    source_prompt_pack_id: null,
    name: "",
    distribution_folder: "user",
    style_tags_text: "",
    scene_profile: "general",
    shared_preference: "",
    built_in: false,
  });
  assert.deepEqual(promptStyleTagsFromText(" portrait,  wedding  documentary "), [
    "portrait",
    "wedding",
    "documentary",
  ]);
});

test("technicalPolicyForCvPolicy mirrors Android threshold presets", () => {
  assert.deepEqual(technicalPolicyForCvPolicy("loose"), {
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
  });
  assert.equal(technicalPolicyForCvPolicy("standard").face_eye_open_warn_threshold, 0.35);
  assert.equal(technicalPolicyForCvPolicy("strict").highlight_clip_threshold, 242);
});

test("settingsForCvThresholdMode preserves the preset while enabling custom overrides", () => {
  const base = settings({ cv_policy: "strict", cv_policy_overrides: null });

  const custom = settingsForCvThresholdMode(base, "custom");

  assert.equal(custom.cv_policy, "strict");
  assert.equal(selectedCvThresholdMode(custom), "custom");
  assert.deepEqual(custom.cv_policy_overrides, technicalPolicyForCvPolicy("strict"));

  const standard = settingsForCvThresholdMode(custom, "standard");
  assert.equal(standard.cv_policy, "standard");
  assert.equal(standard.cv_policy_overrides, null);
  assert.equal(selectedCvThresholdMode(standard), "standard");
});

test("cvThresholdControlSpecs exposes face controls only for portrait projects", () => {
  const policy = technicalPolicyForCvPolicy("standard");
  const generalKeys = cvThresholdControlSpecs(policy, "general").map((control) => control.key);
  const portraitKeys = cvThresholdControlSpecs(policy, "portrait").map((control) => control.key);

  assert.deepEqual(generalKeys, [
    "blur",
    "clipping",
    "shadow_clip",
    "highlight_clip",
    "color_cast",
  ]);
  assert.deepEqual(portraitKeys.slice(-3), ["face_eyes", "face_exposure", "face_color_cast"]);

  const stricterFaceExposure = updateCvThresholdControl(policy, "face_exposure", 1);
  assert.equal(stricterFaceExposure.face_exposure_warn_ratio, 0.12);
});
