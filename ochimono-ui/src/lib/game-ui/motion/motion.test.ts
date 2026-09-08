import { afterEach, describe, expect, it, vi } from 'vitest';
import { ease, Motion } from './motion';

afterEach(() => vi.restoreAllMocks());
describe('interruptible scene motion', () => {
	it('reverses from the displayed position and cancels the previous destination', () => {
		const clock = vi.spyOn(performance, 'now').mockReturnValue(0);
		const motion = new Motion(),
			visual = { x: 0 };
		motion.to(visual, { x: 100 }, 200, ease.linear);
		motion.update(100);
		expect(visual.x).toBe(50);
		clock.mockReturnValue(100);
		motion.to(visual, { x: 0 }, 800, ease.linear);
		motion.update(500);
		expect(visual.x).toBe(25);
		motion.update(900);
		expect(visual.x).toBe(0);
	});
	it('does not resurrect a delayed reveal after the menu is closed', () => {
		const clock = vi.spyOn(performance, 'now').mockReturnValue(0);
		const motion = new Motion(),
			visual = { alpha: 0 };
		motion.to(visual, { alpha: 1 }, 300, ease.linear, 150);
		clock.mockReturnValue(50);
		motion.to(visual, { alpha: 0 }, 300);
		motion.update(1000);
		expect(visual.alpha).toBe(0);
	});
	it('finishes existing motion when reduced motion is enabled', () => {
		vi.spyOn(performance, 'now').mockReturnValue(0);
		const motion = new Motion(),
			visual = { x: 0 };
		motion.to(visual, { x: 1 }, 500);
		motion.reduced = true;
		motion.update(10);
		expect(visual.x).toBe(1);
		motion.to(visual, { x: 0 }, 800, ease.outExpo, 150);
		expect(visual.x).toBe(0);
	});
});
