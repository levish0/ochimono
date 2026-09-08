import { Motion, ease } from '$lib/game-ui/motion/motion';

/** Shared interruptible scroll position for Pixi settings, lists and future records views. */
export class ScrollController {
	offset = 0;
	target = 0;
	max = 0;
	constructor(private motion: Motion) {}
	scrollTo(offset: number) {
		this.target = Math.max(0, Math.min(this.max, offset));
		this.motion.to<ScrollController>(this, { offset: this.target }, 450, ease.outQuint);
	}
}
