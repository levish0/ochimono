import { describe, it, expect } from 'vitest';
import { PracticeGame, type GameAction } from './engine';
import fixture from './fixtures/replay.json';

describe('shared WASM game engine', () => {
	it('applies handling settings through WASM and consumes buffered rotation at spawn', () => {
		const game = new PracticeGame('zen', {
			entryDelay: 100,
			irs: 'tap',
			ihs: 'tap',
			dcd: 3.5,
			safeLockDelay: 50,
			sonicDrop: true,
			cancelDas: false,
			preferSoftDrop: true
		});
		try {
			expect(JSON.parse(game.snapshot()).handling).toMatchObject({
				dcd: 210,
				safe_lock_delay: 3000,
				soft_drop_interval: 0,
				cancel_das_on_direction_change: false,
				prefer_soft_drop: true
			});
			game.input('hard_drop');
			expect(game.state.active).toBe(false);
			const next = game.state.queue[0];
			game.advance(1000);
			game.input('hold');
			game.input('hold', false);
			game.input('half_turn');
			game.input('half_turn', false);
			game.advance(6000);
			expect(game.state.active).toBe(true);
			expect(game.state.hold).toBe(next);
			expect(JSON.parse(game.snapshot()).rotation).toBe(2);
		} finally {
			game.destroy();
		}
	});
	it('matches native snapshots after every replay input', () => {
		const game = new PracticeGame('sprint', { das: 140, arr: 30, gravity: true }, fixture.seed);
		try {
			fixture.inputs.forEach((input, index) => {
				game.advance(input.at);
				game.input(input.action as GameAction, input.pressed);
				expect(JSON.parse(game.snapshot())).toEqual(fixture.expected[index]);
			});
		} finally {
			game.destroy();
		}
	});

	it('restores board, queue and counters when undoing a line clear', () => {
		const game = new PracticeGame();
		try {
			const state = JSON.parse(game.snapshot());
			state.piece = 'I';
			state.rotation = 0;
			state.x = 3;
			state.y = 0;
			state.board[39] = Array.from({ length: 10 }, (_, x) => (x >= 3 && x < 7 ? null : 'O'));
			game.restore(JSON.stringify(state));
			const before = game.snapshot();
			game.input('hard_drop');
			expect(game.state.lines).toBe(1);
			expect(game.undo()).toBe(true);
			expect(game.snapshot()).toBe(before);
			game.input('hard_drop');
			expect(game.state.lines).toBe(1);
		} finally {
			game.destroy();
		}
	});

	it('limits hold until placement and preserves a seven-piece bag', () => {
		const game = new PracticeGame();
		try {
			expect(new Set([game.state.piece, ...game.state.queue.slice(0, 6)]).size).toBe(7);
			const initial = game.state.piece;
			game.input('hold');
			const next = game.state.piece;
			game.input('hold');
			expect(game.state.piece).toBe(next);
			game.input('hard_drop');
			game.input('hold');
			expect(game.state.piece).toBe(initial);
		} finally {
			game.destroy();
		}
	});

	it('stops on block-out and permits undo to recover', () => {
		const game = new PracticeGame();
		try {
			const state = JSON.parse(game.snapshot());
			state.queue[0] = 'O';
			state.piece = 'O';
			state.x = 0;
			state.y = 18;
			state.board[18][4] = 'Z';
			state.board[18][5] = 'Z';
			game.restore(JSON.stringify(state));
			game.input('hard_drop');
			expect(game.state.over).toBe(true);
			const stopped = game.snapshot();
			game.input('hard_drop');
			expect(game.snapshot()).toBe(stopped);
			expect(game.undo()).toBe(true);
			expect(game.state.over).toBe(false);
		} finally {
			game.destroy();
		}
	});

	it('returns to the same orientation after four rotations', () => {
		const game = new PracticeGame();
		try {
			const original = game.state.matrix;
			for (let i = 0; i < 4; i++) game.input('clockwise');
			expect(game.state.matrix).toEqual(original);
		} finally {
			game.destroy();
		}
	});
});
