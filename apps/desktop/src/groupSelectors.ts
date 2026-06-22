import { collapseBurstGroups } from "./burstDisplay";
import type { AppState, ReceivedAssetGroup } from "./appTypes";
import { cssToken, sourceStatus } from "./presentation";

export function selectedProjectForState(state: AppState) {
  return state.projects.find((project) => project.project_id === state.selectedProjectId) ?? null;
}

export function folderBasename(path: string) {
  return path.split(/[\\/]/).filter(Boolean).at(-1) ?? path;
}

export function allGroupsForState(state: AppState) {
  return state.assetPage?.groups ?? [];
}

export function filteredGroupsForState(state: AppState) {
  let groups = allGroupsForState(state);
  if (state.viewFilter === "needs-work") {
    groups = groups.filter(needsWork);
  } else if (state.viewFilter === "missing") {
    groups = groups.filter((group) => sourceStatus(group) === "missing");
  }
  if (state.sourceFilter !== "all") {
    groups = groups.filter((group) => sourceStatus(group) === state.sourceFilter);
  }
  return displayGroupsForSelection(groups, state.selectedGroupId);
}

export function displayGroupsForSelection(groups: ReceivedAssetGroup[], selectedGroupId: string | null) {
  return collapseBurstGroups(groups, selectedGroupId);
}

export function needsWork(group: ReceivedAssetGroup) {
  const technical = cssToken(group.technical_gate_status ?? group.technical_status ?? "pending");
  const model = cssToken(group.model_status ?? "pending");
  return (
    sourceStatus(group) !== "available" ||
    Boolean(group.technical_defects.length) ||
    ["pending", "technical-pending", "failed"].includes(technical) ||
    ["pending", "failed"].includes(model)
  );
}

export function burstMembersOf(groups: ReceivedAssetGroup[], group: ReceivedAssetGroup) {
  const burstId = group.burst?.burst_group_id;
  if (!burstId) return [group];
  const members = groups.filter((candidate) => candidate.burst?.burst_group_id === burstId);
  return members.length ? members : [group];
}

export function selectedBurstIndex(group: ReceivedAssetGroup, members: ReceivedAssetGroup[]) {
  const index = members.findIndex((member) => member.group_id === group.group_id);
  return index >= 0 ? index + 1 : 1;
}

export function adjacentBurstMember(groups: ReceivedAssetGroup[], group: ReceivedAssetGroup, direction: number) {
  const members = burstMembersOf(groups, group);
  const currentIndex = Math.max(0, members.findIndex((member) => member.group_id === group.group_id));
  return members[(currentIndex + direction + members.length) % members.length];
}

export function groupIdentity(group: ReceivedAssetGroup) {
  return group.group_id ?? group.group_key;
}

export function uniqueGroupsByIdentity(groups: ReceivedAssetGroup[]) {
  const seen = new Set<string>();
  return groups.filter((group) => {
    const identity = groupIdentity(group);
    if (seen.has(identity)) {
      return false;
    }
    seen.add(identity);
    return true;
  });
}

export function groupByIdentityIn(groups: ReceivedAssetGroup[], groupId: string) {
  return groups.find((group) => groupIdentity(group) === groupId) ?? null;
}
