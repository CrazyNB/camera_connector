const FULL_THUMBNAIL_FORMATS = new Set(["jpg", "jpeg", "png", "webp", "gif", "bmp"]);
const RAW_THUMBNAIL_FORMATS = new Set(["nef", "nrw", "cr2", "cr3", "arw", "raf", "rw2", "orf", "pef", "dng"]);

export function isPreviewableFormat(format: string) {
  const token = mediaFormatToken(format);
  return FULL_THUMBNAIL_FORMATS.has(token) || RAW_THUMBNAIL_FORMATS.has(token);
}

export function supportsFullThumbnailFormat(format: string) {
  const token = mediaFormatToken(format);
  return FULL_THUMBNAIL_FORMATS.has(token) || RAW_THUMBNAIL_FORMATS.has(token);
}

export function isBrowserPreviewFormat(format: string) {
  return FULL_THUMBNAIL_FORMATS.has(mediaFormatToken(format));
}

export function shouldRequestFullPreview(format: string, original: boolean) {
  const token = mediaFormatToken(format);
  return FULL_THUMBNAIL_FORMATS.has(token) || (original && RAW_THUMBNAIL_FORMATS.has(token));
}

export function shouldRequestOriginalPreview(format: string) {
  return RAW_THUMBNAIL_FORMATS.has(mediaFormatToken(format));
}

function mediaFormatToken(format: string) {
  return format.trim().toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-+|-+$/g, "");
}
