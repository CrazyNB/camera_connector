import test from "node:test";
import assert from "node:assert/strict";

import {
  isBrowserPreviewFormat,
  isPreviewableFormat,
  shouldRequestOriginalPreview,
  shouldRequestFullPreview,
  supportsFullThumbnailFormat,
} from "../src/mediaPreview.js";

test("RAW formats can request fast and full backend previews", () => {
  assert.equal(isPreviewableFormat("NEF"), true);
  assert.equal(supportsFullThumbnailFormat("NEF"), true);
  assert.equal(isBrowserPreviewFormat("NEF"), false);
  assert.equal(shouldRequestFullPreview("NEF", false), false);
  assert.equal(shouldRequestFullPreview("NEF", true), true);
  assert.equal(shouldRequestOriginalPreview("NEF"), true);
});

test("standard raster formats support fast, full, and browser previews", () => {
  assert.equal(isPreviewableFormat("jpg"), true);
  assert.equal(supportsFullThumbnailFormat("jpg"), true);
  assert.equal(isBrowserPreviewFormat("jpg"), true);
  assert.equal(shouldRequestFullPreview("jpg", false), true);
  assert.equal(shouldRequestFullPreview("jpg", true), true);
  assert.equal(shouldRequestOriginalPreview("jpg"), false);
});
