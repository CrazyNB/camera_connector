import type { ReceivedAssetGroup, SourceFilter } from "./appTypes";
import { state, GRID_GAP, THUMBNAIL_PREFETCH_ROWS, THUMBNAIL_SCROLL_IDLE_MS, VIRTUAL_OVERSCAN_ROWS } from "./appState";
import { append, commandButton, el } from "./domHelpers";
import { clamp, sourceStatus } from "./presentation";
import { pumpThumbnailQueue } from "./previewQueue";
import { resetViewerTransform } from "./viewerMode";
import { visibleGridWindow, type VisibleGridWindow } from "./virtualGrid";

let virtualBoardFrame: number | null = null;
let lastVirtualSignature = "";
let thumbnailScrollIdleTimer: number | null = null;
let thumbnailScrolling = false;

export type LightTableOptions = {
  allGroups: () => ReceivedAssetGroup[];
  displayGroupsFor: (groups: ReceivedAssetGroup[]) => ReceivedAssetGroup[];
  filteredGroups: () => ReceivedAssetGroup[];
  render: () => void;
  resetBoardViewport: () => void;
  clearViewerDrag: () => void;
  renderWorkbenchEmptyState: () => HTMLElement;
  renderGroupCard: (group: ReceivedAssetGroup, gridColumns?: number) => HTMLElement;
  loadMoreAssetGroups: () => Promise<void>;
  warmThumbnailsForGroups: (groups: ReceivedAssetGroup[]) => void;
};

export function getThumbnailScrolling() {
  return thumbnailScrolling;
}

export function resetLightTableVirtualSignature() {
  lastVirtualSignature = "";
}
export function handleLightTableWheel(event: WheelEvent, options: LightTableOptions) {
  const target = event.target as HTMLElement | null;
  if (target?.closest("button, input, select, textarea, .size-control")) {
    return;
  }
  const board = (event.currentTarget as HTMLElement).querySelector<HTMLElement>(".group-board");
  if (!board) {
    return;
  }

  const deltaY = normalizedWheelDelta(event.deltaY, event.deltaMode, board.clientHeight);
  const deltaX = normalizedWheelDelta(event.deltaX, event.deltaMode, board.clientWidth);
  const maxTop = Math.max(0, board.scrollHeight - board.clientHeight);
  const maxLeft = Math.max(0, board.scrollWidth - board.clientWidth);
  const nextTop = clamp(board.scrollTop + deltaY, 0, maxTop);
  const nextLeft = clamp(board.scrollLeft + deltaX, 0, maxLeft);

  if (nextTop !== board.scrollTop || nextLeft !== board.scrollLeft) {
    board.scrollTop = nextTop;
    board.scrollLeft = nextLeft;
    handleGroupBoardScroll(board, options);
    event.preventDefault();
  }
}

function normalizedWheelDelta(delta: number, mode: number, pageSize: number) {
  if (mode === WheelEvent.DOM_DELTA_LINE) return delta * 18;
  if (mode === WheelEvent.DOM_DELTA_PAGE) return delta * pageSize;
  return delta;
}

export function renderLightTableToolbar(options: LightTableOptions) {
  const toolbar = el("div", "light-table-toolbar");
  append(
    toolbar,
    renderSourceTabs(options),
    append(
      el("div", "table-controls"),
      el("span", "", "鎺掑簭锛氭媿鎽勬椂闂?"),
      commandButton("缃戞牸", `tool-toggle${state.layoutMode === "grid" ? " is-active" : ""}`, () => {
        state.layoutMode = "grid";
        state.loupe = null;
        options.render();
      }),
      commandButton("Viewer", `tool-toggle${state.layoutMode === "viewer" ? " is-active" : ""}`, () => {
        state.layoutMode = "viewer";
        state.loupe = null;
        state.viewerTransform = resetViewerTransform();
        options.clearViewerDrag();
        options.render();
      }),
      state.layoutMode === "grid"
        ? append(el("label", "size-control"), el("span", "", "灏哄"), renderSizeRange(options))
        : null,
    ),
  );
  return toolbar;
}

function renderSizeRange(options: LightTableOptions) {
  const input = el("input", "size-range") as HTMLInputElement;
  input.type = "range";
  input.min = "220";
  input.max = "420";
  input.step = "10";
  input.value = String(state.thumbSize);
  input.addEventListener("input", () => {
    state.thumbSize = Number(input.value);
    lastVirtualSignature = "";
    const board = document.querySelector<HTMLElement>(".group-board");
    if (board) {
      board.style.setProperty("--thumb-size", `${state.thumbSize}px`);
      updateVirtualBoard(board, options);
    }
  });
  return input;
}

function renderSourceTabs(options: LightTableOptions) {
  const tabs = el("div", "source-tabs");
  const filters: Array<[SourceFilter, string]> = [
    ["all", "鍏ㄩ儴"],
    ["available", "鍙敤"],
    ["changed", "宸插彉鍖?"],
    ["missing", "缂哄け"],
  ];
  for (const [filter, label] of filters) {
    const count =
      filter === "all"
        ? options.displayGroupsFor(options.allGroups()).length
        : options.displayGroupsFor(options.allGroups().filter((group) => sourceStatus(group) === filter)).length;
    const tab = commandButton(`${label} ${count}`, "source-tab", () => {
      state.sourceFilter = filter;
      state.selectedGroupId = null;
      state.selectedGroup = null;
      state.groupDetail = [];
      options.resetBoardViewport();
      options.render();
    });
    if (state.sourceFilter === filter) {
      tab.classList.add("is-active");
    }
    append(tabs, tab);
  }
  return tabs;
}

