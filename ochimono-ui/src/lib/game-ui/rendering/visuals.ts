import { Container, ImageSource, Sprite, Text, Texture } from 'pixi.js';
import play from '@iconify-icons/heroicons/play-solid';
import settings from '@iconify-icons/heroicons/cog-6-tooth-solid';
import home from '@iconify-icons/heroicons/home-solid';
import back from '@iconify-icons/heroicons/arrow-left-solid';
import zen from '@iconify-icons/heroicons/cube-solid';
import sprint from '@iconify-icons/heroicons/bolt-solid';
import sound from '@iconify-icons/heroicons/speaker-wave-solid';
import mute from '@iconify-icons/heroicons/speaker-x-mark-solid';
import close from '@iconify-icons/heroicons/x-mark-solid';
import retry from '@iconify-icons/heroicons/arrow-path-solid';
import pause from '@iconify-icons/heroicons/pause-solid';
import keys from '@iconify-icons/heroicons/command-line-solid';
import full from '@iconify-icons/heroicons/arrows-pointing-out-solid';
import help from '@iconify-icons/heroicons/question-mark-circle-solid';
import check from '@iconify-icons/heroicons/check-solid';
import multiplayer from '@iconify-icons/heroicons/user-group-solid';
import records from '@iconify-icons/heroicons/trophy-solid';
import navHome from '@iconify-icons/heroicons/home';
import navSettings from '@iconify-icons/heroicons/cog-6-tooth';
import navPlay from '@iconify-icons/heroicons/play-circle';
import navRecords from '@iconify-icons/heroicons/trophy';
import navControls from '@iconify-icons/heroicons/command-line';
import navMusic from '@iconify-icons/heroicons/musical-note';
import navCode from '@iconify-icons/heroicons/code-bracket';
import navUser from '@iconify-icons/heroicons/user-circle';
import navSound from '@iconify-icons/heroicons/speaker-wave';
import navFull from '@iconify-icons/heroicons/arrows-pointing-out';
import navRestore from '@iconify-icons/heroicons/arrows-pointing-in';
import navNotifications from '@iconify-icons/heroicons/bell';

const glyphs = {
	navRestore,
	navNotifications,
	navHome,
	navSettings,
	navPlay,
	navRecords,
	navControls,
	navMusic,
	navCode,
	navUser,
	navSound,
	navFull,
	multiplayer,
	records,
	play,
	settings,
	home,
	back,
	zen,
	sprint,
	sound,
	mute,
	close,
	retry,
	pause,
	keys,
	full,
	help,
	check
};
export type Glyph = keyof typeof glyphs;
const textures = new Map<Glyph, Texture>();
let loading: Promise<void> | undefined;
export function preloadIcons() {
	return (loading ??= Promise.all(
		Object.entries(glyphs).map(async ([key, data]) => {
			// Browser SVG rendering preserves even-odd holes and compound Heroicons paths.
			const image = new Image();
			image.src = `data:image/svg+xml;charset=utf-8,${encodeURIComponent(`<svg xmlns="http://www.w3.org/2000/svg" width="96" height="96" viewBox="0 0 24 24">${data.body.replaceAll('currentColor', '#ffffff')}</svg>`)}`;
			await image.decode();
			textures.set(
				key as Glyph,
				new Texture({ source: new ImageSource({ resource: image, resolution: 4 }) })
			);
		})
	).then(() => undefined));
}
export function icon(kind: Glyph, size = 30) {
	const g = new Sprite(textures.get(kind));
	g.pivot.set(12);
	g.scale.set(size / 24);
	return g;
}
export function label(text: string, size = 18, color = 0xffffff) {
	return new Text({
		text,
		resolution: Math.min(3, Math.max(2, devicePixelRatio)),
		roundPixels: true,
		style: {
			fontFamily: ['Sora', 'Pretendard Variable'],
			fontSize: size,
			fill: color,
			fontWeight: '400'
		}
	});
}
export function textAt(
	parent: Container,
	text: string,
	x: number,
	y: number,
	size = 18,
	color = 0xffffff
) {
	const t = label(text, size, color);
	t.position.set(x, y);
	parent.addChild(t);
	return t;
}
