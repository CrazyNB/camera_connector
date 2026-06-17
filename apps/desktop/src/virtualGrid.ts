export type VisibleGridWindowInput = {
  totalItems: number;
  viewportWidth: number;
  viewportHeight: number;
  scrollTop: number;
  itemWidth: number;
  rowHeight: number;
  gap: number;
  overscanRows: number;
};

export type VisibleGridWindow = {
  columns: number;
  rowHeight: number;
  totalRows: number;
  totalHeight: number;
  startIndex: number;
  endIndex: number;
  offsetY: number;
  itemsInDom: number;
};

export function visibleGridWindow(input: VisibleGridWindowInput): VisibleGridWindow {
  const totalItems = Math.max(0, Math.floor(input.totalItems));
  const itemWidth = Math.max(1, input.itemWidth);
  const rowHeight = Math.max(1, input.rowHeight);
  const gap = Math.max(0, input.gap);
  const columns = Math.max(1, Math.floor((Math.max(1, input.viewportWidth) + gap) / (itemWidth + gap)));
  const totalRows = Math.ceil(totalItems / columns);
  const totalHeight = totalRows * rowHeight;
  const maxScrollTop = Math.max(0, totalHeight - Math.max(1, input.viewportHeight));
  const scrollTop = clamp(input.scrollTop, 0, maxScrollTop);
  const overscanRows = Math.max(0, Math.floor(input.overscanRows));
  const firstVisibleRow = Math.floor(scrollTop / rowHeight);
  const firstRow = Math.max(0, firstVisibleRow - overscanRows);
  const visibleRows = Math.ceil(Math.max(1, input.viewportHeight) / rowHeight) + overscanRows * 2;
  const lastRow = Math.min(totalRows, firstRow + visibleRows);
  const startIndex = Math.min(totalItems, firstRow * columns);
  const endIndex = Math.min(totalItems, lastRow * columns);

  return {
    columns,
    rowHeight,
    totalRows,
    totalHeight,
    startIndex,
    endIndex,
    offsetY: firstRow * rowHeight,
    itemsInDom: Math.max(0, endIndex - startIndex),
  };
}

function clamp(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value));
}
