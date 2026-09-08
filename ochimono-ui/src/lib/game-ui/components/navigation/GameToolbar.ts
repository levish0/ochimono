import { Container, Graphics, Rectangle, Text } from 'pixi.js';
import { icon, type Glyph } from '$lib/game-ui/rendering/visuals';
import * as m from '$lib/paraglide/messages';
import { Motion } from '$lib/game-ui/motion/motion';
import { ToolbarClock } from './ToolbarClock';
import { ToolbarUserButton } from './ToolbarUserButton';

export type ToolbarActions = Record<
	| 'settings'
	| 'home'
	| 'records'
	| 'controls'
	| 'sound'
	| 'full'
	| 'music'
	| 'code'
	| 'profile'
	| 'notifications',
	() => void
>;
/** Reusable game chrome. Toolbar.cs: 40 logical pixels; no page layout dependency. */
export class GameToolbar extends Container {
	private background = new Graphics();
	private clock: ToolbarClock;
	private right: Container[] = [];
	private profile: ToolbarUserButton;
	private notifications: Container;
	private fullButton?: Container;
	private fullscreen = false;
	private muted = false;
	constructor(actions: ToolbarActions, context: Text, motion: Motion, hover: () => void) {
		super();
		this.clock = new ToolbarClock(motion);
		this.addChild(this.background);
		const button = (glyph: Glyph, action: () => void, width = 40) => {
			const root = new Container(),
				background = new Graphics().roundRect(-width / 2 + 3, -17, width - 6, 34, 6).fill(0xffffff);
			background.alpha = 0;
			root.addChild(background, icon(glyph, 20));
			root.hitArea = new Rectangle(-width / 2, -20, width, 40);
			root.eventMode = 'static';
			root.cursor = 'pointer';
			root.on('pointerenter', () => {
				motion.to(background, { alpha: 0.16 }, 120);
				hover();
			});
			root.on('pointerleave', () => motion.to(background, { alpha: 0 }, 250));
			root.on('pointertap', action);
			root.y = 20;
			this.addChild(root);
			return root;
		};
		let left = 0;
		(['settings', 'home'] as const).forEach((key) => {
			const glyph = { settings: 'navSettings', home: 'navHome' } as const;
			const width = 56;
			button(glyph[key], actions[key], width).x = left + width / 2;
			left += width;
		});
		context.position.set(128, 12);
		this.addChild(context);
		this.right = [
			button('navCode', actions.code),
			button('navRecords', actions.records),
			button('navControls', actions.controls),
			button('navSound', actions.sound),
			button('navFull', actions.full),
			button('navMusic', actions.music)
		];
		this.fullButton = this.right[4];
		this.syncFullscreen();
		this.profile = new ToolbarUserButton(m.ui_guest(), actions.profile, motion, hover);
		this.addChild(this.profile);
		this.addChild(this.clock);
		this.notifications = button('navNotifications', actions.notifications);
	}
	resize(width: number) {
		this.background.clear().rect(0, 0, width, 40).fill(0x111111);
		const profileX = width - 174 - this.profile.layoutWidth;
		this.right.forEach((button, i) => (button.x = profileX - (this.right.length - i) * 40 + 20));
		this.profile.position.set(profileX, 0);
		this.clock.position.set(width - 174, 0);
		this.notifications.x = width - 20;
	}
	tick(now: number, volume: number) {
		const muted = volume === 0;
		if (muted !== this.muted) {
			this.muted = muted;
			this.right[3].removeChildAt(1).destroy();
			this.right[3].addChild(icon(muted ? 'navMute' : 'navSound', 20));
		}
		this.syncFullscreen();
		this.clock.tick(now);
	}
	private syncFullscreen() {
		const fullscreen = Boolean(document.fullscreenElement);
		if (!this.fullButton || fullscreen === this.fullscreen) return;
		this.fullscreen = fullscreen;
		this.fullButton.removeChildAt(1).destroy();
		this.fullButton.addChild(icon(fullscreen ? 'navRestore' : 'navFull', 20));
	}
}
