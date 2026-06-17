import test from "node:test";
import assert from "node:assert/strict";

import { adjacentViewerGroup, viewerCurrentGroup, viewerQueueWindow } from "../src/viewerMode.js";

type TestGroup = {
  group_id?: string | null;
  group_key: string;
};

const groups: TestGroup[] = Array.from({ length: 8 }, (_, index) => ({
  group_id: `group-${index + 1}`,
  group_key: `DSC_${String(index + 1).padStart(4, "0")}`,
}));

test("viewerCurrentGroup prefers the selected group", () => {
  assert.equal(viewerCurrentGroup(groups, "group-4")?.group_id, "group-4");
});

test("viewerCurrentGroup falls back to the first visible group", () => {
  assert.equal(viewerCurrentGroup(groups, "missing")?.group_id, "group-1");
  assert.equal(viewerCurrentGroup([], "group-1"), null);
});

test("adjacentViewerGroup wraps across the queue", () => {
  assert.equal(adjacentViewerGroup(groups, groups[0], -1)?.group_id, "group-8");
  assert.equal(adjacentViewerGroup(groups, groups[7], 1)?.group_id, "group-1");
});

test("viewerQueueWindow keeps the active group centered when possible", () => {
  assert.deepEqual(
    viewerQueueWindow(groups, groups[4], 2).map((group) => group.group_id),
    ["group-3", "group-4", "group-5", "group-6", "group-7"],
  );
});

test("viewerQueueWindow clamps at the list edges", () => {
  assert.deepEqual(
    viewerQueueWindow(groups, groups[0], 2).map((group) => group.group_id),
    ["group-1", "group-2", "group-3", "group-4", "group-5"],
  );
});
