import type { StoredAsset } from "./appTypes";
import { append, el, statusChip } from "./domHelpers";
import { formatBytes } from "./presentation";

export function renderFilesPanel(groupDetail: StoredAsset[]) {
  const panel = el("section", "detail-panel");
  append(panel, el("h3", "", "鏂囦欢"));
  const list = el("div", "file-list");
  if (!groupDetail.length) {
    append(list, el("div", "empty-note", "閫夋嫨涓€涓収鐗囩粍鍚庢煡鐪嬫枃浠舵槑缁嗐€?"));
  }
  for (const asset of groupDetail) {
    append(
      list,
      append(
        el("div", "file-item"),
        append(el("div", "file-name"), el("strong", "", asset.original_filename), el("span", "", asset.original_path)),
        statusChip(asset.source_status, "source"),
        el("span", "file-size", formatBytes(asset.size_bytes)),
      ),
    );
  }
  append(panel, list);
  return panel;
}
