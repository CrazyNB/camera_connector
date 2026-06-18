import assert from "node:assert/strict";
import test from "node:test";
import { selectLanProjectSnapshotSource } from "../src/lanProjectSync.js";

test("selectLanProjectSnapshotSource prefers the source whose project name matches the active project", () => {
  const sources = [
    source("Android Field Kit", "Wrong Project", "http://phone-a/snapshot"),
    source("Android Spare", "Wedding Selects", "http://phone-b/snapshot"),
  ];

  assert.equal(
    selectLanProjectSnapshotSource(sources, "Wedding Selects")?.snapshot_url,
    "http://phone-b/snapshot",
  );
});

test("selectLanProjectSnapshotSource normalizes project names before matching", () => {
  const sources = [
    source("Android Field Kit", "  wedding selects  ", "http://phone-a/snapshot"),
    source("Android Spare", "Catalog", "http://phone-b/snapshot"),
  ];

  assert.equal(
    selectLanProjectSnapshotSource(sources, "Wedding Selects")?.snapshot_url,
    "http://phone-a/snapshot",
  );
});

test("selectLanProjectSnapshotSource falls back to the first source when no project matches", () => {
  const sources = [
    source("Android Field Kit", "Field Backup", "http://phone-a/snapshot"),
    source("Android Spare", "Catalog", "http://phone-b/snapshot"),
  ];

  assert.equal(
    selectLanProjectSnapshotSource(sources, "Wedding Selects")?.snapshot_url,
    "http://phone-a/snapshot",
  );
});

function source(deviceLabel: string, projectName: string, snapshotUrl: string) {
  return {
    device_label: deviceLabel,
    platform: "android",
    project_name: projectName,
    snapshot_url: snapshotUrl,
    base_url: snapshotUrl.replace("/snapshot", ""),
  };
}
