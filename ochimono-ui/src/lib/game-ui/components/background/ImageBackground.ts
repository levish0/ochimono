import { Assets, Container, Sprite } from 'pixi.js';

const inOutQuart = (t: number) => (t < 0.5 ? 8 * t ** 4 : 1 - (-2 * t + 2) ** 4 / 2);
const outExpo = (t: number) => (t === 1 ? 1 : 1 - 2 ** (-10 * t));

/** Adapted background cover, parallax and transition behavior; see THIRD_PARTY_NOTICES.md. */
export class ImageBackground extends Container {
	private image = new Sprite();
	private offset = { x: 0, y: 0 };
	private parallaxScale = 1;
	private transitionTime = 0;
	private transitionX = 50;
	private transitionAlpha = 0;
	private fromX = 50;
	private fromAlpha = 0;
	private active = true;
	private entering = true;
	constructor() {
		super();
		this.eventMode = 'none';
		this.image.anchor.set(0.5);
		this.addChild(this.image);
	}
	async load() {
		const texture = await Assets.load('/backgrounds/blocks-v1.png');
		if (!this.destroyed) this.image.texture = texture;
	}
	update(
		width: number,
		height: number,
		pointer: { x: number; y: number },
		dt: number,
		shift: number,
		reduced: boolean,
		active: boolean
	) {
		if (active !== this.active) {
			this.active = active;
			this.entering = false;
			this.fromX = this.transitionX;
			this.fromAlpha = this.transitionAlpha;
			this.transitionTime = 0;
		}
		this.transitionTime = Math.min(0.5, this.transitionTime + Math.max(0, dt));
		const t = reduced ? 1 : this.transitionTime / 0.5;
		const transition = this.entering ? inOutQuart(t) : outExpo(t);
		this.transitionX = this.fromX + ((active ? 0 : 50) - this.fromX) * transition;
		this.transitionAlpha = this.fromAlpha + ((active ? 1 : 0) - this.fromAlpha) * transition;
		this.alpha = this.transitionAlpha;
		this.visible = this.alpha > 0;
		// Pointer input is normalized around the viewport centre, in [-0.5, 0.5].
		const elapsed = Math.max(0, Math.min(dt, 0.1));
		const response = 1 - (1 - elapsed / 0.1) ** 5;
		const amount = reduced ? 0 : 0.02;
		const target = (p: number, size: number) =>
			((Math.sign(p) * (1 - 0.999 ** Math.abs(p * size)) * size) / 2) * amount;
		this.offset.x += (target(pointer.x, width) - this.offset.x) * response;
		this.offset.y += (target(pointer.y, height) - this.offset.y) * response;
		this.parallaxScale += (1 + Math.abs(amount) - this.parallaxScale) * (1 - (1 - elapsed) ** 5);
		const screenScale = 1 + 100 / width;
		const coverScale = Math.max(
			width / this.image.texture.width,
			height / this.image.texture.height
		);
		this.image.scale.set(coverScale * screenScale * this.parallaxScale);
		this.image.position.set(
			width / 2 + this.offset.x + this.transitionX + shift,
			height / 2 + this.offset.y
		);
	}
}
