export type LanProjectSnapshotSource = {
  device_label: string;
  platform: string;
  project_name: string;
  snapshot_url: string;
  base_url: string;
};

export function selectLanProjectSnapshotSource(
  sources: LanProjectSnapshotSource[],
  activeProjectName: string | null | undefined,
): LanProjectSnapshotSource | null {
  if (!sources.length) return null;
  const normalizedProjectName = normalizeProjectName(activeProjectName);
  if (normalizedProjectName) {
    const matchingSource = sources.find(
      (source) => normalizeProjectName(source.project_name) === normalizedProjectName,
    );
    if (matchingSource) return matchingSource;
  }
  return sources[0];
}

function normalizeProjectName(value: string | null | undefined): string {
  return (value ?? "").trim().replace(/\s+/g, " ").toLocaleLowerCase();
}
