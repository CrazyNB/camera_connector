import {
  adjacentBurstMember,
  allGroupsForState,
  burstMembersOf as burstMembersIn,
  displayGroupsForSelection,
  filteredGroupsForState,
  groupByIdentityIn,
  selectedProjectForState,
} from "./groupSelectors";
import type {
  AppState,
  ReceivedAssetGroup,
  SelectGroupOptions,
} from "./appTypes";

type SelectGroup = (group: ReceivedAssetGroup, options?: SelectGroupOptions) => void | Promise<void>;

export function createAppSelectors(state: AppState, selectGroup: SelectGroup) {
  function allGroups() {
    return allGroupsForState(state);
  }

  function selectedProject() {
    return selectedProjectForState(state);
  }

  function filteredGroups() {
    return filteredGroupsForState(state);
  }

  function displayGroupsFor(groups: ReceivedAssetGroup[]) {
    return displayGroupsForSelection(groups, state.selectedGroupId);
  }

  function burstMembersOf(group: ReceivedAssetGroup) {
    return burstMembersIn(allGroups(), group);
  }

  function selectAdjacentBurst(group: ReceivedAssetGroup, direction: number) {
    const next = adjacentBurstMember(allGroups(), group, direction);
    void selectGroup(next, { preserveViewerTransform: true });
  }

  function groupByIdentity(groupId: string) {
    return groupByIdentityIn(allGroups(), groupId);
  }

  return {
    allGroups,
    selectedProject,
    filteredGroups,
    displayGroupsFor,
    burstMembersOf,
    selectAdjacentBurst,
    groupByIdentity,
  };
}
