import { state } from "./appState";
import { folderBasename } from "./groupSelectors";
import { append, commandButton, el, textInput } from "./domHelpers";

export type ProjectCreateVariant = "compact" | "hero";

export type ProjectCreateFormOptions = {
  createProject: () => Promise<void>;
  chooseProjectFolderDraft: () => Promise<void>;
};

export function renderProjectCreateForm(options: ProjectCreateFormOptions, variant: ProjectCreateVariant = "compact") {
  const row = el("form", `project-create project-create-${variant}`);
  row.addEventListener("submit", (event) => {
    event.preventDefault();
    void options.createProject();
  });
  const folderName = state.projectFolderDraft ? folderBasename(state.projectFolderDraft) : "閫夋嫨鐓х墖鏂囦欢澶?";
  const folderPicker = commandButton("", state.projectFolderDraft ? "project-folder-picker has-folder" : "project-folder-picker", () => void options.chooseProjectFolderDraft(), Boolean(state.busy));
  folderPicker.title = state.projectFolderDraft || "閫夋嫨鏈湴鐓х墖鏂囦欢澶?";
  append(
    folderPicker,
    el("span", "project-folder-kicker", "鐓х墖鏂囦欢澶?"),
    el("strong", "", folderName),
    el("small", "", state.projectFolderDraft || "閫掑綊鍖呭惈瀛愭枃浠跺す"),
  );
  append(
    row,
    textInput(state.projectNameDraft, "椤圭洰鍚嶇О", (value) => {
      state.projectNameDraft = value;
    }),
    folderPicker,
    append(
      el("div", "project-create-actions"),
      commandButton("Create and index", variant === "hero" ? "primary large" : "primary", () => void options.createProject(), Boolean(state.busy)),
    ),
  );
  return row;
}
