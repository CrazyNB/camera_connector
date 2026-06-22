import type {
  DesktopError,
  ReceivedAsset,
  ReceivedAssetGroup,
  ReceivedAssetTechnicalDefectSummary,
} from "./appTypes";
import type { PromptDraft } from "./intelligence";

export function formatPairLabel(group: ReceivedAssetGroup) {
  if (group.raw && group.jpeg) return "RAW+JPG";
  if (group.raw) return "RAW";
  if (group.jpeg) return "JPG";
  if (group.video) return "MOV";
  return readable(group.primary.format);
}

export function scanTransferDot(health: string) {
  switch (health) {
    case "ready":
      return "available";
    case "working":
      return "changed";
    case "failed":
      return "missing";
    default:
      return "neutral";
  }
}

export function evaluationDot(group: ReceivedAssetGroup) {
  if (group.technical_defects.length) return "missing";
  if (typeof group.model_score === "number") return "available";
  const technical = cssToken(group.technical_gate_status ?? group.technical_status ?? "pending");
  const model = cssToken(group.model_status ?? "pending");
  if (["failed", "reject"].includes(technical) || ["failed", "reject"].includes(model)) return "missing";
  if (["pass", "ready", "completed"].includes(technical) || ["ready", "completed"].includes(model)) return "available";
  return "changed";
}

export function compactEvaluationLabel(group: ReceivedAssetGroup) {
  if (group.technical_defects.length) return "需复核";
  if (typeof group.model_score === "number") return `${group.model_score} ${readable(group.model_tier ?? "score")}`;
  const technical = cssToken(group.technical_gate_status ?? group.technical_status ?? "pending");
  const model = cssToken(group.model_status ?? "pending");
  if (["failed", "reject"].includes(technical) || ["failed", "reject"].includes(model)) return "评价失败";
  if (["pass", "ready", "completed"].includes(technical) || ["ready", "completed"].includes(model)) return "已评价";
  return "待评价";
}

export function clamp(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value));
}

export function compactError(error: string | null) {
  if (!error) return null;
  return error.length > 130 ? `${error.slice(0, 127)}...` : error;
}

export function scanIsActive(phase?: string | null) {
  return Boolean(phase && ["queued", "scanning", "indexing"].includes(phase));
}

export function errorMessage(error: unknown): string {
  if (typeof error === "string") {
    return error;
  }
  if (error && typeof error === "object") {
    const desktopError = error as DesktopError;
    if (desktopError.message) {
      return desktopError.code ? `${desktopError.code}: ${desktopError.message}` : desktopError.message;
    }
  }
  return "Unexpected desktop error";
}

export function membersOf(group: ReceivedAssetGroup) {
  const members = [group.primary, group.jpeg, group.raw, group.video].filter(Boolean) as ReceivedAsset[];
  const seen = new Set<string>();
  return members.filter((member) => {
    if (seen.has(member.id)) return false;
    seen.add(member.id);
    return true;
  });
}

export function sourceStatus(group: ReceivedAssetGroup) {
  const statuses = membersOf(group).map((asset) => asset.source_status ?? "available");
  if (statuses.includes("missing")) return "missing";
  if (statuses.includes("changed")) return "changed";
  return "available";
}

export function technicalGateStatusLabel(value: string | null | undefined) {
  switch (cssToken(value ?? "")) {
    case "pass":
      return "通过";
    case "warn":
      return "有风险";
    case "reject":
      return "严重风险";
    case "inconclusive":
      return "无法判断";
    case "unsupported":
      return "暂不支持";
    case "":
    case "pending":
    case "technical-pending":
      return "待检查";
    default:
      return readable(value ?? "pending");
  }
}

export function modelStatusLabel(value: string | null | undefined) {
  switch (cssToken(value ?? "")) {
    case "ready":
    case "done":
    case "completed":
    case "evaluated":
      return "已评价";
    case "running":
    case "processing":
    case "analyzing":
      return "评价中";
    case "failed":
    case "error":
      return "评价失败";
    case "skipped":
    case "":
      return "未评价";
    case "pending":
    case "queued":
      return "待评价";
    default:
      return readable(value ?? "pending");
  }
}

export function modelTierLabel(value: string | null | undefined) {
  switch (cssToken(value ?? "")) {
    case "excellent":
      return "优秀";
    case "good":
      return "良好";
    case "normal":
      return "普通";
    case "weak":
      return "偏弱";
    case "reject":
      return "不建议入选";
    case "":
    case "none":
      return "未知";
    default:
      return readable(value ?? "none");
  }
}

