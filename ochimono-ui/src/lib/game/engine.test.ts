import { describe, it, expect } from 'vitest';
import { PracticeGame, shapes } from './engine';
describe('local practice game', () => {
	it('keeps a complete seven-piece bag and respects both walls', () => {
		const game = new PracticeGame();
		expect(new Set([game.state.piece, ...game.state.queue.slice(0, 6)]).size).toBe(7);
		for (let i = 0; i < 20; i++) game.move(-1);
		expect(game.fits(game.state.x, game.state.y)).toBe(true);
		expect(game.move(-1)).toBe(false);
		for (let i = 0; i < 20; i++) game.move(1);
		expect(game.move(1)).toBe(false);
	});
	it('clears a completed row and restores board, queue and counters on undo', () => {
		const game = new PracticeGame();
		const s = game.state;
		s.piece = 'I';
		s.matrix = shapes.I.map((r) => [...r]);
		s.x = 3;
		s.y = 0;
		s.board[19] = Array.from({ length: 10 }, (_, x) => (x >= 3 && x < 7 ? null : 'O'));
		const board = structuredClone(s.board),
			queue = [...s.queue];
		expect(game.drop()).toBe(1);
		expect(game.state.lines).toBe(1);
		expect(game.state.board.every((r) => r.every((v) => v === null))).toBe(true);
		expect(game.undo()).toBe(true);
		expect(game.state.board).toEqual(board);
		expect(game.state.queue).toEqual(queue);
		expect(game.state.lines).toBe(0);
		expect(game.drop()).toBe(1);
	});
	it('allows one hold per placement, then enables it again', () => {
		const game = new PracticeGame();
		const initial = game.state.piece;
		game.hold();
		const next = game.state.piece;
		expect(game.state.hold).toBe(initial);
		game.hold();
		expect(game.state.piece).toBe(next);
		game.drop();
		game.hold();
		expect(game.state.piece).toBe(initial);
	});
	it('stops a blocked spawn and allows undo to recover', () => {
		const game = new PracticeGame();
		game.state.queue[0] = 'O';
		game.state.piece = 'O';
		game.state.matrix = shapes.O.map((r) => [...r]);
		game.state.x = 0;
		game.state.y = 18;
		game.state.board[0][4] = 'Z';
		game.state.board[0][5] = 'Z';
		game.drop();
		expect(game.state.over).toBe(true);
		const board = structuredClone(game.state.board);
		game.drop();
		expect(game.state.board).toEqual(board);
		expect(game.undo()).toBe(true);
	});
	it('returns to the same orientation after four rotations in open space', () => {
		const game = new PracticeGame();
		const original = structuredClone(game.state.matrix);
		for (let i = 0; i < 4; i++) game.rotate();
		expect(game.state.matrix).toEqual(original);
	});
});
