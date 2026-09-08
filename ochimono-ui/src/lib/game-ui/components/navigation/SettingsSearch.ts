export class SettingsSearch {
	readonly input = document.createElement('input');
	constructor(
		host: HTMLElement,
		placeholder: string,
		change: (query: string) => void,
		close: () => void
	) {
		this.input.type = 'text';
		this.input.placeholder = placeholder;
		this.input.setAttribute('aria-label', placeholder);
		this.input.className =
			'fixed z-10 border-0 bg-transparent px-3 font-sans outline-none placeholder:text-neutral-400 focus:ring-0';
		this.input.addEventListener('input', () => change(this.input.value));
		this.input.addEventListener('keydown', (event) => {
			if (event.key === 'Escape') {
				event.preventDefault();
				close();
			}
			if (event.key !== 'Tab') event.stopPropagation();
		});
		this.input.addEventListener('pointerdown', (event) => event.stopPropagation());
		host.appendChild(this.input);
	}
	layout(x: number, y: number, width: number, scale: number, color: number, visible: boolean) {
		Object.assign(this.input.style, {
			display: visible ? 'block' : 'none',
			left: `${x}px`,
			top: `${y}px`,
			width: `${width}px`,
			height: `${36 * scale}px`,
			fontSize: `${18 * scale}px`,
			color: `#${color.toString(16).padStart(6, '0')}`
		});
	}
	destroy() {
		this.input.remove();
	}
}
