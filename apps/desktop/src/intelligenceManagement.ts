import type { AppState, SettingsPanel } from "./appTypes";
import {
  newPromptDraft,
  promptDraftFromPack,
  promptDraftIsSaveable,
  providerDraftFromSettings,
  providerDraftIsSaveable,
  type ProjectEvaluationSettings,
  type PromptDraft,
  type SaveModelProviderSettingsRequest,
} from "./intelligence";
import { promptDraftModeLabel, readable } from "./presentation";
import {
  append,
  commandButton,
  el,
  numberInput,
  passwordInput,
  renderIntelligenceField,
  renderToggleRow,
  selectControl,
  textAreaInput,
  textInput,
} from "./domHelpers";

export type IntelligenceManagementRenderer = {
  renderProviderManagement(scope?: SettingsPanel): HTMLElement;
  renderPromptManagement(scope?: SettingsPanel): HTMLElement;
};

type IntelligenceManagementContext = {
  state: AppState;
  render: () => void;
  saveIntelligenceSettings: (patch: Partial<ProjectEvaluationSettings>) => void | Promise<void>;
  saveProviderDraft: () => void | Promise<void>;
  deleteProvider: (settingsId: string) => void | Promise<void>;
  savePromptDraft: () => void | Promise<void>;
  deletePromptPack: (promptPackId: string) => void | Promise<void>;
};

