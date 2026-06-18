import assert from "node:assert/strict";
import test from "node:test";
import { scanStartBlocker, scanTransferDisplay } from "../src/workflow.js";

test("scanStartBlocker allows scanning after a folder is selected", () => {
  assert.equal(
    scanStartBlocker({
      hasProject: true,
      hasRootPath: true,
      busy: false,
      scanPhase: null,
    }),
    null,
  );
  assert.equal(
    scanStartBlocker({
      hasProject: true,
      hasRootPath: true,
      busy: false,
      scanPhase: "completed",
    }),
    null,
  );
  assert.equal(
    scanStartBlocker({
      hasProject: true,
      hasRootPath: true,
      busy: false,
      scanPhase: "scanning",
    }),
    "active_scan",
  );
});

test("scanTransferDisplay keeps current index visible after a failed rescan", () => {
  assert.deepEqual(
    scanTransferDisplay({
      scanPhase: "failed",
      scanFilesSeen: 0,
      scanAssetsIndexed: 0,
      scanGroupsUpdated: 0,
      scanError: "provider failed",
      indexedAssetCount: 56,
      indexedGroupCount: 28,
    }),
    {
      label: "已索引",
      health: "ready",
      files: 56,
      groups: 28,
      assets: 56,
      note: "上次重新扫描失败，现有索引仍可使用。",
    },
  );
});
