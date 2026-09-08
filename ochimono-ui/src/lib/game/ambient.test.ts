import { describe, expect, it } from 'vitest';
import { choosePlacement, fullRows, stamp, compactRows, planPlayback } from './ambient';

describe('background board rules', () => {
	it('replays every piece through legal cells and reaches its selected landing', () => {
		const grid = Array.from({ length: 12 }, () => Array(16).fill(0));
		grid[11].fill(1, 0, 7);
		for (const kind of ['I', 'O', 'T', 'S', 'Z', 'J', 'L'] as const) {
			const target = choosePlacement(grid, kind)!;
			const path = planPlayback(grid, kind, target);
			expect(path.length).toBeGreaterThan(1);
			for (const step of path)
				step.matrix.forEach((row, y) =>
					row.forEach((v, x) => {
						if (v) expect(grid[step.y + y]?.[step.x + x]).toBe(0);
					})
				);
			expect(stamp(grid, path.at(-1)!, 2)).toEqual(stamp(grid, target, 2));
		}
	});
	it('lands an I-piece in a four-cell well and only clears a complete row', () => {
		const grid = Array.from({ length: 12 }, () => Array(20).fill(0));
		grid[11].fill(1, 0, 16);
		expect(fullRows(grid)).toEqual([]);
		const p = choosePlacement(grid, 'I')!;
		const landed = stamp(grid, p, 2);
		expect(fullRows(landed)).toEqual([11]);
		expect(compactRows(landed, [11]).every((row) => row.every((v) => v === 0))).toBe(true);
	});
	it('does not erase incomplete rows when compacting a clear', () => {
		const grid = [
			[0, 0, 0, 0],
			[1, 0, 0, 0],
			[1, 1, 1, 1]
		];
		expect(compactRows(grid, fullRows(grid))).toEqual([
			[0, 0, 0, 0],
			[0, 0, 0, 0],
			[1, 0, 0, 0]
		]);
	});
});
