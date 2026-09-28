export interface ViewerState {
  scale: number;
  x: number;
  y: number;
}

export const ZOOM_MIN = 1;
export const ZOOM_MAX = 4;
export const ZOOM_STEP = 1;

export const RESET_VIEW: ViewerState = { scale: ZOOM_MIN, x: 0, y: 0 };

export const clampScale = (scale: number): number => Math.min(ZOOM_MAX, Math.max(ZOOM_MIN, scale));

export const clampOffset = (scale: number, offset: number): number => {
  const max = (clampScale(scale) - ZOOM_MIN) * 50;
  return Math.min(max, Math.max(-max, offset));
};

const settle = (state: ViewerState): ViewerState =>
  state.scale <= ZOOM_MIN ? RESET_VIEW : { ...state, x: clampOffset(state.scale, state.x), y: clampOffset(state.scale, state.y) };

export const zoomIn = (state: ViewerState): ViewerState => settle({ ...state, scale: clampScale(state.scale + ZOOM_STEP) });

export const zoomOut = (state: ViewerState): ViewerState => settle({ ...state, scale: clampScale(state.scale - ZOOM_STEP) });

export const toggleZoom = (state: ViewerState): ViewerState => (state.scale > ZOOM_MIN ? RESET_VIEW : { scale: 2, x: 0, y: 0 });

export const panBy = (state: ViewerState, dxPercent: number, dyPercent: number): ViewerState =>
  state.scale <= ZOOM_MIN
    ? state
    : { ...state, x: clampOffset(state.scale, state.x + dxPercent), y: clampOffset(state.scale, state.y + dyPercent) };

export const viewerTransform = (state: ViewerState): string => `translate(${state.x}%, ${state.y}%) scale(${state.scale})`;