export function createIntelligenceManagementRenderer({
  state,
  render,
  saveIntelligenceSettings,
  saveProviderDraft,
  deleteProvider,
  savePromptDraft,
  deletePromptPack,
}: IntelligenceManagementContext): IntelligenceManagementRenderer {

  function renderProviderManagement(scope: SettingsPanel = "project") {
    const projectScope = scope === "project";
    const section = el("section", "drawer-section");
    append(
      section,
      append(
        el("div", "drawer-section-head"),
        el("h3", "", "AI 鏈嶅姟"),
        commandButton("鏂板缓鏈嶅姟", "side-link", () => {
          state.promptDraft = null;
          state.providerDraft = providerDraftFromSettings(null);
          render();
        }),
      ),
    );
    const list = el("div", "management-list");
    if (!state.intelligenceProviders.length) {
      append(list, el("p", "side-note", "鏆傛棤鏈嶅姟"));
    }
    for (const provider of state.intelligenceProviders) {
      const selected = state.intelligenceSettings?.model_provider_settings_id === provider.settings_id;
      const row = el("div", selected ? "management-row is-selected" : "management-row");
      append(
        row,
        append(
          el("div", "management-copy"),
          el("strong", "", provider.provider_label || provider.settings_id),
          el("span", "", `${readable(provider.provider_kind)} / ${provider.default_model || "No model name"}${provider.api_key_configured ? "" : " / No API key"}`),
        ),
        projectScope
          ? commandButton(selected ? "宸查€?" : "閫夌敤", "micro-button", () => void saveIntelligenceSettings({ model_provider_settings_id: provider.settings_id }), Boolean(state.busy || selected))
          : selected
            ? el("span", "management-tag", "褰撳墠椤圭洰")
            : null,
        commandButton("缂栬緫", "micro-button", () => {
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
      renderIntelligenceField("鍚嶇О", textInput(draft.provider_label, "OpenAI", (value) => {
        updateDraft({ provider_label: value });
      })),
      renderIntelligenceField(
        "鏈嶅姟绫诲瀷",
        selectControl(
          draft.provider_kind,
          [
            ["openai", "OpenAI"],
            ["custom", "鍏煎 OpenAI 鎺ュ彛"],
            ["none", "涓嶅惎鐢?"],
          ],
          (value) => {
            updateDraft({ provider_kind: value });
          },
        ),
      ),
      renderIntelligenceField("Base URL", textInput(draft.base_url, "https://api.openai.com/v1", (value) => {
        updateDraft({ base_url: value });
      })),
      renderIntelligenceField("妯″瀷", textInput(draft.default_model, "gpt-5-mini", (value) => {
        updateDraft({ default_model: value });
      })),
      renderIntelligenceField("API Key", passwordInput(draft.api_key ?? "", "鐣欑┖琛ㄧず淇濈暀宸叉湁瀵嗛挜", (value) => {
        updateDraft({ api_key: value || null });
      })),
      renderIntelligenceField("瀵嗛挜鍒悕", textInput(draft.key_alias ?? "", "OPENAI_API_KEY", (value) => {
        updateDraft({ key_alias: value || null });
      })),
      renderIntelligenceField("鍥剧墖闀胯竟", numberInput(draft.default_max_image_side, 256, 4096, (value) => {
        updateDraft({ default_max_image_side: value });
      })),
      renderIntelligenceField("鎵归噺鏁伴噺", numberInput(draft.default_batch_size, 1, 32, (value) => {
        updateDraft({ default_batch_size: value });
      })),
      renderToggleRow("鍚敤璇ユ湇鍔?", draft.configured, (checked) => {
        updateDraft({ configured: checked });
      }),
      append(
        el("div", "editor-actions"),
        (saveButton = commandButton("淇濆瓨鏈嶅姟", "primary", () => void saveProviderDraft(), Boolean(state.busy || !providerDraftIsSaveable(draft)))),
        state.intelligenceProviders.some((provider) => provider.settings_id === draft.settings_id)
          ? commandButton("鍒犻櫎鏈嶅姟", "secondary danger-text", () => void deleteProvider(draft.settings_id), Boolean(state.busy))
          : null,
        commandButton("鍙栨秷", "secondary", () => {
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
        el("h3", "", "閫夌墖瑙勫垯"),
        commandButton("鏂板缓", "side-link", () => {
          state.providerDraft = null;
          state.promptDraft = newPromptDraft();
          render();
        }),
      ),
    );
    const list = el("div", "management-list prompt-list");
    if (!packs.length) {
      append(list, el("p", "side-note", "杩樻病鏈夊彲鐢ㄩ€夌墖瑙勫垯銆?"));
    }
    for (const prompt of packs) {
      const selected = state.intelligenceSettings?.prompt_pack_id === prompt.prompt_pack_id;
      const row = el("div", selected ? "management-row is-selected" : "management-row");
      append(
        row,
        append(
          el("div", "management-copy"),
          el("strong", "", prompt.name),
          el("span", "", `${prompt.built_in ? "鍐呯疆" : "鐢ㄦ埛"} / ${readable(prompt.scene_profile)} / ${prompt.distribution_folder}`),
        ),
        projectScope
          ? commandButton(selected ? "宸查€?" : "閫夌敤", "micro-button", () => void saveIntelligenceSettings({ prompt_pack_id: prompt.prompt_pack_id }), Boolean(state.busy || selected))
          : selected
            ? el("span", "management-tag", "褰撳墠椤圭洰")
            : null,
        commandButton(prompt.built_in ? "澶嶅埗" : "缂栬緫", "micro-button", () => {
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
      renderIntelligenceField("鍚嶇О", textInput(draft.name, "渚嬪锛氬绀肩邯瀹炵簿閫?", (value) => {
        updateDraft({ name: value });
      })),
      renderIntelligenceField("鏂囦欢澶?", textInput(draft.distribution_folder, "user", (value) => {
        updateDraft({ distribution_folder: value });
      })),
      renderIntelligenceField(
        "鎷嶆憚鍦烘櫙",
        selectControl(
          draft.scene_profile,
          [
            ["general", "閫氱敤"],
            ["portrait", "浜哄儚"],
            ["action", "杩愬姩"],
            ["landscape", "椋庡厜"],
            ["custom", "鑷畾涔?"],
          ],
          (value) => {
            updateDraft({ scene_profile: value });
          },
        ),
      ),
      renderIntelligenceField("椋庢牸鏍囩", textInput(draft.style_tags_text, "濠氱ぜ 浜哄儚 绾疄", (value) => {
        updateDraft({ style_tags_text: value });
      })),
      renderIntelligenceField("閫夌墖鍋忓ソ", textAreaInput(draft.shared_preference, "鍐欎笅閫夌墖鍋忓ソ銆佷紭鍏堢骇鍜屾窐姹拌鍒欍€?", (value) => {
        updateDraft({ shared_preference: value });
      })),
      append(
        el("div", "editor-actions"),
        (saveButton = commandButton(draft.mode === "edit" ? "淇濆瓨" : draft.mode === "fork" ? "澶嶅埗" : "鍒涘缓", "primary", () => void savePromptDraft(), Boolean(state.busy || !promptDraftIsSaveable(draft)))),
        draft.mode === "edit" && draft.prompt_pack_id
          ? commandButton("绉婚櫎鏈湴鍖?", "secondary danger-text", () => void deletePromptPack(draft.prompt_pack_id as string), Boolean(state.busy))
          : null,
        commandButton("鍙栨秷", "secondary", () => {
          state.promptDraft = null;
          render();
        }),
      ),
    );
    return form;
  }

  return { renderProviderManagement, renderPromptManagement };
}
