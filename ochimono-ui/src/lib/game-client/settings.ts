export interface Settings {
	volume: number;
	motion: boolean;
	ghost: boolean;
	grid: boolean;
	gravity: boolean;
	das: number;
	arr: number;
}
export const defaults: Settings = {
	volume: 25,
	motion: true,
	ghost: true,
	grid: true,
	gravity: false,
	das: 140,
	arr: 30
};
export function readSettings(): Settings {
	try {
		const raw = JSON.parse(localStorage.getItem('ochimono.settings.v1') ?? '{}');
		const s = { ...defaults };
		for (const key of ['motion', 'ghost', 'grid', 'gravity'] as const)
			if (typeof raw[key] === 'boolean') s[key] = raw[key];
		for (const [key, min, max] of [
			['volume', 0, 100],
			['das', 80, 250],
			['arr', 10, 100]
		] as const)
			if (typeof raw[key] === 'number' && Number.isFinite(raw[key]))
				s[key] = Math.min(max, Math.max(min, raw[key]));
		return s;
	} catch {
		return { ...defaults };
	}
}
