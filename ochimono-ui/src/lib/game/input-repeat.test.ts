import { describe, expect, it } from 'vitest';
import { consumeRepeats } from './input-repeat';

describe('held input timing', () => {
	it('preserves elapsed repeats and the fractional frame remainder', () => {
		const held = { next: 100 };
		expect(consumeRepeats(held, 99, 3)).toBe(0);
		expect(consumeRepeats(held, 110, 3)).toBe(4);
		expect(held.next).toBe(112);
		expect(consumeRepeats(held, 116, 3)).toBe(2);
		expect(held.next).toBe(118);
	});
	it('gives identical repeat counts at different render rates', () => {
		const count = (frames: number[]) => {
			const held = { next: 100 };
			return frames.reduce((total, time) => total + consumeRepeats(held, time, 1), 0);
		};
		expect(count([100, 116, 133, 150])).toBe(count([105, 150]));
	});
	it('charges DAS before instant movement and stays charged', () => {
		const held = { next: 100 };
		expect(consumeRepeats(held, 99, 0)).toBe(0);
		expect(consumeRepeats(held, 100, 0)).toBe(Infinity);
		expect(consumeRepeats(held, 116, 0)).toBe(Infinity);
	});
});
