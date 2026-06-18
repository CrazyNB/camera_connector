export type ViewerGroup = {
  group_id?: string | null;
  group_key: string;
};

export type ViewerBurstGroup = ViewerGroup & {
  burst?: {
    burst_group_id?: string | null;
  } | null;
};

export type ViewerTransform = {
  zoom: number;
  panX: number;
  panY: number;
};

export type ViewerPoint = {
  x: number;
  y: number;
};

export type ViewerCarryoverCandidate = {
  url: string | null | undefined;
  loaded: boolean;
  role: "carryover" | "preview";
};

const VIEWER_MIN_ZOOM = 1;
const VIEWER_MAX_ZOOM = 8;

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

export function viewerBurstWarmWindow<T extends ViewerBurstGroup>(groups: T[], currentGroup: T | null) {
  const burstId = currentGroup?.burst?.burst_group_id;
  if (!burstId) return [];
  const members = groups.filter((group) => group.burst?.burst_group_id === burstId);
  if (!members.length) return [];
  const currentId = viewerGroupIdentity(currentGroup);
  return [
    ...members.filter((group) => viewerGroupIdentity(group) === currentId),
    ...members.filter((group) => viewerGroupIdentity(group) !== currentId),
  ];
}

export function viewerGroupIndex<T extends ViewerGroup>(groups: T[], currentGroup: T) {
  const currentId = viewerGroupIdentity(currentGroup);
  return groups.findIndex((group) => viewerGroupIdentity(group) === currentId);
}

export function viewerGroupIdentity(group: ViewerGroup) {
  return group.group_id ?? group.group_key;
}

export function viewerCarryoverSource(candidates: ViewerCarryoverCandidate[], fallbackUrl: string | null = null) {
  const loadedCarryover = candidates.find((candidate) => candidate.role === "carryover" && candidate.loaded && candidate.url);
  if (loadedCarryover?.url) {
    return loadedCarryover.url;
  }
  const loadedPreview = candidates.find((candidate) => candidate.role === "preview" && candidate.loaded && candidate.url);
  return loadedPreview?.url ?? fallbackUrl;
}

export function resetViewerTransform(): ViewerTransform {
  return { zoom: 1, panX: 0, panY: 0 };
}

export function shouldPreserveViewerTransformForSelection<T extends ViewerBurstGroup>(
  currentGroup: T | null | undefined,
  targetGroup: T,
  requested: boolean,
) {
  const currentBurstId = currentGroup?.burst?.burst_group_id;
  const targetBurstId = targetGroup.burst?.burst_group_id;
  return Boolean(requested && currentBurstId && targetBurstId && currentBurstId === targetBurstId);
}

export function zoomViewerTransformAtPoint(
  transform: ViewerTransform,
  imagePoint: ViewerPoint,
  nextZoom: number,
): ViewerTransform {
  const zoom = clamp(nextZoom, VIEWER_MIN_ZOOM, VIEWER_MAX_ZOOM);
  const currentZoom = Math.max(VIEWER_MIN_ZOOM, transform.zoom);
  if (zoom === VIEWER_MIN_ZOOM) {
    return resetViewerTransform();
  }
  const contentX = (imagePoint.x - transform.panX) / currentZoom;
  const contentY = (imagePoint.y - transform.panY) / currentZoom;
  return {
    zoom,
    panX: imagePoint.x - contentX * zoom,
    panY: imagePoint.y - contentY * zoom,
  };
}

export function toggleViewerDoubleClickZoom(
  transform: ViewerTransform,
  imagePoint: ViewerPoint,
  defaultZoom = 2,
): ViewerTransform {
  if (transform.zoom > VIEWER_MIN_ZOOM) {
    return resetViewerTransform();
  }
  return zoomViewerTransformAtPoint(transform, imagePoint, defaultZoom);
}

export function dragViewerTransform(transform: ViewerTransform, delta: ViewerPoint): ViewerTransform {
  if (transform.zoom <= VIEWER_MIN_ZOOM) {
    return resetViewerTransform();
  }
  return {
    ...transform,
    panX: transform.panX + delta.x,
    panY: transform.panY + delta.y,
  };
}

function clamp(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value));
}
