import type { ReceivedAssetGroup, SettingsPanel } from "./appTypes";
import type { IntelligenceSetupState } from "./intelligence";
import {
  cvThresholdControlSpecs,
  selectedCvThresholdMode,
  settingsForCvThresholdMode,
  technicalPolicyForCvPolicy,
  updateCvThresholdControl,
  type CvThresholdMode,
  type ProjectEvaluationSettings,
} from "./intelligence";
import {
  append,
  commandButton,
  el,
  renderIntelligenceField,
  renderToggleRow,
  selectControl,
  settingsSectionHead,
} from "./domHelpers";
import { state } from "./appState";

export type SettingsDrawerOptions = {
  render: () => void;
  saveIntelligenceSettings: (patch: Partial<ProjectEvaluationSettings>) => Promise<void>;
  renderProviderManagement: (scope?: SettingsPanel) => HTMLElement;
  renderPromptManagement: (scope?: SettingsPanel) => HTMLElement;
  currentIntelligenceSetup: () => IntelligenceSetupState;
  filteredGroups: () => ReceivedAssetGroup[];
  evaluateLoadedGroupsWithModel: () => Promise<void>;
};
export function renderSettingsDrawer(panelKind: SettingsPanel, options: SettingsDrawerOptions) {
  const backdrop = el("div", "drawer-backdrop");
  backdrop.addEventListener("click", (event) => {
    if (event.target === backdrop) {
      state.settingsPanel = null;
      options.render();
    }
  });
  return panelKind === "global" ? renderGlobalSettingsDrawer(backdrop, options) : renderProjectSettingsDrawer(backdrop, options);
}

function renderProjectSettingsDrawer(backdrop: HTMLElement, options: SettingsDrawerOptions) {
  const settings = state.intelligenceSettings;
  const setup = options.currentIntelligenceSetup();
  const panel = el("section", "intelligence-drawer");
  append(
    panel,
    append(
      el("div", "drawer-head"),
      append(
        el("div", ""),
        el("p", "eyebrow", "椤圭洰璁剧疆"),
        el("h2", "", "AI 杈呭姪閫夌墖"),
      ),
      commandButton("鍏抽棴", "ghost", () => {
        state.settingsPanel = null;
        options.render();
      }),
    ),
  );

  if (!settings) {
    append(panel, el("p", "side-note", "鍏堥€夋嫨涓€涓」鐩紝鍐嶉厤缃?AI 杈呭姪閫夌墖銆?"));
    append(backdrop, panel);
    return backdrop;
  }

  append(
    panel,
    append(
      el("section", "settings-section"),
      settingsSectionHead("鍩虹閰嶇疆"),
      append(
        el("div", "settings-field-grid"),
        renderIntelligenceField(
          "AI 鏈嶅姟",
          selectControl(
            settings.model_provider_settings_id ?? "",
            [["", "涓嶄娇鐢?AI 鏈嶅姟"], ...state.intelligenceProviders.map((provider) => [provider.settings_id, provider.provider_label] as [string, string])],
            (value) => void options.saveIntelligenceSettings({ model_provider_settings_id: value || null }),
          ),
        ),
        renderIntelligenceField(
          "閫夌墖瑙勫垯",
          selectControl(
            settings.prompt_pack_id ?? "",
            [["", "涓嶇粦瀹氶€夌墖瑙勫垯"], ...state.promptPacks.map((prompt) => [prompt.prompt_pack_id, prompt.name] as [string, string])],
            (value) => void options.saveIntelligenceSettings({ prompt_pack_id: value || null }),
          ),
        ),
        renderIntelligenceField(
          "鎷嶆憚鍦烘櫙",
          selectControl(
            settings.scene_profile,
            [
              ["general", "閫氱敤"],
              ["portrait", "浜哄儚"],
              ["action", "杩愬姩"],
              ["landscape", "椋庡厜"],
              ["custom", "鑷畾涔?"],
            ],
            (value) => void options.saveIntelligenceSettings({ scene_profile: value }),
          ),
        ),
      ),
    ),
    append(
      el("section", "settings-section"),
      settingsSectionHead("璐ㄩ噺椋庨櫓"),
      renderCvThresholdSettings(settings, options),
    ),
    append(
      el("section", "settings-section"),
      settingsSectionHead("鑷姩鍖?"),
      append(
        el("div", "settings-toggle-list"),
        renderToggleRow("鎵弿鍚庤嚜鍔?AI 璇勪环", settings.auto_evaluate_on_upload, (checked) =>
          void options.saveIntelligenceSettings({ auto_evaluate_on_upload: checked }),
        ),
        renderToggleRow("鑷姩鐢熸垚杩炴媿鎺ㄨ崘", settings.auto_burst_recommendation_enabled, (checked) =>
          void options.saveIntelligenceSettings({ auto_burst_recommendation_enabled: checked }),
        ),
        renderToggleRow("鍏佽 AI 閫変腑鏈夐闄╃収鐗?", settings.allow_risky_model_selects, (checked) =>
          void options.saveIntelligenceSettings({ allow_risky_model_selects: checked }),
        ),
      ),
    ),
    renderLoadedModelEvaluationPanel(setup, options),
  );

  append(backdrop, panel);
  return backdrop;
}

