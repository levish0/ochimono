// Local practice rules. Kept independent of rendering for the future shared engine.
export type Piece = 'I' | 'O' | 'T' | 'S' | 'Z' | 'J' | 'L';
export const shapes: Record<Piece, number[][]> = {
	I: [
		[0, 0, 0, 0],
		[1, 1, 1, 1],
		[0, 0, 0, 0],
		[0, 0, 0, 0]
	],
	O: [
		[1, 1],
		[1, 1]
	],
	T: [
		[0, 1, 0],
		[1, 1, 1],
		[0, 0, 0]
	],
	S: [
		[0, 1, 1],
		[1, 1, 0],
		[0, 0, 0]
	],
	Z: [
		[1, 1, 0],
		[0, 1, 1],
		[0, 0, 0]
	],
	J: [
		[1, 0, 0],
		[1, 1, 1],
		[0, 0, 0]
	],
	L: [
		[0, 0, 1],
		[1, 1, 1],
		[0, 0, 0]
	]
};
export const colors: Record<Piece, string> = {
	I: '#77dedb',
	O: '#efcf80',
	T: '#bfa2ee',
	S: '#afd995',
	Z: '#ed8f9d',
	J: '#87aef2',
	L: '#f1af7c'
};
export interface State {
	board: (Piece | null)[][];
	queue: Piece[];
	piece: Piece;
	matrix: number[][];
	x: number;
	y: number;
	hold: Piece | null;
	held: boolean;
	lines: number;
	placed: number;
	over: boolean;
}
export class PracticeGame {
	state: State;
	private history: State[] = [];
	constructor() {
		this.state = {
			board: Array.from({ length: 20 }, () => Array(10).fill(null)),
			queue: [],
			piece: 'T',
			matrix: [],
			x: 3,
			y: 0,
			hold: null,
			held: false,
			lines: 0,
			placed: 0,
			over: false
		};
		this.spawn();
	}
	private replenish() {
		while (this.state.queue.length < 7) {
			const bag = Object.keys(shapes) as Piece[];
			for (let i = bag.length - 1; i > 0; i--) {
				const j = Math.floor(Math.random() * (i + 1));
				[bag[i], bag[j]] = [bag[j], bag[i]];
			}
			this.state.queue.push(...bag);
		}
	}
	private spawn(piece?: Piece) {
		this.replenish();
		const s = this.state;
		s.piece = piece ?? s.queue.shift()!;
		s.matrix = shapes[s.piece].map((r) => [...r]);
		s.x = Math.floor((10 - s.matrix.length) / 2);
		s.y = 0;
		s.over = !this.fits(s.x, s.y);
		this.replenish();
	}
	fits(x: number, y: number, matrix = this.state.matrix) {
		return matrix.every((row, dy) =>
			row.every(
				(v, dx) =>
					!v ||
					(x + dx >= 0 &&
						x + dx < 10 &&
						y + dy < 20 &&
						(y + dy < 0 || !this.state.board[y + dy][x + dx]))
			)
		);
	}
	move(dx: number, dy = 0) {
		const s = this.state;
		if (s.over || !this.fits(s.x + dx, s.y + dy)) return false;
		s.x += dx;
		s.y += dy;
		return true;
	}
	rotate(direction = 1) {
		const s = this.state;
		if (s.over) return false;
		const n = s.matrix.length;
		const rotated = s.matrix.map((row, y) =>
			row.map((_, x) => (direction === 1 ? s.matrix[n - 1 - x][y] : s.matrix[x][n - 1 - y]))
		);
		// Deliberately a practice-only kick policy; not presented as official SRS.
		for (const [dx, dy] of [
			[0, 0],
			[-1, 0],
			[1, 0],
			[-2, 0],
			[2, 0],
			[0, -1]
		]) {
			if (this.fits(s.x + dx, s.y + dy, rotated)) {
				s.matrix = rotated;
				s.x += dx;
				s.y += dy;
				return true;
			}
		}
		return false;
	}
	ghostY() {
		let y = this.state.y;
		while (this.fits(this.state.x, y + 1)) y++;
		return y;
	}
	hold() {
		const s = this.state;
		if (s.over || s.held) return;
		const old = s.hold;
		s.hold = s.piece;
		this.spawn(old ?? undefined);
		s.held = true;
	}
	lock() {
		const s = this.state;
		if (s.over) return 0;
		this.history.push(structuredClone(s));
		if (this.history.length > 100) this.history.shift();
		s.matrix.forEach((row, dy) =>
			row.forEach((v, dx) => {
				if (v && s.y + dy >= 0) s.board[s.y + dy][s.x + dx] = s.piece;
			})
		);
		const remaining = s.board.filter((row) => row.some((v) => !v));
		const cleared = 20 - remaining.length;
		s.board = [...Array.from({ length: cleared }, () => Array(10).fill(null)), ...remaining];
		s.lines += cleared;
		s.placed++;
		s.held = false;
		this.spawn();
		return cleared;
	}
	drop() {
		if (this.state.over) return 0;
		this.state.y = this.ghostY();
		return this.lock();
	}
	undo() {
		const previous = this.history.pop();
		if (previous) this.state = previous;
		return !!previous;
	}
}
