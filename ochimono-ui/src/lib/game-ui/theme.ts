export const primary = 0xff167a;
export const menuColors = {
	multiplayer: primary,
	solo: 0x25e6ef,
	records: 0xffdf37,
	settings: 0x202127
};
export const themes = {
	light: {
		background: 0xf7f7f5,
		surface: 0xffffff,
		sidebar: 0xf1f1f5,
		text: 0x202127,
		muted: 0x656570,
		accent: primary
	},
	dark: {
		background: 0x131318,
		surface: 0x202027,
		sidebar: 0x17171e,
		text: 0xf4f4fa,
		muted: 0xa6a6b5,
		accent: primary
	}
};

export function themeAt(progress: number) {
	const t = Math.max(0, Math.min(1, progress));
	const mix = (a: number, b: number) => {
		let result = 0;
		for (const shift of [16, 8, 0])
			result |= Math.round(((a >> shift) & 255) * (1 - t) + ((b >> shift) & 255) * t) << shift;
		return result;
	};
	return Object.fromEntries(
		Object.keys(themes.light).map((key) => {
			const token = key as keyof typeof themes.light;
			return [token, mix(themes.light[token], themes.dark[token])];
		})
	) as typeof themes.light;
}
