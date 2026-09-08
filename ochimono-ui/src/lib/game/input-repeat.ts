/** Advance from the scheduled deadline, preserving repeats between render frames. */
export function consumeRepeats(held: { next: number }, now: number, interval: number): number {
	if (now < held.next) return 0;
	// Zero ARR means move to the wall on every update after DAS has charged.
	if (interval === 0) return Infinity;
	const count = Math.floor((now - held.next) / interval) + 1;
	held.next += count * interval;
	return count;
}
