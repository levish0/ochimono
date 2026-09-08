import { Container, Graphics } from 'pixi.js';
import { label } from '$lib/game-ui/rendering/visuals';
import { themes } from '$lib/game-ui/theme';
import { Motion, ease } from '$lib/game-ui/motion/motion';

export class KeyCaps extends Container {
	constructor(keys: string[], motion: Motion, dark: boolean, maxWidth = 160) {
		super();
		const theme = dark ? themes.dark : themes.light;
		let x = 0,
			y = 0;
		const releases: (() => void)[] = [];
		for (const key of keys) {
			const text = label(key, 12, theme.text);
			const width = Math.max(30, text.width + 18);
			if (x + width > maxWidth && x > 0) {
				x = 0;
				y += 34;
			}
			const cap = new Container();
			cap.position.set(x, y);
			cap.addChild(
				new Graphics()
					.roundRect(0, 2, width, 28, 7)
					.fill({ color: theme.text, alpha: 0.15 })
					.roundRect(0, 0, width, 27, 7)
					.fill(dark ? 0x36414b : 0xe8edef)
			);
			text.anchor.set(0.5);
			text.position.set(width / 2, 13);
			cap.addChild(text);
			this.addChild(cap);
			const top = y;
			const respond = (pressed: boolean) =>
				motion.to(cap, { y: top + (pressed ? 3 : 0) }, pressed ? 80 : 250, ease.outQuint);
			const names: Record<string, string> = {
				'←': 'ArrowLeft',
				'→': 'ArrowRight',
				'↑': 'ArrowUp',
				'↓': 'ArrowDown',
				Space: ' ',
				Esc: 'Escape',
				Ctrl: 'Control'
			};
			const down = (event: KeyboardEvent) => {
				if (event.key.toLowerCase() === (names[key] ?? key).toLowerCase()) respond(true);
			};
			const up = (event: KeyboardEvent) => {
				if (event.key.toLowerCase() === (names[key] ?? key).toLowerCase()) respond(false);
			};
			const reset = () => respond(false);
			window.addEventListener('keydown', down);
			window.addEventListener('keyup', up);
			window.addEventListener('blur', reset);
			releases.push(() => {
				window.removeEventListener('keydown', down);
				window.removeEventListener('keyup', up);
				window.removeEventListener('blur', reset);
			});
			x += width + 6;
		}
		this.on('destroyed', () => releases.forEach((release) => release()));
	}
}
