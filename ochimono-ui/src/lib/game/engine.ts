import { Game as WasmGame } from 'ochimono-engine';
import { defaults, type Settings } from '../game-client/settings';

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
	active: boolean;
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

const handling = (settings: Settings) =>
	JSON.stringify({
		das: Math.round(settings.das * 60),
		arr: Math.round(settings.arr * 60),
		dcd: Math.round(settings.dcd * 60),
		soft_drop_interval: settings.sonicDrop ? 0 : Math.round(settings.softDropInterval * 60),
		safe_lock_delay: Math.round(settings.safeLockDelay * 60),
		cancel_das_on_direction_change: settings.cancelDas,
		prefer_soft_drop: settings.preferSoftDrop,
		irs: settings.irs,
		ihs: settings.ihs
	});

/** Browser ownership and serialization only; all rules execute in Rust. */
export class PracticeGame {
	private engine: WasmGame;
	private mode: 'zen' | 'sprint';
	state: State;
	constructor(
		mode: 'zen' | 'sprint' = 'zen',
		settings: Partial<Settings> = {},
		seed = String(crypto.getRandomValues(new Uint32Array(1))[0])
	) {
		this.mode = mode;
		const config = { ...defaults, ...settings };
		this.engine = new WasmGame(
			seed,
			mode,
			config.gravity,
			handling(config),
			mode === 'zen' ? Math.round(config.entryDelay * 60) : 0
		);
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
	configure(settings: Settings) {
		this.engine.configure(
			settings.gravity,
			handling(settings),
			this.mode === 'zen' ? Math.round(settings.entryDelay * 60) : 0
		);
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
