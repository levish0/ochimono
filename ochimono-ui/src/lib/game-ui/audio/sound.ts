/** Synthesized menu and gameplay feedback. */
export class Sound {
	private context?: AudioContext;
	volume = 25;
	play(kind: 'hover' | 'open' | 'back' | 'drop' | 'clear' | 'move') {
		if (!this.volume) return;
		try {
			const ctx = (this.context ??= new AudioContext());
			void ctx.resume();
			const now = ctx.currentTime;
			const frequencies = { hover: 840, open: 660, back: 390, drop: 150, clear: 1040, move: 480 };
			const oscillator = ctx.createOscillator(),
				gain = ctx.createGain();
			oscillator.type = kind === 'drop' ? 'triangle' : 'sine';
			oscillator.frequency.setValueAtTime(frequencies[kind], now);
			oscillator.frequency.exponentialRampToValueAtTime(
				frequencies[kind] * (kind === 'open' ? 1.5 : 0.7),
				now + 0.09
			);
			gain.gain.setValueAtTime(0, now);
			gain.gain.linearRampToValueAtTime(
				(this.volume / 100) * (kind === 'hover' ? 0.035 : 0.12),
				now + 0.005
			);
			gain.gain.exponentialRampToValueAtTime(0.0001, now + 0.16);
			oscillator.connect(gain);
			gain.connect(ctx.destination);
			oscillator.start(now);
			oscillator.stop(now + 0.17);
			oscillator.onended = () => {
				oscillator.disconnect();
				gain.disconnect();
			};
		} catch {
			/* Visual interaction remains usable without an audio device. */
		}
	}
	destroy() {
		void this.context?.close();
	}
}
