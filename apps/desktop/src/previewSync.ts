export type PreviewSyncQuality = "fast" | "full" | "original";

export type PreviewSyncNode = {
  previewPath?: string;
  previewMaxEdge?: string;
  previewFullPending?: string;
  currentQuality?: PreviewSyncQuality;
};

export type PreviewSyncItem = {
  localPath: string;
  maxEdge: string;
  quality: PreviewSyncQuality;
  url: string | null;
};

export function shouldApplyPreviewSync(node: PreviewSyncNode, item: PreviewSyncItem) {
  if (!item.url || node.previewPath !== item.localPath || node.previewMaxEdge !== item.maxEdge) {
    return false;
  }
  if (item.quality === "fast" && (node.currentQuality === "full" || node.currentQuality === "original")) {
    return false;
  }
  if (item.quality === "full" && node.currentQuality === "original") {
    return false;
  }
  if (item.quality === "full" && node.previewFullPending && node.previewFullPending !== `full:${item.maxEdge}:${item.localPath}`) {
    return false;
  }
  return true;
}
