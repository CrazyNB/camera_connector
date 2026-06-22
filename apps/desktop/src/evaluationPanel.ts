import type { ReceivedAssetGroup, SubjectAssessment } from "./appTypes";
import { append, el, statusDot } from "./domHelpers";
import { latestFaceAssessment, subjectSignals } from "./faceRiskOverlay";
import {
  checkResultDot,
  cssToken,
  modelStatusLabel,
  readable,
  recommendationStatusLabel,
  sourceStatus,
  technicalGateStatusLabel,
  userFacingTechnicalDefect,
} from "./presentation";

type SubjectAssessmentsByGroup = Record<string, SubjectAssessment[]>;

export function renderEvaluationPanel(group: ReceivedAssetGroup, subjectAssessments: SubjectAssessmentsByGroup) {
  const panel = el("section", "detail-panel");
  append(
    panel,
    el("h3", "", "妫€鏌ョ粨鏋?"),
    qualityResultList(group, subjectAssessments),
  );
  if (group.model_summary) {
    append(panel, el("p", "summary-text", group.model_summary));
  }
  if (group.technical_defects.length) {
    const defects = el("div", "defects");
    for (const defect of group.technical_defects) {
      append(defects, el("div", "defect", userFacingTechnicalDefect(defect)));
    }
    append(panel, defects);
  }
  return panel;
}

function qualityResultList(group: ReceivedAssetGroup, subjectAssessments: SubjectAssessmentsByGroup) {
  const faceAssessment = latestFaceAssessment(group, subjectAssessments);
  const rows: Array<{ label: string; value: string; status: string; note?: string }> = [
    sourceResult(group),
    technicalResult(group),
    modelResult(group),
  ];
  if (faceAssessment) {
    rows.push(faceResult(faceAssessment));
  }
  rows.push(
    {
      label: "杩炴媿",
      value: group.burst ? `${group.burst.member_count} burst` : "None",
      status: group.burst ? "available" : "none",
      note: group.burst ? "Compare within the same burst sequence." : "Single photo group.",
    },
    {
      label: "鎺ㄨ崘",
      value: recommendationStatusLabel(group.burst?.recommendation_status ?? "none"),
      status: group.burst?.recommendation_status ?? "none",
    },
  );
  const list = el("div", "quality-result-list");
  for (const row of rows) {
    append(list, qualityResultRow(row.label, row.value, row.status, row.note));
  }
  return list;
}

function qualityResultRow(label: string, value: string, status: string, note = "") {
  const row = el("div", "quality-result-row");
  append(
    row,
    append(el("div", "quality-result-main"), append(el("span", "quality-label"), statusDot(checkResultDot(status)), el("span", "", label)), el("strong", "", value)),
  );
  if (note) {
    append(row, el("p", "", note));
  }
  return row;
}

function sourceResult(group: ReceivedAssetGroup) {
  const status = sourceStatus(group);
  const note =
    status === "available"
      ? "鍘熷浘璺緞鍙鍙栥€?"
      : status === "changed"
        ? "纾佺洏鏂囦欢宸插彉鍖栵紝寤鸿閲嶆柊鎵弿銆?"
        : status === "missing"
          ? "鍘熷浘璺緞缂哄け銆?"
          : "";
  return { label: "鏂囦欢", value: readable(status), status, note };
}

function technicalResult(group: ReceivedAssetGroup) {
  const status = group.technical_gate_status ?? group.technical_status ?? "pending";
  const token = cssToken(status);
  const value = group.technical_defects.length ? "闇€澶嶆牳" : technicalGateStatusLabel(status);
  const note = group.technical_defects.length
    ? group.technical_defects.slice(0, 2).map(userFacingTechnicalDefect).join(" / ")
    : technicalStatusNote(token);
  return { label: "璐ㄩ噺", value, status, note };
}

function modelResult(group: ReceivedAssetGroup) {
  const status = group.model_status ?? "pending";
  const tier = group.model_tier && cssToken(group.model_tier) !== "none" ? `锛?{modelTierLabel(group.model_tier)}` : "";
  return {
    label: "AI",
    value: typeof group.model_score === "number" ? `${modelStatusLabel(status)}${tier}` : modelStatusLabel(status),
    status,
    note: group.model_summary ? "Model evaluation summary available." : "Model evaluation status for this group.",
  };
}

function faceResult(assessment: SubjectAssessment) {
  const signals = subjectSignals(assessment);
  const faceCount = signals.face_count ?? 0;
  return {
    label: "浜鸿劯",
    value: technicalGateStatusLabel(assessment.gate_status),
    status: assessment.gate_status,
    note: faceCount ? `妫€娴嬪埌 ${faceCount} 寮犱汉鑴革紝椋庨櫓浼氬湪鍥剧墖涓婄敤缁嗙孩妗嗘爣鍑恒€俙` : "鏈娴嬪埌浜鸿劯銆?",
  };
}

function technicalStatusNote(token: string) {
  if (["pass", "ready", "completed"].includes(token)) return "鏈湴璐ㄩ噺闂ㄦ帶閫氳繃銆?";
  if (["warn", "inconclusive"].includes(token)) return "鏈夎川閲忛闄╋紝寤鸿浜哄伐澶嶆牳銆?";
  if (["reject", "failed"].includes(token)) return "璐ㄩ噺闂ㄦ帶涓嶅缓璁叆閫夈€?";
  if (token === "unsupported") return "姝ゆ牸寮忔殏涓嶆敮鎸佹湰鍦拌川閲忔鏌ャ€?";
  return "杩樻病鏈夎繍琛岃川閲忔鏌ャ€?";
}
