import { scanStartBlocker, type ScanStartBlocker } from "./workflow";
import { intelligenceSetupState } from "./intelligence";
import { state } from "./appState";
import { errorMessage } from "./presentation";

export function createWorkflowStatusController(render: () => void) {
  function lanSyncTransferDot() {
    switch (state.lanSyncPhase) {
      case "done":
        return "available";
      case "discovering":
      case "syncing":
        return "changed";
      case "failed":
        return "missing";
      default:
        return "neutral";
    }
  }

  function lanSyncTransferLabel() {
    if (state.lanSyncPhase === "discovering") return "discovering";
    if (state.lanSyncPhase === "syncing") return "matching";
    if (state.lanSyncPhase === "failed") return "failed";
    const summary = state.lanSyncSummary;
    if (!summary) return state.lanSyncSources.length ? `${state.lanSyncSources.length} source` : "no source";
    const applied =
      summary.applied_user_marks +
      summary.applied_model_evaluations +
      summary.applied_selection_recommendations;
    return `${summary.matched_groups} matched / ${applied} applied / ${summary.unresolved_records} unresolved`;
  }

  function canStartScan() {
    return getScanStartBlocker() === null;
  }

  function getScanStartBlocker() {
    return scanStartBlocker({
      hasProject: Boolean(state.selectedProjectId),
      hasRootPath: Boolean(state.rootPath),
      busy: Boolean(state.busy),
      scanPhase: state.scan?.phase ?? null,
    });
  }

  function scanBlockerCopy(blocker: ScanStartBlocker) {
    switch (blocker) {
      case "project":
        return "鍏堝垱寤烘垨閫夋嫨涓€涓」鐩€?";
      case "folder":
        return "鍏堜负椤圭洰缁戝畾鐓х墖鏂囦欢澶广€?";
      case "busy":
        return `姝ｅ湪澶勭悊 ${state.busy}锛岀◢鍚庡啀璇曘€俙`;
      case "active_scan":
        return "褰撳墠椤圭洰姝ｅ湪鎵弿銆?";
    }
  }

  function setStatus(message: string, error: string | null = null) {
    state.status = message;
    state.error = error;
    render();
  }

  async function withBusy<T>(label: string, task: () => Promise<T>): Promise<T | null> {
    state.busy = label;
    state.error = null;
    render();
    try {
      const result = await task();
      state.busy = null;
      render();
      return result;
    } catch (error) {
      state.busy = null;
      setStatus(label, errorMessage(error));
      return null;
    }
  }

  function currentIntelligenceSetup() {
    return intelligenceSetupState(state.intelligenceProviders, state.promptPacks, state.intelligenceSettings);
  }

  return {
    lanSyncTransferDot,
    lanSyncTransferLabel,
    canStartScan,
    getScanStartBlocker,
    scanBlockerCopy,
    setStatus,
    withBusy,
    currentIntelligenceSetup,
  };
}
