export type PreviewStage = "idle" | "queued" | "loading" | "fast" | "full" | "original" | "failed";

export type PreviewBadge = {
  label: string;
  tone: "waiting" | "working" | "low" | "high" | "original" | "failed" | "empty";
  title: string;
};

export type PreviewProgress = {
  total: number;
  high: number;
  low: number;
  pending: number;
  failed: number;
  label: string;
};

export function previewBadge(stage: PreviewStage): PreviewBadge {
  switch (stage) {
    case "queued":
      return { label: "排队", tone: "waiting", title: "预览已进入队列" };
    case "loading":
      return { label: "生成", tone: "working", title: "正在生成预览" };
    case "fast":
      return { label: "低清", tone: "low", title: "已显示内嵌或快速预览，正在等待高清预览" };
    case "full":
      return { label: "高清", tone: "high", title: "高清预览已就绪" };
    case "original":
      return { label: "原图", tone: "original", title: "正在使用原图预览" };
    case "failed":
      return { label: "失败", tone: "failed", title: "预览生成失败" };
    case "idle":
    default:
      return { label: "等待", tone: "empty", title: "等待进入预览队列" };
  }
}

export function previewProgress(stages: PreviewStage[]): PreviewProgress {
  const total = stages.length;
  const high = stages.filter((stage) => stage === "full" || stage === "original").length;
  const low = stages.filter((stage) => stage === "fast").length;
  const pending = stages.filter((stage) => stage === "queued" || stage === "loading").length;
  const failed = stages.filter((stage) => stage === "failed").length;
  const parts = [`高清 ${high}/${total}`];
  if (low) parts.push(`低清 ${low}`);
  if (pending) parts.push(`待处理 ${pending}`);
  if (failed) parts.push(`失败 ${failed}`);
  return {
    total,
    high,
    low,
    pending,
    failed,
    label: total ? parts.join("，") : "暂无预览任务",
  };
}
