export type BufferMode = 'off' | 'hold' | 'tap';
export interface Settings {
	dark: boolean;
	volume: number;
	motion: boolean;
	ghost: boolean;
	grid: boolean;
	gravity: boolean;
	das: number;
	arr: number;
	dcd: number;
	softDropInterval: number;
	sonicDrop: boolean;
	safeLockDelay: number;
	cancelDas: boolean;
	preferSoftDrop: boolean;
	irs: BufferMode;
	ihs: BufferMode;
	entryDelay: number;
}
export const defaults: Settings = {
	dark: false,
	volume: 25,
	motion: true,
	ghost: true,
	grid: true,
	gravity: false,
	das: 140,
	arr: 30,
	dcd: 0,
	softDropInterval: 25,
	sonicDrop: false,
	safeLockDelay: 0,
	cancelDas: true,
	preferSoftDrop: false,
	irs: 'off',
	ihs: 'off',
	entryDelay: 0
};
export function readSettings(): Settings {
	try {
		const raw = JSON.parse(localStorage.getItem('ochimono.settings.v1') ?? '{}');
		const s = { ...defaults };
		for (const key of [
			'dark',
			'motion',
			'ghost',
			'grid',
			'gravity',
			'sonicDrop',
			'cancelDas',
			'preferSoftDrop'
		] as const)
			if (typeof raw[key] === 'boolean') s[key] = raw[key];
		for (const [key, min, max] of [
			['volume', 0, 100],
			['das', 0, 500],
			['arr', 0, 100],
			['dcd', 0, 1000],
			['softDropInterval', 1, 1000],
			['safeLockDelay', 0, 1000],
			['entryDelay', 0, 1000]
		] as const)
			if (typeof raw[key] === 'number' && Number.isFinite(raw[key]))
				s[key] = Math.min(max, Math.max(min, Math.round(raw[key] * 10) / 10));
		for (const key of ['irs', 'ihs'] as const)
			if (['off', 'hold', 'tap'].includes(raw[key])) s[key] = raw[key];
		return s;
	} catch {
		return { ...defaults };
	}
}
