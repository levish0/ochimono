import { Game as WasmGame } from 'ochimono-engine';
import type { Settings } from '../game-client/settings';

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

export type GameAction =
	| 'left'
	| 'right'
	| 'soft_drop'
	| 'clockwise'
	| 'counterclockwise'
	| 'half_turn'
	| 'hold'
	| 'hard_drop';
export interface State {
	board: (Piece | null)[][];
	board_top: number;
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
	complete: boolean;
	time: number;
	ghost_y: number;
}

const handling = (settings: Pick<Settings, 'das' | 'arr'>) =>
	JSON.stringify({
		das: settings.das * 60,
		arr: settings.arr * 60,
		dcd: 0,
		soft_drop_interval: 25 * 60
	});

/** Browser ownership and serialization only; all rules execute in Rust. */
export class PracticeGame {
	private engine: WasmGame;
	state: State;
	constructor(
		mode: 'zen' | 'sprint' = 'zen',
		settings = { das: 140, arr: 30, gravity: false },
		seed = String(crypto.getRandomValues(new Uint32Array(1))[0])
	) {
		this.engine = new WasmGame(seed, mode, settings.gravity, handling(settings));
		this.state = JSON.parse(this.engine.view());
	}
	private refresh() {
		this.state = JSON.parse(this.engine.view());
	}
	input(action: GameAction, pressed = true) {
		this.engine.input(JSON.stringify({ at: this.state.time, action, pressed }));
		this.refresh();
	}
	advance(ticks: number) {
		this.engine.advance(ticks);
		this.refresh();
	}
	configure(settings: Pick<Settings, 'das' | 'arr' | 'gravity'>) {
		this.engine.configure(settings.gravity, handling(settings));
		this.refresh();
	}
	ghostY() {
		return this.state.ghost_y;
	}
	snapshot() {
		return this.engine.snapshot();
	}
	restore(snapshot: string) {
		this.engine.restore(snapshot);
		this.refresh();
	}
	releaseInputs() {
		this.engine.release_inputs();
	}
	undo() {
		const changed = this.engine.undo();
		this.refresh();
		return changed;
	}
	redo() {
		const changed = this.engine.redo();
		this.refresh();
		return changed;
	}
	destroy() {
		this.engine.free();
	}
}
