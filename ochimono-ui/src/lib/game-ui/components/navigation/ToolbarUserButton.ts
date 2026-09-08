import { Container, Graphics, Rectangle } from 'pixi.js';
import { icon, label } from '$lib/game-ui/rendering/visuals';
import { Motion } from '$lib/game-ui/motion/motion';

export class ToolbarUserButton extends Container {
	readonly layoutWidth: number;

	constructor(username: string, action: () => void, motion: Motion, hover: () => void) {
		super();
		const name = label(username, 14);
		name.text = username;
		name.anchor.set(0, 0.5);
		name.position.set(20, 20);
		const avatarX = 20 + name.width + 5;
		this.layoutWidth = avatarX + 32 + 20;
		const background = new Graphics().roundRect(3, 3, this.layoutWidth - 6, 34, 6).fill(0xffffff);
		background.alpha = 0;
		const avatar = icon('navUser', 32);
		avatar.position.set(avatarX + 16, 20);
		this.addChild(background, name, avatar);
		this.hitArea = new Rectangle(0, 0, this.layoutWidth, 40);
		this.eventMode = 'static';
		this.cursor = 'pointer';
		this.on('pointerenter', () => {
			motion.to(background, { alpha: 0.16 }, 120);
			hover();
		});
		this.on('pointerleave', () => motion.to(background, { alpha: 0 }, 250));
		this.on('pointertap', action);
	}
}
