import test from "node:test";
import assert from "node:assert/strict";

import { collapseBurstGroups } from "../src/burstDisplay.js";

type TestGroup = {
  group_id: string;
  group_key: string;
  burst?: {
    burst_group_id: string;
    best_asset_group_id?: string | null;
  } | null;
  is_model_select: boolean;
  user_marks: {
    favorite: boolean;
    marked: boolean;
  };
};

function group(groupId: string, burstId?: string, bestGroupId?: string | null): TestGroup {
  return {
    group_id: groupId,
    group_key: groupId,
    burst: burstId ? { burst_group_id: burstId, best_asset_group_id: bestGroupId } : null,
    is_model_select: false,
    user_marks: {
      favorite: false,
      marked: false,
    },
  };
}

test("collapseBurstGroups keeps one display card per burst", () => {
  const groups = [group("IMG_0001", "burst-a"), group("IMG_0002", "burst-a"), group("IMG_0100")];

  assert.deepEqual(
    collapseBurstGroups(groups).map((candidate) => candidate.group_id),
    ["IMG_0001", "IMG_0100"],
  );
});

test("collapseBurstGroups keeps the selected burst member visible", () => {
  const groups = [group("IMG_0001", "burst-a"), group("IMG_0002", "burst-a"), group("IMG_0100")];

  assert.deepEqual(
    collapseBurstGroups(groups, "IMG_0002").map((candidate) => candidate.group_id),
    ["IMG_0002", "IMG_0100"],
  );
});

test("collapseBurstGroups prefers the core best asset group", () => {
  const groups = [
    group("IMG_0001", "burst-a", "IMG_0003"),
    group("IMG_0002", "burst-a", "IMG_0003"),
    group("IMG_0003", "burst-a", "IMG_0003"),
  ];

  assert.deepEqual(
    collapseBurstGroups(groups).map((candidate) => candidate.group_id),
    ["IMG_0003"],
  );
});
