import assert from "node:assert/strict";
import test from "node:test";
import { visibleGridWindow } from "../src/virtualGrid.js";

test("visibleGridWindow renders only rows near the viewport", () => {
  const window = visibleGridWindow({
    totalItems: 1000,
    viewportWidth: 980,
    viewportHeight: 720,
    scrollTop: 2400,
    itemWidth: 300,
    rowHeight: 320,
    gap: 12,
    overscanRows: 2,
  });

  assert.equal(window.columns, 3);
  assert.equal(window.startIndex, 15);
  assert.equal(window.endIndex, 36);
  assert.equal(window.itemsInDom, 21);
  assert.equal(window.totalHeight, 106880);
  assert.equal(window.offsetY, 1600);
});

test("visibleGridWindow clamps scroll past the last row", () => {
  const window = visibleGridWindow({
    totalItems: 10,
    viewportWidth: 640,
    viewportHeight: 360,
    scrollTop: 50000,
    itemWidth: 300,
    rowHeight: 320,
    gap: 12,
    overscanRows: 1,
  });

  assert.equal(window.columns, 2);
  assert.equal(window.startIndex, 4);
  assert.equal(window.endIndex, 10);
  assert.equal(window.totalHeight, 1600);
  assert.equal(window.offsetY, 640);
});