function renderCvThresholdSettings(settings: ProjectEvaluationSettings, options: SettingsDrawerOptions) {
  const mode = selectedCvThresholdMode(settings);
  const panel = el("section", "cv-threshold-panel");
  append(
    panel,
    renderIntelligenceField(
      "闃堝€兼柟妗?",
      selectControl(
        mode,
        [
          ["loose", "瀹芥澗"],
          ["standard", "鏍囧噯"],
          ["strict", "涓ユ牸"],
          ["custom", "鑷畾涔?"],
        ],
        (value) => {
          const next = settingsForCvThresholdMode(settings, value as CvThresholdMode);
          void options.saveIntelligenceSettings({
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
      el("strong", "", "鑷畾涔夊弬鏁?"),
      commandButton("鎭㈠棰勮", "micro-button", () =>
        void options.saveIntelligenceSettings({
          cv_policy_overrides: technicalPolicyForCvPolicy(settings.cv_policy),
        }),
      ),
    ),
  );
  for (const control of controls) {
    append(panel, renderCvThresholdControl(settings, policy, control, options));
  }
  return panel;
}

function renderCvThresholdControl(
  settings: ProjectEvaluationSettings,
  policy: NonNullable<ProjectEvaluationSettings["cv_policy_overrides"]>,
  control: ReturnType<typeof cvThresholdControlSpecs>[number],
  options: SettingsDrawerOptions,
) {
  const input = el("input", "cv-threshold-slider") as HTMLInputElement;
  input.type = "range";
  input.min = "0";
  input.max = "1";
  input.step = "0.01";
  input.value = String(control.sliderValue);
  input.addEventListener("change", () => {
    const nextPolicy = updateCvThresholdControl(policy, control.key, Number(input.value));
    void options.saveIntelligenceSettings({
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

function renderGlobalSettingsDrawer(backdrop: HTMLElement, options: SettingsDrawerOptions) {
  const panel = el("section", "intelligence-drawer is-global-settings");
  append(
    panel,
    append(
      el("div", "drawer-head"),
      append(
        el("div", ""),
        el("p", "eyebrow", "鍏ㄥ眬璁剧疆"),
        el("h2", "", "AI 涓庨€夌墖瑙勫垯"),
      ),
      commandButton("鍏抽棴", "ghost", () => {
        state.settingsPanel = null;
        options.render();
      }),
    ),
    append(el("div", "global-settings-grid"), options.renderProviderManagement("global"), options.renderPromptManagement("global")),
  );
  append(backdrop, panel);
  return backdrop;
}

function renderLoadedModelEvaluationPanel(setup: IntelligenceSetupState, options: SettingsDrawerOptions) {
  const groupIds = options.filteredGroups()
    .map((group) => group.group_id)
    .filter((groupId): groupId is string => Boolean(groupId));
  return append(
    el("section", "drawer-section"),
    append(el("div", "drawer-section-head"), el("h3", "", "鎵ц"), el("span", "", `${groupIds.length} 缁刞`)),
    commandButton("璇勪环褰撳墠瑙嗗浘", "source-action", () => void options.evaluateLoadedGroupsWithModel(), Boolean(state.busy || !setup.modelReady || !groupIds.length)),
  );
}
