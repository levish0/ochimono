import { Container, Graphics, Rectangle } from 'pixi.js';
import { icon, label, type Glyph } from '$lib/game-ui/rendering/visuals';
import { Motion, ease } from '$lib/game-ui/motion/motion';
import { themeAt } from '$lib/game-ui/theme';

/** Adapted sidebar selection geometry and transitions; see THIRD_PARTY_NOTICES.md. */
export class SettingsSidebarItem extends Container {
	private background = new Graphics();
	private indicator = new Graphics();
	private content = new Container();
	private selected = false;
	private hovered = false;
	private visual = { alpha: 0, height: 4, hover: 0, selected: 0 };
	constructor(title: string, glyph: Glyph, private motion: Motion, run: () => void, hoverSound: () => void) {
		super();
		const symbol = icon(glyph, 20);
		symbol.position.set(40, 23);
		const text = label(title, 16, 0xffffff);
		text.anchor.set(0, 0.5);
		text.position.set(65, 23);
		this.content.addChild(symbol, text);
		this.addChild(this.background, this.content, this.indicator);
		this.hitArea = new Rectangle(0, 0, 170, 46);
		this.eventMode = 'static';
		this.cursor = 'pointer';
		this.on('pointertap', run);
		this.on('pointerenter', () => { this.hovered = true; this.motion.to(this.visual, { hover: 1 }, 500); hoverSound(); });
		this.on('pointerleave', () => { this.hovered = false; this.motion.to(this.visual, { hover: 0 }, 500); });
	}
	update(selected: boolean, dark: number) {
		if (this.selected !== selected) {
			this.selected = selected;
			this.motion.to(this.visual, { alpha: selected ? 1 : 0, selected: selected ? 1 : 0 }, 500);
			const elasticHalf = (t: number) => t === 0 || t === 1 ? t : 1 + 2 ** (-10 * t) * Math.sin((t - 0.15) * 2 * Math.PI / 0.6);
			this.motion.to(this.visual, { height: selected ? 18 : 4 }, 500, selected ? elasticHalf : ease.outQuint);
		}
		const theme = themeAt(dark);
		this.background.clear().roundRect(5, 5, 160, 36, 5).fill({ color: theme.text, alpha: this.visual.hover * 0.1 });
		this.content.tint = this.selected || this.hovered ? theme.text : theme.muted;
		this.indicator.clear().roundRect(14, 23 - this.visual.height / 2, 4, this.visual.height, 1.5)
			.fill({ color: theme.accent, alpha: this.visual.alpha });
	}
}
