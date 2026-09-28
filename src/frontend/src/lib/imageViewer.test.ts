import { describe, expect, it } from 'vitest';
import { RESET_VIEW, clampOffset, clampScale, panBy, toggleZoom, viewerTransform, zoomIn, zoomOut } from './imageViewer';

describe('imageViewer zoom/pan state', () => {
  it('clampScale keeps scale within 1..4', () => {
    expect(clampScale(0)).toBe(1);
    expect(clampScale(2.5)).toBe(2.5);
    expect(clampScale(9)).toBe(4);
  });

  it('zoomIn/zoomOut step the scale and snap back to reset at 1x', () => {
    let state = RESET_VIEW;
    state = zoomIn(state);
    expect(state.scale).toBe(2);
    state = zoomIn(zoomIn(zoomIn(state)));
    expect(state.scale).toBe(4);
    state = zoomOut(zoomOut(zoomOut(zoomOut(state))));
    expect(state).toEqual(RESET_VIEW);
  });

  it('zoomOut clamps pan offsets to the new, smaller scale', () => {
    const state = zoomOut({ scale: 3, x: 100, y: -100 });
    expect(state.scale).toBe(2);
    expect(clampOffset(2, state.x)).toBe(state.x);
    expect(Math.abs(state.x)).toBeLessThanOrEqual(50);
  });

  it('toggleZoom flips between 1x and 2x', () => {
    expect(toggleZoom(RESET_VIEW)).toEqual({ scale: 2, x: 0, y: 0 });
    expect(toggleZoom({ scale: 3, x: 10, y: 10 })).toEqual(RESET_VIEW);
  });

  it('panBy is a no-op at 1x and clamps offsets while zoomed', () => {
    expect(panBy(RESET_VIEW, 10, 10)).toEqual(RESET_VIEW);
    const panned = panBy({ scale: 2, x: 0, y: 0 }, 1000, -1000);
    expect(panned.x).toBe(50);
    expect(panned.y).toBe(-50);
  });

  it('viewerTransform renders a translate+scale CSS transform', () => {
    expect(viewerTransform({ scale: 2, x: 10, y: -5 })).toBe('translate(10%, -5%) scale(2)');
  });
});
