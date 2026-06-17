export type ViewerGroup = {
  group_id?: string | null;
  group_key: string;
};

export function viewerCurrentGroup<T extends ViewerGroup>(groups: T[], selectedGroupId: string | null | undefined) {
  if (!groups.length) return null;
  if (!selectedGroupId) return groups[0];
  return groups.find((group) => viewerGroupIdentity(group) === selectedGroupId) ?? groups[0];
}

export function adjacentViewerGroup<T extends ViewerGroup>(groups: T[], currentGroup: T | null, direction: number) {
  if (!groups.length) return null;
  if (!currentGroup) return groups[0];
  const currentIndex = viewerGroupIndex(groups, currentGroup);
  const safeIndex = currentIndex >= 0 ? currentIndex : 0;
  return groups[(safeIndex + direction + groups.length) % groups.length];
}

export function viewerQueueWindow<T extends ViewerGroup>(groups: T[], currentGroup: T | null, radius = 7) {
  if (!groups.length) return [];
  const windowSize = Math.min(groups.length, radius * 2 + 1);
  const currentIndex = currentGroup ? viewerGroupIndex(groups, currentGroup) : 0;
  const safeIndex = currentIndex >= 0 ? currentIndex : 0;
  const start = Math.min(Math.max(0, safeIndex - radius), Math.max(0, groups.length - windowSize));
  return groups.slice(start, start + windowSize);
}

export function viewerGroupIndex<T extends ViewerGroup>(groups: T[], currentGroup: T) {
  const currentId = viewerGroupIdentity(currentGroup);
  return groups.findIndex((group) => viewerGroupIdentity(group) === currentId);
}

export function viewerGroupIdentity(group: ViewerGroup) {
  return group.group_id ?? group.group_key;
}