export function recommendationStatusLabel(value: string | null | undefined) {
  switch (cssToken(value ?? "")) {
    case "recommended":
    case "completed":
    case "ready":
    case "done":
      return "已推荐";
    case "running":
    case "processing":
    case "analyzing":
      return "推荐中";
    case "stale":
      return "更新中";
    case "no-selection":
    case "none":
      return "未推荐";
    case "unsupported":
      return "不支持推荐";
    case "failed":
    case "error":
      return "推荐失败";
    case "":
    case "pending":
    case "queued":
      return "待推荐";
    default:
      return readable(value ?? "pending");
  }
}

export function userFacingTechnicalDefect(defect: ReceivedAssetTechnicalDefectSummary) {
  const type = cssToken(defect.defect_type);
  const severity = cssToken(defect.severity);
  if (type === "blur") {
    if (severity === "severe") return "严重失焦";
    if (severity === "high") return "失焦";
    if (severity === "medium") return "清晰度偏软";
    if (severity === "low") return "细节略软";
    return "画面不够清晰";
  }
  if (type === "highlight-clip") {
    if (severity === "severe") return "大面积过曝";
    if (severity === "high") return "过曝";
    if (severity === "medium") return "局部过曝";
    if (severity === "low") return "高光略有溢出";
    return "高光过曝";
  }
  if (type === "shadow-clip") {
    if (severity === "severe") return "大面积死黑";
    if (severity === "high") return "暗部死黑";
    if (severity === "medium") return "暗部略有死黑";
    if (severity === "low") return "暗部略暗";
    return "暗部死黑";
  }
  if (type === "noise") {
    if (severity === "severe") return "高噪点明显";
    if (severity === "high") return "噪点偏高";
    if (severity === "medium") return "细节略脏";
    if (severity === "low") return "轻微噪点";
    return "噪点偏高";
  }
  if (type === "color-cast") {
    if (severity === "severe") return "严重偏色";
    if (severity === "high") return "偏色明显";
    if (severity === "medium") return "色彩偏色";
    if (severity === "low") return "轻微偏色";
    return "色彩偏色";
  }
  if (type === "unsupported") {
    return "需人工确认";
  }
  return defect.reason?.trim() || readable(defect.defect_type);
}

export function modelLabel(group: ReceivedAssetGroup) {
  if (typeof group.model_score === "number") {
    return `${group.model_score} ${modelTierLabel(group.model_tier ?? "model")}`;
  }
  return modelStatusLabel(group.model_status ?? "pending");
}

export function checkResultDot(status: string) {
  const token = cssToken(status);
  if (["available", "pass", "ready", "completed", "evaluated"].includes(token)) return "available";
  if (["missing", "failed", "reject"].includes(token)) return "missing";
  if (["changed", "pending", "queued", "setup"].includes(token)) return "changed";
  return "neutral";
}

export function readable(value: string) {
  const normalized = value.replace(/_/g, " ").trim().toLowerCase();
  const mapped = READABLE_LABELS[normalized];
  if (mapped) return mapped;
  return value
    .replace(/_/g, " ")
    .replace(/([a-z])([A-Z])/g, "$1 $2")
    .replace(/\b\w/g, (letter) => letter.toUpperCase());
}

const READABLE_LABELS: Record<string, string> = {
  action: "运动",
  all: "全部",
  available: "可用",
  auto: "自动",
  changed: "已变化",
  completed: "已完成",
  custom: "自定义",
  disabled: "未启用",
  evaluated: "已评价",
  failed: "失败",
  general: "通用",
  indexed: "已索引",
  inconclusive: "无法判断",
  landscape: "风光",
  loose: "宽松",
  missing: "缺失",
  none: "无",
  "not generated": "未生成",
  openai: "OpenAI",
  pass: "通过",
  pending: "待处理",
  portrait: "人像",
  model: "AI",
  prompt: "选片规则",
  provider: "AI 服务",
  queued: "已排队",
  ready: "就绪",
  reject: "淘汰",
  scanning: "扫描中",
  score: "分",
  setup: "待设置",
  standard: "标准",
  strict: "严格",
  unsupported: "暂不支持",
  warn: "有风险",
  "model select": "AI 推荐",
  "technical pending": "质量待查",
};

export function promptDraftModeLabel(mode: PromptDraft["mode"]) {
  if (mode === "create") return "新建选片规则";
  if (mode === "fork") return "复制选片规则";
  return "编辑选片规则";
}

export function cssToken(value: string) {
  return value.toLowerCase().replace(/[^a-z0-9]+/g, "-");
}

export function formatBytes(bytes: number) {
  if (bytes < 1024) return `${bytes} B`;
  const units = ["KB", "MB", "GB", "TB"];
  let value = bytes / 1024;
  let unit = units[0];
  for (let index = 1; value >= 1024 && index < units.length; index += 1) {
    value /= 1024;
    unit = units[index];
  }
  return `${value.toFixed(value >= 10 ? 0 : 1)} ${unit}`;
}
