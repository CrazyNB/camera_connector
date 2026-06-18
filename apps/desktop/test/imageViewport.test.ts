import test from "node:test";
import assert from "node:assert/strict";

import { containedImageRect, normalizedContainedImagePoint } from "../src/imageViewport.js";

test("containedImageRect fits a wide image inside a tall container", () => {
  assert.deepEqual(
    containedImageRect({ left: 10, top: 20, width: 400, height: 400 }, { width: 1600, height: 900 }),
    { left: 10, top: 107.5, width: 400, height: 225 },
  );
});

test("containedImageRect fits a portrait image inside a wide container", () => {
  assert.deepEqual(
    containedImageRect({ left: 0, top: 0, width: 500, height: 300 }, { width: 800, height: 1200 }),
    { left: 150, top: 0, width: 200, height: 300 },
  );
});

test("normalizedContainedImagePoint reports black-bar hits outside the image", () => {
  const result = normalizedContainedImagePoint(
    { left: 10, top: 20, width: 400, height: 400 },
    { width: 1600, height: 900 },
    { x: 210, y: 40 },
  );

  assert.equal(result.inside, false);
  assert.equal(result.x, 0.5);
  assert.equal(result.y, 0);
});

test("normalizedContainedImagePoint preserves edge coordinates inside the image", () => {
  const result = normalizedContainedImagePoint(
    { left: 10, top: 20, width: 400, height: 400 },
    { width: 1600, height: 900 },
    { x: 410, y: 332.5 },
  );

  assert.equal(result.inside, true);
  assert.equal(result.x, 1);
  assert.equal(result.y, 1);
});
