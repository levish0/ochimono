import { shapes, type Piece } from './engine';
export type AmbientGrid = number[][];
export type Placement = { matrix: number[][]; x: number; y: number };
export type PlaybackStep = Placement & { action: 'move' | 'rotate' | 'soft' | 'hard' };

/** Collision-checked input path, including reachable floor rotations before lock. */
export function planPlayback(grid: AmbientGrid, piece: Piece, target: Placement): PlaybackStep[] {
	const rotations: number[][][] = [];
	let matrix = shapes[piece].map((row) => [...row]);
	for (let i = 0; i < 4; i++) {
		while (!matrix[0].some(Boolean)) matrix.shift();
		while (!matrix.at(-1)!.some(Boolean)) matrix.pop();
		while (!matrix.some((row) => row[0])) matrix = matrix.map((row) => row.slice(1));
		while (!matrix.some((row) => row.at(-1))) matrix = matrix.map((row) => row.slice(0, -1));
		rotations.push(matrix);
		matrix = matrix[0].map((_, x) => matrix.map((row) => row[x]).reverse());
	}
	const valid = (x: number, y: number, r: number) =>
		rotations[r].every((row, dy) =>
			row.every(
				(v, dx) =>
					!v ||
					(x + dx >= 0 &&
						x + dx < grid[0].length &&
						y + dy >= 0 &&
						y + dy < grid.length &&
						!grid[y + dy][x + dx])
			)
		);
	type Node = { x: number; y: number; r: number; parent: number; action: PlaybackStep['action'] };
	const nodes: Node[] = [
		{ x: Math.floor(grid[0].length / 2) - 1, y: 0, r: 0, parent: -1, action: 'move' }
	];
	const seen = new Set<string>();
	let found = -1,
		fallback = -1;
	const signature = JSON.stringify(target.matrix);
	for (let i = 0; i < nodes.length; i++) {
		const n = nodes[i];
		if (n.x === target.x && n.y === target.y && JSON.stringify(rotations[n.r]) === signature) {
			fallback = i;
			if (n.action === 'rotate' && ['T', 'S', 'Z'].includes(piece)) {
				found = i;
				break;
			}
			if (!['T', 'S', 'Z'].includes(piece)) {
				found = i;
				break;
			}
		}
		const add = (x: number, y: number, r: number, action: Node['action']) => {
			const key = `${x},${y},${r},${action === 'rotate'}`;
			if (!valid(x, y, r) || seen.has(key)) return;
			seen.add(key);
			nodes.push({ x, y, r, parent: i, action });
		};
		add(n.x - 1, n.y, n.r, 'move');
		add(n.x + 1, n.y, n.r, 'move');
		add(n.x, n.y + 1, n.r, 'soft');
		for (const direction of [-1, 1]) {
			const r = (n.r + direction + 4) % 4;
			for (const [dx, dy] of [
				[0, 0],
				[-1, 0],
				[1, 0],
				[-2, 0],
				[2, 0],
				[0, -1]
			]) {
				if (valid(n.x + dx, n.y + dy, r)) {
					add(n.x + dx, n.y + dy, r, 'rotate');
					break;
				}
			}
		}
		let y = n.y;
		while (valid(n.x, y + 1, n.r)) y++;
		if (y > n.y) add(n.x, y, n.r, 'soft');
	}
	if (found < 0) found = fallback;
	if (found < 0) return [];
	const steps: PlaybackStep[] = [];
	for (let i = found; i >= 0; i = nodes[i].parent) {
		const n = nodes[i];
		steps.unshift({ x: n.x, y: n.y, matrix: rotations[n.r], action: n.action });
	}
	if (steps.length > 1 && steps.at(-1)!.action === 'soft' && Math.random() < 0.65)
		steps.at(-1)!.action = 'hard';
	// A small, legal correction before committing to the placement.
	const first = steps[0];
	if (valid(first.x + 1, first.y, 0) && Math.random() < 0.6)
		steps.splice(1, 0, { ...first, x: first.x + 1 }, { ...first });
	const discrete: PlaybackStep[] = [];
	for (const step of steps) {
		const previous = discrete.at(-1);
		if (step.action === 'hard' && previous && step.y > previous.y + 3 && Math.random() < 0.75) {
			const stop = previous.y + 1 + Math.floor(Math.random() * (step.y - previous.y - 2));
			for (let y = previous.y + 1; y <= stop; y++) discrete.push({ ...step, y, action: 'soft' });
		}
		if (step.action === 'soft' && previous && step.y > previous.y + 1) {
			for (let y = previous.y + 1; y < step.y; y++) discrete.push({ ...step, y });
		}
		discrete.push(step);
	}
	return discrete;
}
export const fullRows = (grid: AmbientGrid) =>
	grid.flatMap((row, y) => (row.every(Boolean) ? [y] : []));
export function compactRows(grid: AmbientGrid, rows: number[]) {
	return [
		...rows.map(() => Array(grid[0].length).fill(0)),
		...grid.filter((_, y) => !rows.includes(y))
	];
}
export function stamp(grid: AmbientGrid, p: Placement, color: number) {
	const next = grid.map((row) => [...row]);
	p.matrix.forEach((row, dy) =>
		row.forEach((cell, dx) => {
			if (cell) next[p.y + dy][p.x + dx] = color;
		})
	);
	return next;
}
/** Enumerate legal hard drops; prefer complete rows and low stacks without holes. */
export function choosePlacement(grid: AmbientGrid, piece: Piece): Placement | undefined {
	let matrix = shapes[piece].map((row) => [...row]);
	let best: Placement | undefined,
		bestScore = Infinity;
	for (let rotation = 0; rotation < 4; rotation++) {
		while (!matrix[0].some(Boolean)) matrix.shift();
		while (!matrix.at(-1)!.some(Boolean)) matrix.pop();
		while (!matrix.some((row) => row[0])) matrix = matrix.map((row) => row.slice(1));
		while (!matrix.some((row) => row.at(-1))) matrix = matrix.map((row) => row.slice(0, -1));
		for (let x = 0; x <= grid[0].length - matrix[0].length; x++) {
			const collides = (y: number) =>
				matrix.some((row, dy) =>
					row.some((v, dx) => v && (y + dy >= grid.length || grid[y + dy][x + dx]))
				);
			if (collides(0)) continue;
			let y = 0;
			while (!collides(y + 1)) y++;
			const placement = { matrix, x, y };
			let next = stamp(grid, placement, 1);
			const lines = fullRows(next);
			next = compactRows(next, lines);
			let holes = 0;
			const heights = next[0].map((_, col) => {
				const first = next.findIndex((row) => row[col]);
				if (first < 0) return 0;
				for (let r = first; r < next.length; r++) if (!next[r][col]) holes++;
				return next.length - first;
			});
			const bump = heights.slice(1).reduce((sum, h, i) => sum + Math.abs(h - heights[i]), 0);
			const score =
				heights.reduce((a, b) => a + b, 0) * 0.5 + holes * 9 + bump * 0.7 - lines.length * 12;
			if (score < bestScore) {
				bestScore = score;
				best = placement;
			}
		}
		matrix = matrix[0].map((_, x) => matrix.map((row) => row[x]).reverse());
	}
	return best;
}
