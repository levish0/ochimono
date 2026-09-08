import { Container, Graphics } from 'pixi.js';
import { MenuButton, type Action } from './MenuButton';
import { Motion } from '$lib/game-ui/motion/motion';
import { Sound } from '$lib/game-ui/audio/sound';
import { label } from '$lib/game-ui/rendering/visuals';

/** Gameplay overlay layout adapted from the upstream reference; see THIRD_PARTY_NOTICES.md. */
export class SessionMenu extends Container {
	readonly buttons: MenuButton[];
	private shade = new Graphics();
	private surface = new Container();
	private heading;
	private detail;

	constructor(
		title: string,
		detail: string,
		actions: Action[],
		motion: Motion,
		sound: Sound,
		interact: () => void
	) {
		super();
		this.heading = label(title, 48, 0xffdf37);
		this.heading.style.fontWeight = '600';
		this.heading.style.letterSpacing = 0;
		this.heading.anchor.set(0.5);
		this.detail = label(detail, 18, 0xffffff);
		this.detail.anchor.set(0.5);
		this.detail.style.align = 'center';
		this.surface.addChild(this.heading, this.detail);
		this.buttons = actions.map((action) => {
			const button = new MenuButton(action, motion, sound, interact, 'dialog');
			this.surface.addChild(button.root);
			return button;
		});
		this.addChild(this.shade, this.surface);
	}

	layout(width: number, height: number, _progress: number, _dark = 0) {
		this.shade.clear().rect(0, 0, width, height).fill({ color: 0x000000, alpha: 0.75 });
		const menuWidth = Math.max(0, Math.min(720, width - 80));
		const buttonHeight = Math.min(80, height * 0.105);
		const heights = this.buttons.map((button) => buttonHeight * (1 + button.visual.hover * 0.5));
		const total = heights.reduce((sum, value) => sum + value, 0);
		const top = (height - total) / 2;
		const restingTop = (height - this.buttons.length * buttonHeight) / 2;
		this.heading.position.set(width / 2, restingTop / 2);
		this.detail.position.set(width / 2, height - restingTop / 2);
		let y = top;
		this.buttons.forEach((button, index) => {
			button.root.position.set((width - menuWidth) / 2, y);
			button.draw(menuWidth, heights[index]);
			y += heights[index];
		});
	}
}
