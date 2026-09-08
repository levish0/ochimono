import type { GameAction } from '$lib/game/engine';

/** Several physical keys can hold one logical action without releasing each other. */
export class HeldKeys {
	private keys = new Map<string, GameAction>();
	press(code: string, action: GameAction): boolean {
		if (this.keys.has(code)) return false;
		const active = [...this.keys.values()].includes(action);
		this.keys.set(code, action);
		return !active;
	}
	release(code: string): GameAction | undefined {
		const action = this.keys.get(code);
		this.keys.delete(code);
		return action && ![...this.keys.values()].includes(action) ? action : undefined;
	}
	clear() {
		this.keys.clear();
	}
}
