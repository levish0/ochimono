import { Container, Graphics } from 'pixi.js';
import { Motion, ease } from '$lib/game-ui/motion/motion';
import { label } from '$lib/game-ui/rendering/visuals';

/** Compact square clock with an inset seconds indicator. */
export class ToolbarClock extends Container {
	private face = new Container();
	private hands = [new Graphics(), new Graphics(), new Graphics()];
	private time = label('', 14);
	private running = label('', 10, 0x9bcfd7);
	private started = performance.now();
	private second = -1;
	constructor(private motion: Motion) {
		super();
		this.face.position.set(12, 19);
		this.face.addChild(
			new Graphics().roundRect(-11, -11, 22, 22, 4).stroke({ color: 0xffffff, width: 1.4 })
		);
		this.hands.forEach((hand, i) => {
			if (i === 2) hand.circle(8.5, 0, 1.1).fill(0x87e0ee);
			else
				hand
					.moveTo(0, 0)
					.lineTo([5.5, 7.5][i], 0)
					.stroke({ color: 0xffffff, width: 1.5, cap: 'round' });
			this.face.addChild(hand);
		});
		this.face.addChild(new Graphics().circle(0, 0, 1).fill(0xffffff));
		this.time.position.set(31, 3);
		this.running.position.set(31, 22);
		this.addChild(this.face, this.time, this.running);
	}
	tick(now: number) {
		const date = new Date();
		if (date.getSeconds() === this.second) return;
		this.second = date.getSeconds();
		const minute = (date.getMinutes() + date.getSeconds() / 60) / 60;
		const fractions = [((date.getHours() % 12) + minute) / 12, minute, date.getSeconds() / 60];
		this.hands.forEach((hand, i) => {
			let rotation = fractions[i] * Math.PI * 2 - Math.PI / 2;
			while (rotation < hand.rotation - Math.PI) rotation += Math.PI * 2;
			this.motion.to(hand, { rotation }, 320, ease.outElastic);
		});
		this.time.text = date.toLocaleTimeString('en-US');
		const seconds = Math.floor((now - this.started) / 1000);
		this.running.text = `RUNNING ${String(Math.floor(seconds / 3600)).padStart(2, '0')}:${String(Math.floor(seconds / 60) % 60).padStart(2, '0')}:${String(seconds % 60).padStart(2, '0')}`;
	}
}
