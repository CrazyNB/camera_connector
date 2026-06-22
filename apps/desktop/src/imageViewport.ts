export type ViewportRect = {
  left: number;
  top: number;
  width: number;
  height: number;
};

export type ImageSize = {
  width: number;
  height: number;
};

export type Point = {
  x: number;
  y: number;
};

export type NormalizedImagePoint = Point & {
  inside: boolean;
  imageRect: ViewportRect;
};

export function containedImageRect(container: ViewportRect, image: ImageSize): ViewportRect {
  const safeContainerWidth = Math.max(1, container.width);
  const safeContainerHeight = Math.max(1, container.height);
  const imageWidth = Math.max(1, image.width);
  const imageHeight = Math.max(1, image.height);
  const scale = Math.min(safeContainerWidth / imageWidth, safeContainerHeight / imageHeight);
  const width = imageWidth * scale;
  const height = imageHeight * scale;
  return {
    left: container.left + (safeContainerWidth - width) / 2,
    top: container.top + (safeContainerHeight - height) / 2,
    width,
    height,
  };
}

export function coverImageRect(container: ViewportRect, image: ImageSize): ViewportRect {
  const safeContainerWidth = Math.max(1, container.width);
  const safeContainerHeight = Math.max(1, container.height);
  const imageWidth = Math.max(1, image.width);
  const imageHeight = Math.max(1, image.height);
  const scale = Math.max(safeContainerWidth / imageWidth, safeContainerHeight / imageHeight);
  const width = imageWidth * scale;
  const height = imageHeight * scale;
  return {
    left: container.left + (safeContainerWidth - width) / 2,
    top: container.top + (safeContainerHeight - height) / 2,
    width,
    height,
  };
}

export function normalizedPointInRect(rect: ViewportRect, point: Point): Point {
  return {
    x: clamp((point.x - rect.left) / Math.max(1, rect.width), 0, 1),
    y: clamp((point.y - rect.top) / Math.max(1, rect.height), 0, 1),
  };
}

export function normalizedContainedImagePoint(
  container: ViewportRect,
  image: ImageSize,
  point: Point,
): NormalizedImagePoint {
  const imageRect = containedImageRect(container, image);
  const x = clamp((point.x - imageRect.left) / imageRect.width, 0, 1);
  const y = clamp((point.y - imageRect.top) / imageRect.height, 0, 1);
  const inside =
    point.x >= imageRect.left &&
    point.x <= imageRect.left + imageRect.width &&
    point.y >= imageRect.top &&
    point.y <= imageRect.top + imageRect.height;
  return { x, y, inside, imageRect };
}

function clamp(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value));
}
