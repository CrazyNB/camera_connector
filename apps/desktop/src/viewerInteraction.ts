import type { ViewerTransform } from "./viewerMode";
import {
  dragViewerTransform,
  toggleViewerDoubleClickZoom,
  zoomViewerTransformAtPoint,
} from "./viewerMode";
import type { ViewerCarryoverImage } from "./appTypes";
import { containedImageRect, normalizedContainedImagePoint } from "./imageViewport";

type ViewerInteractionDeps = {
  getCarryoverImage: () => ViewerCarryoverImage | null;
  getTransform: () => ViewerTransform;
  setCarryoverImage: (image: ViewerCarryoverImage | null) => void;
  setTransform: (transform: ViewerTransform) => void;
};

export function createViewerInteraction(deps: ViewerInteractionDeps) {
  let dragState: { x: number; y: number } | null = null;

  function appendCarryoverImage(preview: HTMLElement) {
    const carryover = deps.getCarryoverImage();
    if (!carryover?.url) {
      return;
    }
    const image = document.createElement("img");
    image.className = "viewer-carryover-image";
    image.src = carryover.url;
    image.alt = "";
    image.decoding = "async";
    image.draggable = false;
    preview.append(image);
  }

  function clearCarryover(preview: HTMLElement) {
    deps.setCarryoverImage(null);
    preview.querySelectorAll(":scope > img.viewer-carryover-image").forEach((image) => image.remove());
  }

  function handleWheel(event: WheelEvent, preview: HTMLElement) {
    const point = imagePointFromEvent(event, preview);
    if (!point) {
      return;
    }
    event.preventDefault();
    event.stopPropagation();
    const current = deps.getTransform();
    const multiplier = event.deltaY < 0 ? 1.18 : 1 / 1.18;
    deps.setTransform(zoomViewerTransformAtPoint(current, point, current.zoom * multiplier));
    applyTransformToNode(preview);
  }

  function handleDoubleClick(event: MouseEvent, preview: HTMLElement) {
    const current = deps.getTransform();
    const point = imagePointFromEvent(event, preview);
    if (!point && current.zoom <= 1) {
      return;
    }
    event.preventDefault();
    event.stopPropagation();
    deps.setTransform(toggleViewerDoubleClickZoom(current, point ?? { x: 0, y: 0 }));
    dragState = null;
    applyTransformToNode(preview);
  }

  function handlePointerDown(event: PointerEvent, preview: HTMLElement) {
    if (event.button !== 0 || deps.getTransform().zoom <= 1 || isViewerChromeTarget(event.target)) {
      return;
    }
    event.preventDefault();
    preview.setPointerCapture?.(event.pointerId);
    dragState = { x: event.clientX, y: event.clientY };
    preview.classList.add("is-dragging");
    applyTransformToNode(preview);
  }

  function handlePointerMove(event: PointerEvent, preview: HTMLElement) {
    if (!dragState) {
      return;
    }
    event.preventDefault();
    const delta = {
      x: event.clientX - dragState.x,
      y: event.clientY - dragState.y,
    };
    dragState = { x: event.clientX, y: event.clientY };
    deps.setTransform(dragViewerTransform(deps.getTransform(), delta));
    applyTransformToNode(preview);
  }

  function endDrag(preview: HTMLElement, event?: PointerEvent) {
    if (event && preview.hasPointerCapture?.(event.pointerId)) {
      preview.releasePointerCapture?.(event.pointerId);
    }
    dragState = null;
    preview.classList.remove("is-dragging");
    applyTransformToNode(preview);
  }

  function clearDrag() {
    dragState = null;
  }

  function imagePointFromEvent(event: MouseEvent, preview: HTMLElement) {
    const image = preview.querySelector<HTMLImageElement>(":scope > img.preview-image");
    if (!image) {
      return null;
    }
    const previewRect = preview.getBoundingClientRect();
    const naturalSize = {
      width: image.naturalWidth || previewRect.width || 1,
      height: image.naturalHeight || previewRect.height || 1,
    };
    const fit = containedImageRect(
      { left: previewRect.left, top: previewRect.top, width: previewRect.width, height: previewRect.height },
      naturalSize,
    );
    const point = normalizedContainedImagePoint(
      {
        left: previewRect.left,
        top: previewRect.top,
        width: previewRect.width,
        height: previewRect.height,
      },
      naturalSize,
      { x: event.clientX, y: event.clientY },
    );
    if (!point.inside && deps.getTransform().zoom <= 1) {
      return null;
    }
    const x = clamp(event.clientX - fit.left, 0, fit.width);
    const y = clamp(event.clientY - fit.top, 0, fit.height);
    if (!Number.isFinite(x) || !Number.isFinite(y)) {
      return null;
    }
    return {
      x: clamp(x, 0, fit.width),
      y: clamp(y, 0, fit.height),
    };
  }

  function applyTransformToNode(preview: HTMLElement) {
    const image = preview.querySelector<HTMLImageElement>(":scope > img.preview-image");
    if (!image) {
      return;
    }
    const previewRect = preview.getBoundingClientRect();
    const naturalSize = {
      width: image.naturalWidth || previewRect.width || 1,
      height: image.naturalHeight || previewRect.height || 1,
    };
    const fit = containedImageRect(
      { left: 0, top: 0, width: previewRect.width, height: previewRect.height },
      naturalSize,
    );
    const current = deps.getTransform();
    const transform = `translate3d(${current.panX}px, ${current.panY}px, 0) scale(${current.zoom})`;

    image.style.left = `${fit.left}px`;
    image.style.top = `${fit.top}px`;
    image.style.right = "auto";
    image.style.bottom = "auto";
    image.style.width = `${fit.width}px`;
    image.style.height = `${fit.height}px`;
    image.style.objectFit = "fill";
    image.style.transformOrigin = "0 0";
    image.style.transform = transform;
    image.style.cursor = dragState ? "grabbing" : "";
    preview.style.cursor = dragState ? "grabbing" : "";

    preview.querySelectorAll<HTMLImageElement>(":scope > img.viewer-carryover-image").forEach((carryoverImage) => {
      const carryoverSize = {
        width: carryoverImage.naturalWidth || naturalSize.width,
        height: carryoverImage.naturalHeight || naturalSize.height,
      };
      const carryoverFit = containedImageRect(
        { left: 0, top: 0, width: previewRect.width, height: previewRect.height },
        carryoverSize,
      );
      carryoverImage.style.left = `${carryoverFit.left}px`;
      carryoverImage.style.top = `${carryoverFit.top}px`;
      carryoverImage.style.right = "auto";
      carryoverImage.style.bottom = "auto";
      carryoverImage.style.width = `${carryoverFit.width}px`;
      carryoverImage.style.height = `${carryoverFit.height}px`;
      carryoverImage.style.objectFit = "fill";
      carryoverImage.style.transformOrigin = "0 0";
      carryoverImage.style.transform = transform;
      carryoverImage.style.cursor = dragState ? "grabbing" : "";
    });

    const faceLayer = preview.querySelector<HTMLElement>(":scope > .face-risk-layer");
    if (faceLayer && !faceLayer.hidden) {
      faceLayer.style.left = `${fit.left}px`;
      faceLayer.style.top = `${fit.top}px`;
      faceLayer.style.width = `${fit.width}px`;
      faceLayer.style.height = `${fit.height}px`;
      faceLayer.style.transformOrigin = "0 0";
      faceLayer.style.transform = transform;
    }

    preview.classList.toggle("is-zoomed", current.zoom > 1);
    preview.classList.toggle("is-dragging", Boolean(dragState));
  }

  return {
    appendCarryoverImage,
    applyTransformToNode,
    clearCarryover,
    clearDrag,
    endDrag,
    handleDoubleClick,
    handlePointerDown,
    handlePointerMove,
    handleWheel,
  };
}

function isViewerChromeTarget(target: EventTarget | null) {
  return target instanceof HTMLElement && Boolean(target.closest("button, input, select, textarea"));
}

function clamp(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value));
}
