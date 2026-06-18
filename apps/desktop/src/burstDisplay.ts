type BurstDisplaySummary = {
  burst_group_id: string;
  best_asset_group_id?: string | null;
};

export type BurstDisplayGroup = {
  group_id?: string | null;
  group_key: string;
  burst?: BurstDisplaySummary | null;
  is_model_select?: boolean;
  user_marks?: {
    favorite?: boolean;
    marked?: boolean;
  };
};

export function collapseBurstGroups<T extends BurstDisplayGroup>(
  groups: T[],
  selectedGroupId: string | null = null,
): T[] {
  const collapsed: T[] = [];
  const burstSlots = new Map<string, number>();

  for (const group of groups) {
    const burstId = group.burst?.burst_group_id;
    if (!burstId) {
      collapsed.push(group);
      continue;
    }

    const slot = burstSlots.get(burstId);
    if (slot === undefined) {
      burstSlots.set(burstId, collapsed.length);
      collapsed.push(group);
      continue;
    }

    if (rankBurstRepresentative(group, selectedGroupId) > rankBurstRepresentative(collapsed[slot], selectedGroupId)) {
      collapsed[slot] = group;
    }
  }

  return collapsed;
}

function rankBurstRepresentative(group: BurstDisplayGroup, selectedGroupId: string | null) {
  if (selectedGroupId && groupIdentity(group) === selectedGroupId) return 5;
  if (group.burst?.best_asset_group_id && groupIdentity(group) === group.burst.best_asset_group_id) return 4;
  if (group.is_model_select) return 3;
  if (group.user_marks?.favorite) return 2;
  if (group.user_marks?.marked) return 1;
  return 0;
}

function groupIdentity(group: BurstDisplayGroup) {
  return group.group_id ?? group.group_key;
}
