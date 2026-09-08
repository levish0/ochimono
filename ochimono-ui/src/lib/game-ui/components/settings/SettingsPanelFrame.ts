import { Container, Graphics } from 'pixi.js';
import { icon, textAt } from '$lib/game-ui/rendering/visuals';
import { themeAt } from '$lib/game-ui/theme';

/** Expandable title followed by a pinned search field and continuous settings content. */
export class SettingsPanelFrame extends Container {
	static readonly top = 40;
	static readonly sidebar = 170;
	static readonly header = 100;
	static readonly searchHeight = 48;
	readonly surface = new Graphics();
	private search = new Graphics();
	private heading;
	private magnifier = icon('search', 22);
	constructor(title: string) {
		super();
		this.eventMode = 'none';
		this.addChild(this.surface, this.search);
		this.heading = textAt(this, title, 182, 60, 40);
		this.addChild(this.magnifier);
	}
	static searchY(offset: number) {
		return this.top + Math.max(0, this.header - offset);
	}
	static contentY(offset: number) {
		return this.searchY(offset) + this.searchHeight;
	}
	static viewportHeight(height: number) {
		return height - this.top - this.searchHeight;
	}
	layout(width: number, height: number, offset: number, themeProgress: number) {
		const theme = themeAt(themeProgress),
			y = SettingsPanelFrame.searchY(offset);
		this.surface
			.clear()
			.rect(0, 40, width, height - 40)
			.fill(theme.surface)
			.rect(0, 40, 170, height - 40)
			.fill(theme.sidebar);
		this.heading.y = 60 - Math.min(100, offset);
		this.heading.visible = offset < 60;
		this.heading.style.fill = theme.text;
		this.search
			.clear()
			.roundRect(182, y + 6, width - 204, 36, 5)
			.fill(theme.sidebar);
		this.magnifier.position.set(width - 42, y + 24);
		this.magnifier.tint = theme.text;
	}
}
