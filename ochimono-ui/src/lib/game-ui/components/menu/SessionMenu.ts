import { Container, Graphics } from 'pixi.js';
import { MenuButton, type Action } from './MenuButton';
import { Motion } from '$lib/game-ui/motion/motion';
import { Sound } from '$lib/game-ui/audio/sound';
import { label } from '$lib/game-ui/rendering/visuals';
import { themeAt } from '$lib/game-ui/theme';

/** Shared pause and session-result presentation. */
export class SessionMenu extends Container {
	readonly buttons: MenuButton[];
	private shade = new Graphics();
	private surface = new Container();
	private heading;
	private detail;

	constructor(title: string, detail: string, actions: Action[], motion: Motion, sound: Sound, interact: () => void) {
		super();
		this.heading = label(title, 30, 0x30383c);
		this.heading.style.fontWeight = '300';
		this.heading.style.letterSpacing = 1.5;
		this.detail = label(detail, 13, 0x657078);
		this.surface.addChild(this.heading, this.detail);
		this.buttons = actions.map(action => {
			const button = new MenuButton(action, motion, sound, interact, 'home');
			this.surface.addChild(button.root);
			return button;
		});
		this.addChild(this.shade, this.surface);
	}

	layout(width: number, height: number, progress: number, dark = 0) {
		const theme = themeAt(dark);
		this.heading.style.fill = theme.text;
		this.detail.style.fill = theme.muted;
		this.shade.clear().rect(0, 0, width, height).fill({ color: theme.background, alpha: 0.94 });
		const menuWidth = Math.min(520, width - 80);
		const baseHeight = Math.min(78, (height - 200) / (this.buttons.length + 0.5));
		const heights = this.buttons.map(button => baseHeight * (1 + button.visual.hover * 0.5));
		const total = heights.reduce((sum, value) => sum + value, 0);
		this.surface.position.set((width - menuWidth) / 2 + (1 - progress) * 40, 40 + (height - 40 - total - 92) / 2);
		this.heading.position.set(0, 0);
		this.detail.position.set(0, 45);
		let y = 92;
		this.buttons.forEach((button, index) => {
			button.root.position.set(0, y);
			button.draw(menuWidth, heights[index]);
			y += heights[index];
		});
	}
}
