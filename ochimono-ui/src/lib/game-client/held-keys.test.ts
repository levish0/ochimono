import { expect, it } from 'vitest';
import { HeldKeys } from './held-keys';

it('keeps a rotation held until its last physical key is released', () => {
	const held = new HeldKeys();
	expect(held.press('KeyX', 'clockwise')).toBe(true);
	expect(held.press('KeyX', 'clockwise')).toBe(false);
	expect(held.press('ArrowUp', 'clockwise')).toBe(false);
	expect(held.release('KeyX')).toBeUndefined();
	expect(held.release('ArrowUp')).toBe('clockwise');
});

it('clears keys on focus loss and keeps directions independent', () => {
	const held = new HeldKeys();
	expect(held.press('ArrowLeft', 'left')).toBe(true);
	expect(held.press('ArrowRight', 'right')).toBe(true);
	held.clear();
	expect(held.release('ArrowLeft')).toBeUndefined();
	expect(held.press('ArrowRight', 'right')).toBe(true);
});
