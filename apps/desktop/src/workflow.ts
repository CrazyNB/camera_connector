export type ScanStartInput = {
  hasProject: boolean;
  hasRootPath: boolean;
  busy: boolean;
  scanPhase: string | null;
};

export type ScanStartBlocker = "project" | "folder" | "busy" | "active_scan";

export type ScanTransferInput = {
  scanPhase: string | null;
  scanFilesSeen: number;
  scanAssetsIndexed: number;
  scanGroupsUpdated: number;
  scanError: string | null;
  indexedAssetCount: number;
  indexedGroupCount: number;
};

export type ScanTransferDisplay = {
  label: string;
  health: "empty" | "ready" | "working" | "failed";
  files: number;
  groups: number;
  assets: number;
  note: string | null;
};

export function scanStartBlocker(input: ScanStartInput): ScanStartBlocker | null {
  if (!input.hasProject) {
    return "project";
  }
  if (!input.hasRootPath) {
    return "folder";
  }
  if (input.busy) {
    return "busy";
  }
  if (input.scanPhase && ["queued", "scanning", "indexing"].includes(input.scanPhase)) {
    return "active_scan";
  }
  return null;
}

export function scanTransferDisplay(input: ScanTransferInput): ScanTransferDisplay {
  const hasIndexedAssets = input.indexedAssetCount > 0 || input.indexedGroupCount > 0;
  const isActive = Boolean(input.scanPhase && ["queued", "scanning", "indexing"].includes(input.scanPhase));

  if (!input.scanPhase && !hasIndexedAssets) {
    return {
      label: "未扫描",
      health: "empty",
      files: 0,
      groups: 0,
      assets: 0,
      note: null,
    };
  }

  if (input.scanPhase === "failed" && hasIndexedAssets) {
    return {
      label: "已索引",
      health: "ready",
      files: input.indexedAssetCount,
      groups: input.indexedGroupCount,
      assets: input.indexedAssetCount,
      note: "上次重新扫描失败，现有索引仍可使用。",
    };
  }

  return {
    label: isActive ? "扫描中" : input.scanPhase === "failed" ? "扫描失败" : "已索引",
    health: isActive ? "working" : input.scanPhase === "failed" ? "failed" : "ready",
    files: input.scanFilesSeen || input.indexedAssetCount,
    groups: input.scanGroupsUpdated || input.indexedGroupCount,
    assets: input.scanAssetsIndexed || input.indexedAssetCount,
    note: input.scanPhase === "failed" ? compactScanError(input.scanError) : null,
  };
}

function compactScanError(error: string | null) {
  if (!error) return null;
  return error.length > 120 ? `${error.slice(0, 117)}...` : error;
}
