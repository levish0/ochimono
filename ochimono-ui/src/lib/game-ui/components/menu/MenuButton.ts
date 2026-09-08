import { Container, Graphics, Rectangle, Sprite } from 'pixi.js';
import { Motion, ease } from '$lib/game-ui/motion/motion';
import { Sound } from '$lib/game-ui/audio/sound';
import { icon, label, type Glyph } from '$lib/game-ui/rendering/visuals';
export type Action = {
	title: string;
	description?: string;
	disabled?: boolean;
	glyph: Glyph;
	color: number;
	run: () => void;
};
export class MenuButton {
	root = new Container();
	shape = new Graphics();
	flash = new Graphics();
	content = new Container();
	visual = { width: 0, hover: 0, flash: 0 };
	active = true;
	private lastDraw = '';
	private symbol: Sprite;
	constructor(
		public action: Action,
		private motion: Motion,
		private sound: Sound,
		private interact: () => void,
		private layout: 'tile' | 'row' | 'home' = 'tile'
	) {
		this.symbol = icon(action.glyph, 28);
		this.symbol.position.set(0, -5);
		const title = label(action.title.toUpperCase(), 22, 0xffffff);
		title.anchor.set(0.5);
		title.y = 22;
		this.content.addChild(this.symbol, title);
		if (layout === 'home') {
			title.anchor.set(0, 0.5);
			title.position.set(76, -12);
			title.style.fontWeight = '300';
			title.style.fontSize = 26;
			title.style.letterSpacing = 1.5;
			this.symbol.position.set(25, 0);
			const detail = label(action.description ?? '', 12);
			detail.alpha = 0.65;
			detail.position.set(76, 13);
			this.content.addChild(detail);
		}
		if (layout === 'row') {
			title.anchor.set(0.5);
			title.position.set(0, 22);
			title.style.fontSize = 22;
			title.style.fontWeight = '300';
			this.symbol.position.set(0, -6);
			const description = label(action.description ?? '', 13, 0xd0dae2);
			description.anchor.set(0.5);
			description.position.set(0, 43);
			description.style.fontSize = 11;
			description.visible = false;
			this.content.addChild(description);
		}
		this.root.addChild(this.shape, this.flash, this.content);
		this.root.eventMode = 'static';
		this.root.cursor = 'pointer';
		if (action.disabled) {
			this.active = false;
			this.root.cursor = 'default';
		}
		this.root.on('pointerenter', () => this.hover(true));
		this.root.on('pointerleave', () => this.hover(false));
		this.root.on('pointertap', () => this.trigger());
	}
	hover(value: boolean) {
		if (!this.active && !this.action.disabled) return;
		this.motion.to(this.visual, { hover: value ? 1 : 0 }, 500, ease.outElastic);
		if (value) {
			this.sound.play('hover');
			this.interact();
		}
	}
	trigger() {
		if (!this.active) return;
		this.interact();
		this.sound.play('open');
		this.visual.flash = 0.85;
		this.motion.to(this.visual, { flash: 0 }, 800, ease.outExpo);
		this.action.run();
	}
	draw(width: number, height = 90) {
		const signature = `${height.toFixed(3)}:${width.toFixed(3)}:${this.visual.hover.toFixed(3)}:${this.visual.flash.toFixed(3)}`;
		if (signature === this.lastDraw) return;
		this.lastDraw = signature;
		if (this.layout === 'home') {
			this.shape.clear().rect(0, 0, width, height).fill(this.action.color);
			this.flash
				.clear()
				.rect(0, 0, width, height)
				.fill({ color: 0xffffff, alpha: this.visual.hover * 0.08 + this.visual.flash * 0.35 });
			this.content.position.set(24 + this.visual.hover * 8, height / 2);
			this.content.alpha = Math.min(1, height / 60);
			this.symbol.scale.set((32 / 24) * (1 + this.visual.hover * 0.08));
			this.root.hitArea = new Rectangle(0, 0, width, height);
			return;
		}
		if (this.layout === 'row') {
			const points = [0, -45, width, -45, width, 45, 0, 45];
			this.shape.clear().poly(points).fill(this.action.color);
			this.flash
				.clear()
				.poly(points)
				.fill({ color: 0xffffff, alpha: this.visual.hover * 0.08 + this.visual.flash * 0.7 });
			this.content.x = width / 2;
			this.content.alpha = Math.min(1, Math.max(0, (width - 65) / 65));
			this.symbol.scale.set((28 / 24) * (1 + this.visual.hover * 0.2));
			this.symbol.y = -6 - this.visual.hover * 3;
			this.root.hitArea = new Rectangle(0, -45, width, 90);
			return;
		}
		const points = [0, -45, width, -45, width, 45, 0, 45];
		this.shape.clear().poly(points).fill(this.action.color);
		this.flash
			.clear()
			.poly(points)
			.fill({ color: 0xffffff, alpha: Math.max(0, this.visual.flash + this.visual.hover * 0.07) });
		this.content.x = width / 2;
		this.content.alpha = Math.min(1, Math.max(0, (width / 145 - 0.45) / 0.35));
		this.symbol.scale.set((28 / 24) * (1 + this.visual.hover * 0.2));
		this.symbol.rotation = this.visual.hover * 0.05;
		this.symbol.y = -5 - this.visual.hover * 3;
		this.root.hitArea = new Rectangle(0, -45, width, 90);
	}
}