export function renderGroupBoard(options: LightTableOptions) {
  const groups = options.filteredGroups();
  const board = el("div", "group-board");
  board.style.setProperty("--thumb-size", `${state.thumbSize}px`);
  if (!groups.length) {
    append(board, options.renderWorkbenchEmptyState());
    return board;
  }
  board.addEventListener("scroll", () => handleGroupBoardScroll(board, options), { passive: true });
  const spacer = el("div", "virtual-board-spacer");
  const windowNode = el("div", "virtual-board-window");
  append(spacer, windowNode);
  append(board, spacer);
  renderVirtualWindow(board, groups, virtualMetricsForBoard(board, groups.length), options);
  requestAnimationFrame(() => {
    if (!board.isConnected) return;
    board.scrollTop = state.boardScrollTop;
    state.boardWidth = board.clientWidth;
    lastVirtualSignature = "";
    updateVirtualBoard(board, options);
  });
  return board;
}

function handleGroupBoardScroll(board: HTMLElement, options: LightTableOptions) {
  state.boardScrollTop = board.scrollTop;
  state.boardWidth = board.clientWidth;
  markThumbnailScrolling();
  if (virtualBoardFrame !== null) {
    return;
  }
  virtualBoardFrame = requestAnimationFrame(() => {
    virtualBoardFrame = null;
    if (board.isConnected) {
      updateVirtualBoard(board, options);
    }
  });
}

function markThumbnailScrolling() {
  thumbnailScrolling = true;
  if (thumbnailScrollIdleTimer !== null) {
    window.clearTimeout(thumbnailScrollIdleTimer);
  }
  thumbnailScrollIdleTimer = window.setTimeout(() => {
    thumbnailScrolling = false;
    thumbnailScrollIdleTimer = null;
    pumpThumbnailQueue();
  }, THUMBNAIL_SCROLL_IDLE_MS);
}

export function updateActiveVirtualBoard(options: LightTableOptions) {
  const board = document.querySelector<HTMLElement>(".group-board");
  if (board) {
    lastVirtualSignature = "";
    updateVirtualBoard(board, options);
  }
}

function updateVirtualBoard(board: HTMLElement, options: LightTableOptions) {
  const groups = options.filteredGroups();
  if (!groups.length) {
    board.replaceChildren(options.renderWorkbenchEmptyState());
    return;
  }
  if (!board.querySelector(".virtual-board-spacer")) {
    const spacer = el("div", "virtual-board-spacer");
    append(spacer, el("div", "virtual-board-window"));
    board.replaceChildren(spacer);
  }
  const metrics = virtualMetricsForBoard(board, groups.length);
  renderVirtualWindow(board, groups, metrics, options);
  if (shouldLoadMoreGroups(board, metrics)) {
    void options.loadMoreAssetGroups();
  }
}

function renderVirtualWindow(board: HTMLElement, groups: ReceivedAssetGroup[], metrics: VisibleGridWindow, options: LightTableOptions) {
  const spacer = board.querySelector<HTMLElement>(".virtual-board-spacer");
  const windowNode = board.querySelector<HTMLElement>(".virtual-board-window");
  if (!spacer || !windowNode) {
    return;
  }
  spacer.style.height = `${metrics.totalHeight}px`;
  windowNode.style.transform = `translate3d(0, ${metrics.offsetY}px, 0)`;
  windowNode.style.gridTemplateColumns = `repeat(${metrics.columns}, minmax(0, var(--thumb-size, 320px)))`;
  const signature = [
    metrics.startIndex,
    metrics.endIndex,
    metrics.columns,
    groups.length,
    state.thumbSize,
    state.selectedGroupId ?? "",
    state.sourceFilter,
    state.viewFilter,
  ].join(":");
  if (signature === lastVirtualSignature && windowNode.childElementCount) {
    prefetchThumbnailsAroundWindow(groups, metrics, options);
    return;
  }
  lastVirtualSignature = signature;
  windowNode.replaceChildren(...groups.slice(metrics.startIndex, metrics.endIndex).map((group) => options.renderGroupCard(group, metrics.columns)));
  prefetchThumbnailsAroundWindow(groups, metrics, options);
}

function virtualMetricsForBoard(board: HTMLElement, totalItems: number) {
  const viewportWidth = Math.max(1, board.clientWidth || state.boardWidth || window.innerWidth - 320);
  const viewportHeight = Math.max(1, board.clientHeight || window.innerHeight - 210);
  return visibleGridWindow({
    totalItems,
    viewportWidth,
    viewportHeight,
    scrollTop: board.scrollTop || state.boardScrollTop,
    itemWidth: state.thumbSize,
    rowHeight: estimatedGroupRowHeight(),
    gap: GRID_GAP,
    overscanRows: VIRTUAL_OVERSCAN_ROWS,
  });
}

function estimatedGroupRowHeight() {
  return Math.round(state.thumbSize * 0.75) + 96;
}

function shouldLoadMoreGroups(board: HTMLElement, metrics: VisibleGridWindow) {
  const page = state.assetPage;
  if (!page?.has_more || state.assetPageLoading || state.sourceFilter !== "all" || state.viewFilter !== "light-table") {
    return false;
  }
  return board.scrollTop + board.clientHeight >= metrics.totalHeight - metrics.rowHeight * 4;
}

function prefetchThumbnailsAroundWindow(groups: ReceivedAssetGroup[], metrics: VisibleGridWindow, options: LightTableOptions) {
  const preloadCount = Math.max(metrics.columns, metrics.columns * THUMBNAIL_PREFETCH_ROWS);
  const endIndex = Math.min(groups.length, metrics.endIndex + preloadCount);
  options.warmThumbnailsForGroups(groups.slice(metrics.endIndex, endIndex));
}
