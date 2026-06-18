export const FULL_THUMBNAIL_SCROLLING_CONCURRENCY = 5;
export const FULL_THUMBNAIL_IDLE_CONCURRENCY = 10;

export function fullThumbnailConcurrency(isScrolling: boolean) {
  return isScrolling ? FULL_THUMBNAIL_SCROLLING_CONCURRENCY : FULL_THUMBNAIL_IDLE_CONCURRENCY;
}
