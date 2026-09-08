import { CanvasTextMetrics, Container, Graphics, Text } from 'pixi.js';
import { label as makeLabel } from '$lib/game-ui/rendering/visuals';

/** Inline canvas editor. An invisible text input supplies selection, paste and keyboard input. */
export class NumberEditor {
	private cleanup?: () => void;
	constructor(private host: HTMLElement) {}
	open(
		caption: string,
		value: Text,
		width: number,
		current: number,
		color: number,
		commit: (value: number) => void
	) {
		this.destroy();
		const parent = value.parent;
		if (!parent) return;
		const input = document.createElement('input');
		input.type = 'text';
		input.inputMode = 'numeric';
		input.autocomplete = 'off';
		input.spellcheck = false;
		input.setAttribute('aria-label', caption);
		input.className = 'fixed left-0 top-0 h-px w-px pointer-events-none opacity-0';
		input.value = String(current);
		const view = new Container();
		view.position.copyFrom(value.position);
		view.eventMode = 'none';
		const marks = new Graphics();
		const text = makeLabel(input.value, 13, color);
		view.addChild(marks, text);
		parent.addChild(view);
		value.visible = false;
		const measure = (s: string) => CanvasTextMetrics.measureText(s, text.style).width;
		const draw = () => {
			text.text = input.value;
			const start = input.selectionStart ?? input.value.length;
			const end = input.selectionEnd ?? start;
			const x = measure(input.value.slice(0, start));
			const right = measure(input.value.slice(0, end));
			marks.clear().rect(0, 18, width, 1).fill({ color, alpha: 0.5 });
			if (start !== end) marks.rect(x, 0, Math.max(1, right - x), 17).fill({ color, alpha: 0.28 });
			else marks.rect(x, 0, 1.2, 17).fill(color);
		};
		let active = true;
		const finish = (save: boolean) => {
			if (!active) return;
			const next = input.value.trim() === '' ? NaN : Number(input.value);
			this.destroy();
			if (save && Number.isFinite(next)) commit(next);
		};
		const blur = () => finish(true);
		input.addEventListener('input', draw);
		input.addEventListener('select', draw);
		input.addEventListener('keydown', (event) => {
			event.stopPropagation();
			if (event.key === 'Enter' || event.key === 'Escape') {
				event.preventDefault();
				finish(event.key === 'Enter');
				this.host.focus();
			}
		});
		input.addEventListener('keyup', (event) => {
			event.stopPropagation();
			draw();
		});
		input.addEventListener('blur', blur);
		window.addEventListener('wheel', blur, { capture: true, passive: true });
		window.addEventListener('resize', blur);
		this.host.appendChild(input);
		const frame = requestAnimationFrame(() => {
			if (active) {
				input.focus({ preventScroll: true });
				input.select();
				draw();
			}
		});
		this.cleanup = () => {
			active = false;
			cancelAnimationFrame(frame);
			window.removeEventListener('wheel', blur, true);
			window.removeEventListener('resize', blur);
			input.remove();
			if (!value.destroyed) value.visible = true;
			if (!view.destroyed) view.destroy({ children: true });
		};
		draw();
	}
	destroy() {
		const cleanup = this.cleanup;
		this.cleanup = undefined;
		cleanup?.();
	}
}
