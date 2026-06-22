import type { ReceivedAssetGroup, SubjectAssessment, SubjectRegion, SubjectSignals } from "./appTypes";
import { append, el } from "./domHelpers";
import { cssToken } from "./presentation";

type SubjectAssessmentsByGroup = Record<string, SubjectAssessment[]>;

export type FaceRiskLayerOptions = {
  onViewerPreview?: (container: HTMLElement) => void;
};

export function renderFaceRiskOverlay(
  group: ReceivedAssetGroup,
  assessments: SubjectAssessmentsByGroup,
  options: FaceRiskLayerOptions = {},
) {
  const assessment = latestFaceAssessment(group, assessments);
  const regions = assessment ? subjectRegions(assessment) : [];
  const signals = assessment ? subjectSignals(assessment) : {};
  const imageWidth = signals.image_width ?? 0;
  const imageHeight = signals.image_height ?? 0;
  const layer = el("div", "face-risk-layer");
  if (!assessment || !regions.length || imageWidth <= 0 || imageHeight <= 0) {
    layer.hidden = true;
    return layer;
  }
  layer.dataset.imageWidth = String(imageWidth);
  layer.dataset.imageHeight = String(imageHeight);
  layer.title = assessment.summary;
  for (const region of regions) {
    const x = finiteNumber(region.x);
    const y = finiteNumber(region.y);
    const width = finiteNumber(region.width ?? region.w);
    const height = finiteNumber(region.height ?? region.h);
    if (x === null || y === null || width === null || height === null || width <= 0 || height <= 0) {
      continue;
    }
    const box = el("span", "face-risk-box");
    box.style.left = `${(x / imageWidth) * 100}%`;
    box.style.top = `${(y / imageHeight) * 100}%`;
    box.style.width = `${(width / imageWidth) * 100}%`;
    box.style.height = `${(height / imageHeight) * 100}%`;
    const label = faceRiskLabel(assessment, signals);
    if (label) append(box, el("span", "face-risk-label", label));
    append(layer, box);
  }
  if (!layer.childElementCount) {
    layer.hidden = true;
  }
  requestAnimationFrame(() => {
    const parent = layer.parentElement;
    if (parent) syncFaceRiskLayer(parent, options);
  });
  return layer;
}

export function latestFaceAssessment(group: ReceivedAssetGroup, assessments: SubjectAssessmentsByGroup) {
  if (!group.group_id) return null;
  return (
    assessments[group.group_id]
      ?.filter((assessment) => assessment.subject_type === "face")
      .find((assessment) => subjectRegions(assessment).length > 0) ?? null
  );
}

function subjectRegions(assessment: SubjectAssessment): SubjectRegion[] {
  const parsed = parseJson<unknown>(assessment.regions_json, []);
  return Array.isArray(parsed) ? (parsed as SubjectRegion[]) : [];
}

export function subjectSignals(assessment: SubjectAssessment): SubjectSignals {
  const parsed = parseJson<unknown>(assessment.signals_json, {});
  return parsed && typeof parsed === "object" && !Array.isArray(parsed) ? (parsed as SubjectSignals) : {};
}

function parseJson<T>(source: string, fallback: T): T {
  try {
    return JSON.parse(source) as T;
  } catch {
    return fallback;
  }
}

function finiteNumber(value: unknown): number | null {
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

function faceRiskLabel(assessment: SubjectAssessment, signals: SubjectSignals) {
  if (cssToken(assessment.gate_status) === "pass") return "";
  if (signals.closed_eyes) return "闂溂";
  if (signals.face_exposure_risk) return "闈㈤儴鏇濆厜";
  if (signals.face_color_cast_risk) return "鍋忚壊";
  return cssToken(assessment.gate_status) === "warn" ? "椋庨櫓" : "";
}

export function syncAllFaceRiskLayers(options: FaceRiskLayerOptions = {}) {
  document.querySelectorAll<HTMLElement>(".face-risk-layer").forEach((layer) => {
    const parent = layer.parentElement;
    if (parent) syncFaceRiskLayer(parent, options);
  });
}

export function syncFaceRiskLayer(container: HTMLElement, options: FaceRiskLayerOptions = {}) {
  const layer = container.querySelector<HTMLElement>(":scope > .face-risk-layer");
  const image = container.querySelector<HTMLImageElement>(":scope > img.preview-image");
  if (!layer || !image || layer.hidden || !image.naturalWidth || !image.naturalHeight) {
    return;
  }
  const containerWidth = container.clientWidth;
  const containerHeight = container.clientHeight;
  if (containerWidth <= 0 || containerHeight <= 0) {
    return;
  }
  if (container.classList.contains("viewer-main-preview")) {
    options.onViewerPreview?.(container);
    return;
  }
  const imageRatio = image.naturalWidth / image.naturalHeight;
  const containerRatio = containerWidth / containerHeight;
  const objectFit = getComputedStyle(image).objectFit;
  const contain = objectFit === "contain";
  const scale = contain
    ? containerRatio > imageRatio
      ? containerHeight / image.naturalHeight
      : containerWidth / image.naturalWidth
    : containerRatio > imageRatio
      ? containerWidth / image.naturalWidth
      : containerHeight / image.naturalHeight;
  const renderedWidth = image.naturalWidth * scale;
  const renderedHeight = image.naturalHeight * scale;
  layer.style.left = `${(containerWidth - renderedWidth) / 2}px`;
  layer.style.top = `${(containerHeight - renderedHeight) / 2}px`;
  layer.style.width = `${renderedWidth}px`;
  layer.style.height = `${renderedHeight}px`;
}
