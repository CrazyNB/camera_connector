import test from "node:test";
import assert from "node:assert/strict";

import { previewBadge, previewProgress } from "../src/previewStatus.js";

test("previewBadge distinguishes queued, embedded, high quality, and original states", () => {
  assert.deepEqual(previewBadge("queued"), {
    label: "排队",
    tone: "waiting",
    title: "预览已进入队列",
  });
  assert.deepEqual(previewBadge("fast"), {
    label: "低清",
    tone: "low",
    title: "已显示内嵌或快速预览，正在等待高清预览",
  });
  assert.deepEqual(previewBadge("full"), {
    label: "高清",
    tone: "high",
    title: "高清预览已就绪",
  });
  assert.deepEqual(previewBadge("original"), {
    label: "原图",
    tone: "original",
    title: "正在使用原图预览",
  });
});

test("previewProgress summarizes loaded and pending preview work", () => {
  assert.deepEqual(previewProgress(["fast", "full", "original", "loading", "queued", "failed"]), {
    total: 6,
    high: 2,
    low: 1,
    pending: 2,
    failed: 1,
    label: "高清 2/6，低清 1，待处理 2，失败 1",
  });
});
