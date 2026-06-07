export type WorkbenchStage = "project" | "folder" | "scan" | "review";

export type WorkbenchStageInput = {
  hasProject: boolean;
  hasRootPath: boolean;
  scanPhase: string | null;
  groupCount: number;
};

export function deriveWorkbenchStage(input: WorkbenchStageInput): WorkbenchStage {
  if (!input.hasProject) {
    return "project";
  }
  if (!input.hasRootPath) {
    return "folder";
  }
  if (input.scanPhase && ["queued", "scanning", "indexing"].includes(input.scanPhase)) {
    return "scan";
  }
  if (input.groupCount > 0) {
    return "review";
  }
  return "scan";
}
