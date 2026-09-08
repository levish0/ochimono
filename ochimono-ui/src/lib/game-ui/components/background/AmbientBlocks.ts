import { Graphics } from 'pixi.js';
import {
	choosePlacement,
	compactRows,
	fullRows,
	stamp,
	planPlayback,
	type PlaybackStep,
	type AmbientGrid,
	type Placement
} from '$lib/game/ambient';
import { shapes, type Piece } from '$lib/game/engine';
const shades: Record<Piece, number> = {
	I: 0xbde4e8,
	O: 0xeee0ae,
	T: 0xd8c9ec,
	S: 0xcadfbc,
	Z: 0xe8c2ce,
	J: 0xc2d2ec,
	L: 0xecd1b7
};

export class AmbientBlocks {
	readonly view = new Graphics();
	private grid: AmbientGrid = [];
	private piece?: Placement;
	private pieceColor = shades.I;
	private playback: PlaybackStep[] = [];
	private step = 0;
	private stepTime = 0;
	private bag: Piece[] = [];
	private clearing: number[] = [];
	private clearTime = 0;
	private size = 56;
	private bottomOffset = 40;
	private cellHeight = 56;
	private width = 0;
	private height = 0;
	resize(width: number, height: number) {
		if (this.width === width && this.height === height) return;
		this.width = width;
		this.height = height;
		const cols = Math.max(12, Math.round(width / 66));
		this.size = width / cols;
		const rows = Math.max(1, Math.round((height - 40) / this.size));
		this.cellHeight = (height - 40) / rows;
		this.grid = Array.from({ length: rows }, () => Array(cols).fill(0));
		// A four-cell well makes the first I-piece complete a real row.
		for (let y = rows - 2; y < rows; y++)
			for (let x = 0; x < cols - 4; x++)
				this.grid[y][x] = Object.values(shades)[Math.floor(x / 3) % 7];
		this.clearing = [];
		this.clearTime = 0;
		this.piece = choosePlacement(this.grid, 'I');
		this.prepare('I');
	}
	private prepare(kind: Piece) {
		this.pieceColor = shades[kind];
		this.playback = this.piece ? planPlayback(this.grid, kind, this.piece) : [];
		this.step = 0;
		this.stepTime = 0;
	}
	private spawn() {
		if (!this.bag.length) {
			this.bag = Object.keys(shapes) as Piece[];
			for (let i = this.bag.length - 1; i > 0; i--) {
				const j = Math.floor(Math.random() * (i + 1));
				[this.bag[i], this.bag[j]] = [this.bag[j], this.bag[i]];
			}
		}
		const kind = this.bag.pop()!;
		this.piece = choosePlacement(this.grid, kind);
		this.prepare(kind);
	}
	private duration() {
		const current = this.playback[this.step];
		const previous = this.playback[Math.max(0, this.step - 1)];
		return current?.action === 'hard'
			? 0.065
			: current?.action === 'soft'
				? Math.max(0.1, (current.y - previous.y) * 0.07)
				: 0.1;
	}
	tick(dt: number, reduced: boolean) {
		if (!reduced) {
			if (this.clearing.length) {
				this.clearTime += dt;
				if (this.clearTime > 0.42) {
					this.grid = compactRows(this.grid, this.clearing);
					this.clearing = [];
					this.spawn();
				}
			} else if (this.piece) {
				this.stepTime += dt;
				const duration = this.duration();
				if (this.stepTime >= duration) {
					this.step++;
					this.stepTime = 0;
				}
				if (this.step >= this.playback.length) {
					this.grid = stamp(this.grid, this.piece, this.pieceColor);
					this.piece = undefined;
					this.clearing = fullRows(this.grid);
					this.clearTime = 0;
					if (!this.clearing.length) this.spawn();
				}
			}
		}
		this.view.clear();
		for (let x = 0; x <= this.width; x += this.size) this.view.moveTo(x, 40).lineTo(x, this.height);
		for (let y = this.bottomOffset; y <= this.height; y += this.cellHeight)
			this.view.moveTo(0, y).lineTo(this.width, y);
		this.view.stroke({ color: 0x797d81, alpha: 0.055, width: 1 });
		const draw = (x: number, y: number, color: number, wipe = 0) => {
			const left = x * this.size;
			const removed = Math.max(0, Math.min(this.size, wipe - left));
			if (removed < this.size)
				this.view
					.rect(left + removed, 40 + y * this.cellHeight, this.size - removed, this.cellHeight)
					.fill({ color, alpha: 0.65 });
		};
		const progress = Math.min(1, this.clearTime / 0.32);
		const edge = this.width * (1 - (1 - progress) ** 3);
		this.grid.forEach((row, y) =>
			row.forEach((color, x) => {
				if (color) draw(x, y, color, this.clearing.includes(y) ? edge : 0);
			})
		);
		for (const y of this.clearing) {
			const cy = 40 + (y + 0.5) * this.cellHeight;
			this.view
				.moveTo(Math.max(0, edge - 100), cy)
				.lineTo(edge, cy)
				.stroke({ color: 0xffffff, alpha: 1 - progress, width: 2 });
			this.view
				.moveTo(edge - 4, cy - 7)
				.lineTo(edge + 4, cy + 7)
				.stroke({ color: 0xffffff, alpha: 1 - progress, width: 1 });
		}
		const active = this.playback[Math.min(this.step, this.playback.length - 1)];
		if (this.piece && active)
			active.matrix.forEach((row, y) =>
				row.forEach((v, x) => {
					if (v) draw(active.x + x, active.y + y, this.pieceColor);
				})
			);
	}
}
