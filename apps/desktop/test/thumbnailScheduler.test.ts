import test from "node:test";
import assert from "node:assert/strict";

import { fullThumbnailConcurrency } from "../src/thumbnailScheduler.js";

test("fullThumbnailConcurrency uses five slots while scrolling", () => {
  assert.equal(fullThumbnailConcurrency(true), 5);
});

test("fullThumbnailConcurrency uses ten slots after scrolling settles", () => {
  assert.equal(fullThumbnailConcurrency(false), 10);
});
