import assert from "node:assert/strict";
import test from "node:test";

import { expandedGridColumn, initialLoupeZoom, nextLoupeZoom, viewerActionScope } from "../src/lightTableLayout.js";

test("expandedGridColumn centers a two-column hero inside the grid", () => {
  assert.equal(expandedGridColumn(1), null);
  assert.equal(expandedGridColumn(2), null);
  assert.equal(expandedGridColumn(3), "1 / span 2");
  assert.equal(expandedGridColumn(4), "2 / span 2");
  assert.equal(expandedGridColumn(5), "2 / span 2");
  assert.equal(expandedGridColumn(6), "3 / span 2");
});

test("viewer quality and AI actions are scoped to the current group", () => {
  assert.equal(viewerActionScope("quality"), "group");
  assert.equal(viewerActionScope("ai"), "group");
});

test("global toolbar actions stay project scoped", () => {
  assert.equal(viewerActionScope("global-quality"), "project");
  assert.equal(viewerActionScope("global-recommend"), "project");
});

test("loupe starts at 4x and clamps between 4x and 16x", () => {
  assert.equal(initialLoupeZoom(null), 4);
  assert.equal(initialLoupeZoom(2), 4);
  assert.equal(initialLoupeZoom(18), 16);
  assert.equal(nextLoupeZoom(4, "out"), 4);
  assert.equal(nextLoupeZoom(15.8, "in"), 16);
});
