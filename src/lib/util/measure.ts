/**
 * Shared constants + helpers for the prose measure (reading-column width).
 *
 * The measure is stored in settings as an integer number of `ch` units;
 * the CSS variable `--measure-max` holds the current value. The rendered
 * column width uses `clamp(48ch, 72vw, var(--measure-max))` so the column
 * still scales with the window below the cap.
 */

import { ipc } from '../ipc';

export const DEFAULT_MEASURE_CH = 92;
export const MIN_MEASURE_CH = 40;
export const MAX_MEASURE_CH = 160;

/** Read the currently-applied measure-max in `ch` units. */
export function readMeasureCh(): number {
  const raw = getComputedStyle(document.documentElement)
    .getPropertyValue('--measure-max')
    .trim();
  const m = /^([\d.]+)ch$/.exec(raw);
  return m ? parseFloat(m[1]) : DEFAULT_MEASURE_CH;
}

/** Write the measure-max CSS variable in `ch`. */
export function applyMeasureCh(ch: number): void {
  document.documentElement.style.setProperty('--measure-max', `${ch}ch`);
}

/** Persist a measure change through the settings store. */
export async function persistMeasureCh(ch: number): Promise<void> {
  const rounded = Math.round(ch);
  try {
    const s = await ipc.getSettings();
    await ipc.setSettings({ ...s, measureCh: rounded });
  } catch {
    // Non-fatal — the in-memory CSS value still takes effect.
  }
}

/** Measure the pixel width of one `ch` unit in the given element's font. */
export function measureChPx(within: HTMLElement): number {
  const probe = document.createElement('span');
  probe.textContent = '0'.repeat(100);
  probe.style.cssText = 'visibility:hidden;position:absolute;font:inherit;';
  within.appendChild(probe);
  const px = probe.offsetWidth / 100;
  probe.remove();
  return px || 9; // reasonable fallback
}

export function clampMeasureCh(ch: number): number {
  return Math.max(MIN_MEASURE_CH, Math.min(MAX_MEASURE_CH, ch));
}
