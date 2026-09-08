export const ease = {
	linear: (t: number) => t,
	in: (t: number) => t * t,
	outQuint: (t: number) => 1 - (1 - t) ** 5,
	outExpo: (t: number) => (t === 1 ? 1 : 1 - 2 ** (-10 * t)),
	inSine: (t: number) => 1 - Math.cos((t * Math.PI) / 2),
	outElastic: (t: number) =>
		t === 0 || t === 1 ? t : 2 ** (-10 * t) * Math.sin(((t * 10 - 0.75) * Math.PI * 2) / 3) + 1
};
type Track = {
	object: Record<string, number>;
	key: string;
	from: number;
	to: number;
	start: number;
	duration: number;
	curve: (t: number) => number;
};
/** A new transform starts from the displayed value, including when reversing a delayed transition. */
export class Motion {
	private tracks: Track[] = [];
	reduced = false;
	to<T extends object>(
		object: T,
		values: Partial<Record<keyof T, number>>,
		duration: number,
		curve = ease.outQuint,
		delay = 0
	) {
		const numeric = object as Record<string, number>;
		for (const [key, value] of Object.entries(values)) {
			this.tracks = this.tracks.filter((t) => t.object !== numeric || t.key !== key);
			if (this.reduced) numeric[key] = value as number;
			else
				this.tracks.push({
					object: numeric,
					key,
					from: numeric[key],
					to: value as number,
					start: performance.now() + delay,
					duration,
					curve
				});
		}
	}
	update(now: number) {
		this.tracks = this.tracks.filter((t) => {
			if (now < t.start) return true;
			const p = this.reduced ? 1 : Math.min(1, (now - t.start) / t.duration);
			t.object[t.key] = t.from + (t.to - t.from) * t.curve(p);
			return p < 1;
		});
	}
	clear() {
		this.tracks = [];
	}
}
