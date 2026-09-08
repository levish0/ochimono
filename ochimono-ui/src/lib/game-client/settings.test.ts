import { afterEach, expect, it, vi } from 'vitest';
import { defaults, readSettings } from './settings';

afterEach(() => vi.unstubAllGlobals());

it('loads fractional timing and buffering settings without dropping existing preferences', () => {
	vi.stubGlobal('localStorage', {
		getItem: () =>
			JSON.stringify({
				dark: true,
				das: 83.3,
				arr: 0,
				dcd: 16.7,
				irs: 'tap',
				ihs: 'hold',
				sonicDrop: true,
				cancelDas: false,
				preferSoftDrop: true,
				safeLockDelay: 50,
				entryDelay: 100
			})
	});
	expect(readSettings()).toMatchObject({
		dark: true,
		das: 83.3,
		arr: 0,
		dcd: 16.7,
		irs: 'tap',
		ihs: 'hold',
		sonicDrop: true,
		cancelDas: false,
		preferSoftDrop: true,
		safeLockDelay: 50,
		entryDelay: 100
	});
});

it('defaults missing or invalid controls and bounds timing values', () => {
	vi.stubGlobal('localStorage', {
		getItem: () =>
			JSON.stringify({ irs: 'unknown', ihs: 2, dcd: -1, softDropInterval: 0, safeLockDelay: 10000 })
	});
	expect(readSettings()).toEqual({ ...defaults, safeLockDelay: 1000, softDropInterval: 1 });
});
