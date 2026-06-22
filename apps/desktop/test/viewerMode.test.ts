import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join, normalize } from "node:path";

import {
  adjacentViewerGroup,
  dragViewerTransform,
  resetViewerTransform,
  shouldPreserveViewerTransformForSelection,
  toggleViewerDoubleClickZoom,
  viewerBurstWarmWindow,
  viewerCarryoverSource,
  viewerCurrentGroup,
  viewerReplacementAfterDelete,
  viewerQueueWindow,
  zoomViewerTransformAtPoint,
} from "../src/viewerMode.js";

type TestGroup = {
  group_id?: string | null;
  group_key: string;
};

const groups: TestGroup[] = Array.from({ length: 8 }, (_, index) => ({
  group_id: `group-${index + 1}`,
  group_key: `DSC_${String(index + 1).padStart(4, "0")}`,
}));

const burstGroups = [
  { group_id: "burst-a-1", group_key: "DSC_1001", burst: { burst_group_id: "burst-a" } },
  { group_id: "burst-a-2", group_key: "DSC_1002", burst: { burst_group_id: "burst-a" } },
  { group_id: "burst-b-1", group_key: "DSC_2001", burst: { burst_group_id: "burst-b" } },
  { group_id: "single", group_key: "DSC_9001", burst: null },
];

function readCssWithImports(path: string, visited = new Set<string>()): string {
  const normalizedPath = normalize(path);
  if (visited.has(normalizedPath)) return "";
  visited.add(normalizedPath);

  return readFileSync(normalizedPath, "utf8").replace(/^@import\s+"([^"]+)";/gm, (_match, importPath: string) =>
    readCssWithImports(join(dirname(normalizedPath), importPath), visited),
  );
}

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

test("viewerReplacementAfterDelete keeps the viewer near the deleted group", () => {
  assert.equal(viewerReplacementAfterDelete(groups, groups[3])?.group_id, "group-5");
  assert.equal(viewerReplacementAfterDelete(groups, groups[7])?.group_id, "group-7");
  assert.equal(viewerReplacementAfterDelete([groups[0]], groups[0]), null);
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

test("shouldPreserveViewerTransformForSelection only preserves inside the same burst", () => {
  assert.equal(shouldPreserveViewerTransformForSelection(burstGroups[0], burstGroups[1], true), true);
  assert.equal(shouldPreserveViewerTransformForSelection(burstGroups[0], burstGroups[2], true), false);
  assert.equal(shouldPreserveViewerTransformForSelection(burstGroups[0], burstGroups[1], false), false);
  assert.equal(shouldPreserveViewerTransformForSelection(burstGroups[0], burstGroups[3], true), false);
  assert.equal(shouldPreserveViewerTransformForSelection(null, burstGroups[1], true), false);
});

test("viewerBurstWarmWindow includes same-burst members with the current frame first", () => {
  assert.deepEqual(
    viewerBurstWarmWindow(burstGroups, burstGroups[1]).map((group) => group.group_id),
    ["burst-a-2", "burst-a-1"],
  );
  assert.deepEqual(viewerBurstWarmWindow(burstGroups, burstGroups[3]), []);
});

test("viewerCarryoverSource keeps the visible decoded frame during rapid burst switching", () => {
  assert.equal(
    viewerCarryoverSource([
      { url: "next-frame.jpg", loaded: false, role: "preview" },
      { url: "previous-visible-frame.jpg", loaded: true, role: "carryover" },
      { url: "older-preview.jpg", loaded: true, role: "preview" },
    ]),
    "previous-visible-frame.jpg",
  );
});

test("viewerCarryoverSource skips unloaded preview images", () => {
  assert.equal(
    viewerCarryoverSource([
      { url: "unloaded-target.jpg", loaded: false, role: "preview" },
      { url: "decoded-current.jpg", loaded: true, role: "preview" },
    ]),
    "decoded-current.jpg",
  );
  assert.equal(viewerCarryoverSource([{ url: "unloaded-target.jpg", loaded: false, role: "preview" }]), null);
});

test("viewerCarryoverSource falls back to the last retained visible frame", () => {
  assert.equal(
    viewerCarryoverSource([{ url: "unloaded-target.jpg", loaded: false, role: "preview" }], "retained-visible.jpg"),
    "retained-visible.jpg",
  );
});

test("zoomViewerTransformAtPoint keeps the hovered image point fixed", () => {
  const next = zoomViewerTransformAtPoint({ zoom: 1, panX: 0, panY: 0 }, { x: 300, y: 120 }, 2);

  assert.deepEqual(next, { zoom: 2, panX: -300, panY: -120 });
});

test("zoomViewerTransformAtPoint clamps zoom and preserves existing pan", () => {
  const next = zoomViewerTransformAtPoint({ zoom: 2, panX: -120, panY: -80 }, { x: 300, y: 200 }, 100);

  assert.deepEqual(next, { zoom: 8, panX: -1380, panY: -920 });
});

test("toggleViewerDoubleClickZoom zooms to 4x around the clicked image point", () => {
  const next = toggleViewerDoubleClickZoom({ zoom: 1, panX: 0, panY: 0 }, { x: 300, y: 120 });

  assert.deepEqual(next, { zoom: 4, panX: -900, panY: -360 });
});

test("toggleViewerDoubleClickZoom resets when the viewer is already zoomed", () => {
  const next = toggleViewerDoubleClickZoom({ zoom: 3, panX: -220, panY: -90 }, { x: 300, y: 120 });

  assert.deepEqual(next, { zoom: 1, panX: 0, panY: 0 });
});

test("dragViewerTransform pans only while zoomed", () => {
  assert.deepEqual(dragViewerTransform({ zoom: 1, panX: 0, panY: 0 }, { x: 80, y: -40 }), {
    zoom: 1,
    panX: 0,
    panY: 0,
  });
  assert.deepEqual(dragViewerTransform({ zoom: 2, panX: -20, panY: 10 }, { x: 80, y: -40 }), {
    zoom: 2,
    panX: 60,
    panY: -30,
  });
});

test("resetViewerTransform restores the unzoomed viewer", () => {
  assert.deepEqual(resetViewerTransform(), { zoom: 1, panX: 0, panY: 0 });
});

test("viewer navigation keeps its centered transform while pressed", () => {
  const css = readCssWithImports("src/styles.css");

  assert.match(css, /\.viewer-nav:active:not\(:disabled\)\s*{[^}]*transform:\s*translateY\(-50%\)/s);
});
