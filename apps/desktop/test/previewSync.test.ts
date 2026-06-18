import test from "node:test";
import assert from "node:assert/strict";

import { shouldApplyPreviewSync } from "../src/previewSync.js";

test("shouldApplyPreviewSync applies a completed preview to the matching live node", () => {
  assert.equal(
    shouldApplyPreviewSync(
      { previewPath: "photo.nef", previewMaxEdge: "1280" },
      { localPath: "photo.nef", maxEdge: "1280", quality: "fast", url: "asset://fast.jpg" },
    ),
    true,
  );
});

test("shouldApplyPreviewSync ignores stale nodes and empty urls", () => {
  assert.equal(
    shouldApplyPreviewSync(
      { previewPath: "other.nef", previewMaxEdge: "1280" },
      { localPath: "photo.nef", maxEdge: "1280", quality: "fast", url: "asset://fast.jpg" },
    ),
    false,
  );
  assert.equal(
    shouldApplyPreviewSync(
      { previewPath: "photo.nef", previewMaxEdge: "1280" },
      { localPath: "photo.nef", maxEdge: "1280", quality: "fast", url: null },
    ),
    false,
  );
});

test("shouldApplyPreviewSync does not downgrade an already better preview", () => {
  assert.equal(
    shouldApplyPreviewSync(
      { previewPath: "photo.nef", previewMaxEdge: "1280", currentQuality: "full" },
      { localPath: "photo.nef", maxEdge: "1280", quality: "fast", url: "asset://fast.jpg" },
    ),
    false,
  );
  assert.equal(
    shouldApplyPreviewSync(
      { previewPath: "photo.nef", previewMaxEdge: "original", currentQuality: "original" },
      { localPath: "photo.nef", maxEdge: "original", quality: "full", url: "asset://full.jpg" },
    ),
    false,
  );
});

test("shouldApplyPreviewSync respects the pending full thumbnail generation key", () => {
  assert.equal(
    shouldApplyPreviewSync(
      { previewPath: "photo.nef", previewMaxEdge: "1280", previewFullPending: "full:960:photo.nef" },
      { localPath: "photo.nef", maxEdge: "1280", quality: "full", url: "asset://full.jpg" },
    ),
    false,
  );
  assert.equal(
    shouldApplyPreviewSync(
      { previewPath: "photo.nef", previewMaxEdge: "1280", previewFullPending: "full:1280:photo.nef" },
      { localPath: "photo.nef", maxEdge: "1280", quality: "full", url: "asset://full.jpg" },
    ),
    true,
  );
});
