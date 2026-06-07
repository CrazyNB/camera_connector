import assert from "node:assert/strict";
import test from "node:test";
import { deriveWorkbenchStage } from "../src/workflow.js";

test("deriveWorkbenchStage guides the desktop review flow", () => {
  assert.equal(
    deriveWorkbenchStage({ hasProject: false, hasRootPath: false, scanPhase: null, groupCount: 0 }),
    "project",
  );
  assert.equal(
    deriveWorkbenchStage({ hasProject: true, hasRootPath: false, scanPhase: null, groupCount: 0 }),
    "folder",
  );
  assert.equal(
    deriveWorkbenchStage({ hasProject: true, hasRootPath: true, scanPhase: "scanning", groupCount: 0 }),
    "scan",
  );
  assert.equal(
    deriveWorkbenchStage({ hasProject: true, hasRootPath: true, scanPhase: "completed", groupCount: 2 }),
    "review",
  );
});
