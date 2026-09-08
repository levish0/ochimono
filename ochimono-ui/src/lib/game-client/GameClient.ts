import { SettingsSearch } from '$lib/game-ui/components/navigation/SettingsSearch';
import { KeyCaps } from '$lib/game-ui/components/menu/KeyCaps';
import { themeAt, menuColors } from '$lib/game-ui/theme';
import { SessionMenu } from '$lib/game-ui/components/menu/SessionMenu';
import { GameToolbar } from '$lib/game-ui/components/navigation/GameToolbar';
import { ScrollController } from '$lib/game-ui/scroll/ScrollController';
import { MenuButton, type Action } from '$lib/game-ui/components/menu/MenuButton';
import * as m from '$lib/paraglide/messages';
import { Application, Container, Graphics, Rectangle, Sprite, Text } from 'pixi.js';
import { PracticeGame, colors, shapes, type Piece } from '$lib/game/engine';
import { readSettings, type Settings } from './settings';
import { ease, Motion } from '$lib/game-ui/motion/motion';
import { icon, label, preloadIcons, textAt, type Glyph } from '$lib/game-ui/rendering/visuals';
import { Sound } from '$lib/game-ui/audio/sound';
import { AmbientBlocks } from '$lib/game-ui/components/background/AmbientBlocks';

type MenuState = 'initial' | 'top' | 'play' | 'multiplayer';
type Mode = 'zen' | 'sprint';
type Panel = 'settings' | 'help' | 'records' | 'profile' | 'music' | 'notifications';
type RecordEntry = { mode: Mode; lines: number; pieces: number; seconds: number; date: string };

const mint = 0x51edc2;
const formatTime = (s: number) =>
	`${Math.floor(s / 60)
		.toString()
		.padStart(2, '0')}:${(s % 60).toFixed(2).padStart(5, '0')}`;

/** One persistent scene graph. SvelteKit owns the lifecycle; every visible control is drawn here. */
export class GameClient {
	private app = new Application();
	private motion = new Motion();
	private sound = new Sound();
	private settings: Settings = readSettings();
	private themeTransition = { value: this.settings.dark ? 1 : 0 };
	private lastTheme = -1;
	private settingsSurface?: Graphics;
	private stage = new Container();
	private scene = new Container();
	private backdrop = new Graphics();
	private ambient = new AmbientBlocks();
	private shade = new Graphics();
	private menu = new Container();
	private band = new Graphics();
	private ribbon = new Container();
	private toolbar = new Container();
	private bottom = new Container();
	private gameView = new Container();
	private board = new Graphics();
	private blocks = new Graphics();
	private previews = new Graphics();
	private dropGlow = new Graphics();
	private get theme() { return themeAt(this.themeTransition.value); }
	private pauseView = new Container();
	private pauseControl?: Container;
	private sessionMenu?: SessionMenu;
	private drawer = new Container();
	private drawerShade = new Graphics();
	private drawerContent = new Container();
	private settingsRows = new Container();

	private notice = label('', 15, 0x33434e);
	private toolbarLocation = label(m.ui_home(), 13, 0xffffff);
	private toolbarUI!: GameToolbar;
	private linesText = label('0', 46, 0x30383c);
	private timeText = label('00:00.00', 30, 0xffffff);
	private piecesText = label('0', 26);
	private titleText = label('ZEN', 34, 0x30383c);
	private modeDescription = label('');
	private buttons: MenuButton[] = [];
	private oldButtons: MenuButton[] = [];
	private pauseButtons: MenuButton[] = [];
	private leftButton?: MenuButton;
	private game = new PracticeGame();
	private screen: 'menu' | 'game' = 'menu';
	private menuState: MenuState = 'initial';
	private mode: Mode = 'zen';
	private panel: Panel | null = null;
	private paused = false;
	private finished = false;
	private saved = false;
	private time = 0;
	private gravity = 0;
	private dirty = true;
	private focus = -1;
	private pointer = { x: 0, y: 0 };
	private parallax = { x: 0, y: 0 };
	private visual = {
		logoX: 0.5,
		logoY: 0.5,
		logoScale: 1,
		logoHover: 0,
		logoPress: 0,
		band: 0,
		menuAlpha: 1,
		menuX: 0,
		toolbar: 0,
		toolbarAlpha: 0,
		bottom: 0,
		gameAlpha: 0,
		gameX: 140,
		dim: 0,
		drawer: 0,
		pause: 0,
		impact: 0,
		drop: 0
	};
	private width = 1280;
	private height = 720;
	private lastInteraction = performance.now();
	private lastTick = performance.now();
	private lastClock = 0;
	private held = new Map<string, { next: number }>();
	private disposed = false;
	private initialized = false;
	private observer?: ResizeObserver;
	private media = matchMedia('(prefers-reduced-motion: reduce)');
	private records: RecordEntry[] = [];

	constructor(
		private host: HTMLElement,
		private announce: (message: string) => void
	) {
		try {
			const r = JSON.parse(localStorage.getItem('ochimono.records.v1') ?? '[]');
			if (Array.isArray(r))
				this.records = r
					.filter(
						(v) =>
							v &&
							['zen', 'sprint'].includes(v.mode) &&
							[v.lines, v.pieces, v.seconds].every(
								(n) => typeof n === 'number' && Number.isFinite(n) && n >= 0
							) &&
							typeof v.date === 'string'
					)
					.slice(0, 8);
		} catch {
			/* No saved sessions yet. */
		}
	}
	async init() {
		await Promise.all([
			document.fonts.load('300 22px "Sora"'),
			document.fonts.load('400 18px "Sora"')
		]);
		await preloadIcons();
		if (this.disposed) return;
		await this.app.init({
			background: 0xf7f7f5,
			antialias: true,
			resolution: Math.min(devicePixelRatio, 2),
			autoDensity: true,
			resizeTo: this.host,
			preference: 'webgl'
		});
		this.initialized = true;
		if (this.disposed) {
			this.app.destroy(true, { children: true });
			return;
		}
		this.host.appendChild(this.app.canvas);
		this.app.canvas.setAttribute('aria-hidden', 'true');
		this.app.stage.addChild(this.stage);
		this.scene.addChild(
			this.ambient.view,
			this.shade,
			this.gameView,
			this.menu,
			this.toolbar,
			this.bottom,
			this.pauseView
		);
		this.stage.addChild(this.backdrop, this.scene, this.drawer, this.notice);
		this.menu.addChild(this.band, this.ribbon);
		this.drawer.addChild(this.drawerShade, this.drawerContent);
		this.buildToolbar();
		this.buildGame();
		this.setMenu('top');
		this.notice.anchor.set(0.5);
		this.notice.alpha = 0;
		this.resize();
		this.observer = new ResizeObserver(() => this.resize());
		this.observer.observe(this.host);
		this.host.addEventListener('keydown', this.keyDown);
		window.addEventListener('keyup', this.keyUp);
		window.addEventListener('pointerup', this.endDrag);
		window.addEventListener('blur', this.blur);
		document.addEventListener('visibilitychange', this.visibility);
		this.host.addEventListener('pointermove', this.pointerMove);
		this.host.addEventListener('pointerdown', this.pointerDown);
		this.media.addEventListener('change', this.motionPreference);
		this.motionPreference();
		this.app.ticker.add(this.tick);
		this.announce(m.ui_main_menu_select_solo_records_or_settings());
		this.host.focus({ preventScroll: true });
	}
	private interact = () => {
		this.lastInteraction = performance.now();
	};
	private motionPreference = () => {
		this.motion.reduced = this.media.matches || !this.settings.motion;
	};
	private pointerDown = () => {
		this.host.focus({ preventScroll: true });
		this.interact();
	};
	private pointerMove = (e: PointerEvent) => {
		this.sliderDrag?.(e.clientX);
		const r = this.host.getBoundingClientRect();
		this.pointer.x = (e.clientX - r.left) / r.width - 0.5;
		this.pointer.y = (e.clientY - r.top) / r.height - 0.5;
		this.interact();
	};
	private visibility = () => {
		if (document.hidden) this.blur();
	};
	private blur = () => {
		this.endDrag();
		this.held.clear();
		if (this.screen === 'game' && !this.paused && !this.finished) this.pause(true);
	};

	private activateLogo() {
		if (this.panel) return;
		this.sound.play('open');
		this.interact();
		if (this.menuState === 'initial') this.setMenu('top');
		else if (this.menuState === 'top') this.setMenu('play');
		else this.start('zen');
	}
	private actions(state: MenuState): Action[] {
		if (state === 'multiplayer')
			return [
				{
					title: m.ui_quick_match(),
					description: m.ui_online_unavailable(),
					glyph: 'multiplayer',
					color: menuColors.multiplayer,
					disabled: true,
					run: () => {}
				},
				{
					title: m.ui_rooms(),
					description: m.ui_online_unavailable(),
					glyph: 'navControls',
					color: menuColors.solo,
					disabled: true,
					run: () => {}
				},
				{
					title: m.ui_back(),
					description: m.ui_main_menu(),
					glyph: 'back',
					color: menuColors.settings,
					run: () => this.setMenu('top')
				}
			];
		if (state === 'play')
			return [
				{
					title: 'Zen',
					description: m.ui_free_practice(),
					glyph: 'zen',
					color: menuColors.solo,
					run: () => this.start('zen')
				},
				{
					title: '40 Lines',
					description: m.ui_clear_40_lines_as_fast_as_possible(),
					glyph: 'sprint',
					color: menuColors.multiplayer,
					run: () => this.start('sprint')
				},
				{
					title: m.ui_controls(),
					description: m.ui_key_bindings_and_handling(),
					glyph: 'keys',
					color: menuColors.records,
					run: () => this.openPanel('help')
				},
				{
					title: m.ui_back(),
					description: m.ui_main_menu(),
					glyph: 'back',
					color: menuColors.settings,
					run: () => this.setMenu('top')
				}
			];
		return [
			{
				title: m.ui_multiplayer(),
				description: m.ui_quick_match_and_rooms(),
				glyph: 'multiplayer',
				color: menuColors.multiplayer,
				run: () => this.setMenu('multiplayer')
			},
			{
				title: m.ui_solo(),
				description: m.ui_zen_and_40_lines(),
				glyph: 'play',
				color: menuColors.solo,
				run: () => this.setMenu('play')
			},
			{
				title: m.ui_records(),
				description: m.ui_local_session_history(),
				glyph: 'records',
				color: menuColors.records,
				run: () => this.openPanel('records')
			},
			{
				title: m.ui_settings(),
				description: m.ui_display_controls_and_sound(),
				glyph: 'settings',
				color: menuColors.settings,
				run: () => this.openPanel('settings')
			}
		];
	}
	private setMenu(next: MenuState) {
		if (next === 'initial') next = 'top';
		this.menuState = next;
		this.toolbarLocation.text = (
			next === 'play' ? m.ui_solo() : next === 'multiplayer' ? m.ui_multiplayer() : m.ui_home()
		).toUpperCase();
		this.focus = -1;
		this.interact();
		for (const b of this.oldButtons) b.root.destroy({ children: true });
		this.oldButtons = this.buttons;
		for (const b of this.oldButtons) {
			b.active = false;
			b.root.eventMode = 'none';
			this.motion.to(b.root, { alpha: 0 }, 160);
		}
		this.buttons = [];
		this.motion.to(
			this.visual,
			{ logoX: 0.75, logoY: 0.51, logoScale: 0.8, bottom: 0, band: 0 },
			500,
			ease.outQuint
		);
		// Initial menu expansion precedes the toolbar by 200ms;
		// the toolbar then moves over 500ms with a shorter 125ms fade.
		const toolbarDelay = this.visual.toolbar === 0 ? 200 : 0;
		this.motion.to(this.visual, { toolbar: 1 }, 500, ease.outQuint, toolbarDelay);
		this.motion.to(this.visual, { toolbarAlpha: 1 }, 125, ease.outQuint, toolbarDelay);
		this.buttons = this.actions(next).map((action, i) => {
			const b = new MenuButton(action, this.motion, this.sound, this.interact, 'home');
			this.ribbon.addChild(b.root);
			if (action.disabled) {
				b.root.on('pointerenter', () => this.toast(action.description ?? m.ui_not_available_yet()));
				b.root.on('pointertap', () => this.toast(m.ui_not_available_yet()));
			}
			this.motion.to(b.visual, { width: 1 }, 500, ease.outExpo, i * 45);
			return b;
		});
		this.announce(
			next === 'multiplayer'
				? m.ui_quick_match_and_rooms()
				: next === 'top'
					? m.ui_main_menu_solo_multiplayer_records_settings()
					: m.ui_solo_zen_or_40_lines()
		);
	}
	private toolButton(
		parent: Container,
		glyph: Glyph,
		x: number,
		y: number,
		run: () => void,
		title?: string
	) {
		const root = new Container(),
			bg = new Graphics().rect(-20, -20, 40, 40).fill({ color: 0xffffff, alpha: 0.001 });
		root.addChild(bg, icon(glyph, 21));
		root.position.set(x, y);
		root.eventMode = 'static';
		root.cursor = 'pointer';
		root.hitArea = new Rectangle(-20, -20, 40, 40);
		root.on('pointerenter', () => {
			this.motion.to(root, { alpha: 0.65 }, 150);
			if (title) this.toast(title);
			this.sound.play('hover');
		});
		root.on('pointerleave', () => this.motion.to(root, { alpha: 1 }, 200));
		root.on('pointertap', () => {
			this.interact();
			this.sound.play('open');
			run();
		});
		parent.addChild(root);
		return root;
	}
	private buildToolbar() {
		this.toolbarUI = new GameToolbar(
			{
				settings: () => this.openPanel('settings'),
				home: () => this.home(),
				records: () => this.openPanel('records'),
				controls: () => this.openPanel('help'),
				profile: () => this.openPanel('profile'),
				music: () => this.openPanel('music'),
				notifications: () => this.openPanel('notifications'),
				code: () => {
					window.open('https://github.com/ochimono/ochimono', '_blank', 'noopener,noreferrer');
				},
				sound: () => {
					this.settings.volume = this.settings.volume ? 0 : 25;
					this.saveSettings();
					this.toast(this.settings.volume ? m.ui_sound_on() : m.ui_sound_off());
				},
				full: () => {
					void this.fullscreen();
				}
			},
			this.toolbarLocation,
			this.motion,
			() => this.sound.play('hover')
		);
		this.toolbar.addChild(this.toolbarUI);
	}
	private async fullscreen() {
		try {
			if (document.fullscreenElement) await document.exitFullscreen();
			else await this.host.requestFullscreen();
		} catch {
			this.toast(m.ui_fullscreen_is_unavailable_in_this_browser());
		}
	}
	private buildGame() {
		this.gameView.addChild(
			this.board,
			this.blocks,
			this.dropGlow,
			this.previews,
			this.titleText,
			this.modeDescription,
			this.linesText,
			this.timeText,
			this.piecesText
		);
		textAt(this.gameView, m.ui_hold(), -275, -222, 13, 0x657078);
		textAt(this.gameView, m.ui_next(), 181, -222, 13, 0x657078);
		textAt(this.gameView, m.ui_lines(), -275, -48, 12, 0x657078);
		textAt(this.gameView, m.ui_time(), -275, 54, 12, 0x657078);
		textAt(this.gameView, m.ui_pieces(), -275, 139, 12, 0x657078);
		this.linesText.position.set(-275, -29);
		this.timeText.position.set(-275, 76);
		this.piecesText.position.set(-275, 160);
		this.titleText.position.set(-130, -297);
		this.modeDescription.position.set(-130, -254);
		this.modeDescription.style.fontSize = 14;
		this.modeDescription.alpha = 0.7;
		const help = textAt(
			this.gameView,
			'← → move     Z / X rotate     Space drop     C hold     Esc pause',
			0,
			321,
			13,
			0x657078
		);
		help.anchor.set(0.5, 0);
		this.pauseControl = this.toolButton(this.gameView, 'pause', 268, -278, () => this.pause(true), m.ui_pause());
	}
	private start(mode: Mode) {
		this.panel = null;
		this.motion.to(this.visual, { drawer: 0 }, 300);
		this.mode = mode;
		this.game = new PracticeGame();
		this.time = 0;
		this.gravity = 0;
		this.saved = false;
		this.finished = false;
		this.paused = false;
		this.held.clear();
		this.dirty = true;
		this.screen = 'game';
		this.titleText.text = mode === 'zen' ? 'ZEN' : '40 LINES';
		this.toolbarLocation.text = this.titleText.text;
		this.modeDescription.text = mode === 'zen' ? m.ui_free_practice() : m.ui_clear_40_lines();
		this.visual.gameX = 140;
		this.motion.to(
			this.visual,
			{ menuAlpha: 0, menuX: -700, band: 0, bottom: 0 },
			400,
			ease.inSine
		);
		this.motion.to(
			this.visual,
			{ gameAlpha: 1, gameX: 0, dim: 0, pause: 0 },
			600,
			ease.outQuint
		);
		this.motion.to(this.visual, { toolbar: 0 }, 500, ease.outQuint);
		this.motion.to(this.visual, { toolbarAlpha: 0 }, 500, ease.inQuint);
		this.announce(m.ui_session_started({ mode: mode === 'zen' ? 'Zen' : '40 Lines' }));
	}
	private home() {
		if (this.screen === 'game' && !this.paused && !this.finished) {
			this.pause(true);
			return;
		}
		if (this.screen === 'game') this.saveRecord();
		this.screen = 'menu';
		this.paused = false;
		this.held.clear();
		this.panel = null;
		this.motion.to(
			this.visual,
			{ menuAlpha: 1, menuX: 0, gameAlpha: 0, dim: 0, pause: 0, drawer: 0 },
			500,
			ease.outQuint
		);
		this.setMenu('top');
		this.sound.play('back');
	}
	private pause(value: boolean) {
		if (this.screen !== 'game' || (!value && this.finished)) return;
		this.paused = value;
		this.held.clear();
		this.motion.to(this.visual, { pause: value ? 1 : 0 }, 200, ease.in);
		if (value) {
			this.buildPause();
			this.announce(
				this.finished
					? m.ui_session_ended_restart_or_return_to_the_menu()
					: m.ui_paused_enter_to_resume_r_to_restart_escape_to_resume()
			);
		} else this.announce(m.ui_resumed());
	}
	private buildPause() {
		this.pauseButtons = [];
		this.focus = -1;
		this.pauseView.removeChildren().forEach((c) => c.destroy({ children: true }));
		const actions: Action[] = [
			...(this.finished
				? []
				: [
						{
							title: m.ui_continue(),
							glyph: 'play' as Glyph,
							color: menuColors.settings,
							run: () => this.pause(false)
						}
					]),
			{ title: m.ui_retry(), glyph: 'retry', color: menuColors.solo, run: () => this.start(this.mode) },
			{
				title: m.ui_settings(),
				glyph: 'settings',
				color: menuColors.records,
				run: () => this.openPanel('settings')
			},
			{ title: m.ui_leave(), glyph: 'back', color: menuColors.settings, run: () => this.home() }
		];
		this.sessionMenu = new SessionMenu(
			this.finished ? (this.game.state.over ? m.ui_game_over() : m.ui_complete()) : m.ui_paused(),
			`${this.titleText.text}  ·  ${this.game.state.lines} ${m.ui_lines()}  ·  ${formatTime(this.time)}`,
			actions, this.motion, this.sound, this.interact
		);
		this.pauseButtons = this.sessionMenu.buttons;
		this.pauseView.addChild(this.sessionMenu);
		this.sessionMenu.layout(this.width, this.height, this.visual.pause, this.themeTransition.value);
	}
	private saveRecord() {
		if (this.saved || !this.game.state.placed) return;
		this.saved = true;
		this.records.unshift({
			mode: this.mode,
			lines: this.game.state.lines,
			pieces: this.game.state.placed,
			seconds: this.time,
			date: new Date().toISOString()
		});
		this.records = this.records.slice(0, 8);
		try {
			localStorage.setItem('ochimono.records.v1', JSON.stringify(this.records));
		} catch {
			this.toast(m.ui_unable_to_save_records_in_this_browser());
		}
	}
	private openPanel(panel: Panel) {
		if (this.screen === 'game') this.pause(true);
		this.panel = panel;
		this.held.clear();
		this.buildDrawer();
		this.motion.to(this.visual, { drawer: 1 }, 600, ease.outQuint);
		this.announce(
			panel === 'notifications'
				? m.ui_notifications()
				: panel === 'profile'
					? m.ui_profile()
					: panel === 'music'
						? m.ui_music()
						: panel === 'settings'
							? m.ui_settings_tab_to_select_enter_to_toggle_arrow_keys_to_adjust_escape_to_close()
							: panel === 'help'
								? m.ui_controls_escape_to_close()
								: m.ui_local_records_escape_to_close()
		);
	}
	private closePanel() {
		this.endDrag();
		this.host.focus({ preventScroll: true });
		this.panel = null;
		this.motion.to(this.visual, { drawer: 0 }, 600, ease.outQuint);
		this.sound.play('back');
		this.focus = -1;
	}
	private panelActions: (() => void)[] = [];
	private panelAdjust: (((direction: number) => void) | null)[] = [];
	private sliderDrag: ((clientX: number) => void) | null = null;
	private endDrag = () => {
		if (this.sliderDrag) this.saveSettings();
		this.sliderDrag = null;
	};
	private panelFocus: Graphics[] = [];
	private toggleDraws: (() => void)[] = [];
	private buildDrawer() {
		this.toggleDraws = [];
		this.settingsRows = new Container();
		this.drawerContent.removeChildren().forEach((c) => c.destroy({ children: true }));
		this.panelActions = [];
		this.panelAdjust = [];
		this.panelFocus = [];
		this.focus = -1;
		const w = Math.min(this.panel === 'settings' ? 700 : 530, this.width - 36);
		this.drawerShade
			.clear()
			.rect(0, 0, this.width, this.height)
			.fill({ color: 0x070a13, alpha: 0.22 });
		this.drawerShade.eventMode = 'static';
		this.drawerShade.cursor = 'pointer';
		this.drawerShade.removeAllListeners();
		this.drawerShade.on('pointertap', () => this.closePanel());
		this.drawerContent.addChild(
			new Graphics()
				.rect(0, 0, w, this.height)
				.fill({ color: this.theme.surface, alpha: 1 })
				.rect(0, 0, 5, this.height)
				.fill(this.theme.accent)
		);
		this.drawerContent.eventMode = 'static';
		this.drawerContent.removeAllListeners();
		this.drawerContent.on('pointertap', (e) => e.stopPropagation());
		textAt(
			this.drawerContent,
			this.panel === 'notifications'
				? m.ui_notifications()
				: this.panel === 'profile'
					? m.ui_profile()
					: this.panel === 'music'
						? m.ui_music()
						: this.panel === 'settings'
							? m.ui_settings()
							: this.panel === 'help'
								? m.ui_controls()
								: m.ui_records(),
			32,
			54,
			32,
			0xffffff
		);
		this.toolButton(this.drawerContent, 'close', w - 35, 67, () => this.closePanel());
		if (this.panel === 'settings') {
			this.buildSettingsSections();
		} else if (this.panel === 'notifications') {
			textAt(this.drawerContent, m.ui_notifications_empty(), 33, 140, 24);
		} else if (this.panel === 'profile' || this.panel === 'music') {
			textAt(
				this.drawerContent,
				this.panel === 'profile' ? m.ui_guest() : m.ui_music_empty(),
				33,
				140,
				24
			);
			textAt(
				this.drawerContent,
				this.panel === 'profile' ? m.ui_guest_profile() : m.ui_music_local(),
				33,
				195,
				16
			);
			if (this.panel === 'profile') textAt(this.drawerContent, m.ui_accounts_later(), 33, 230, 14);
		} else if (this.panel === 'help') {
			textAt(this.drawerContent, m.ui_keyboard_bindings(), 33, 105, 15, 0x9faebb);
			const rows = [
				['←  →', m.ui_move_left_right()],
				['↓', m.ui_soft_drop()],
				['X / ↑  ·  Z', m.ui_rotate_clockwise_counterclockwise()],
				['Space', m.ui_hard_drop()],
				['C / Shift', m.ui_hold()],
				['Ctrl + Z', m.ui_undo_zen()],
				['Esc', m.ui_pause_back()],
				['Tab / ← →  ·  Enter', m.ui_select_confirm()]
			];
			let rowY = 160;
			rows.forEach(([key, desc]) => {
				const keyText = new KeyCaps(key.split(/[ /·]+/).filter(Boolean), this.motion, this.settings.dark, 150);
				keyText.position.set(33, rowY);
				this.drawerContent.addChild(keyText);
				const description = textAt(this.drawerContent, desc, 205, rowY, 15);
				description.style.wordWrap = true;
				description.style.wordWrapWidth = w - 238;
				description.style.breakWords = true;
				rowY += Math.max(48, keyText.height + 18, description.height + 18);
			});
		} else {
			textAt(this.drawerContent, m.ui_sessions_on_this_device(), 33, 105, 15, 0x9faebb);
			if (!this.records.length) {
				textAt(this.drawerContent, m.ui_no_sessions_recorded(), 33, 190, 22);
				textAt(this.drawerContent, m.ui_completed_sessions_appear_here(), 33, 229, 16, 0x9faebb);
			}
			this.records.forEach((r, i) => {
				textAt(
					this.drawerContent,
					r.mode === 'zen' ? 'ZEN' : '40 LINES',
					33,
					157 + i * 57,
					16,
					mint
				);
				textAt(this.drawerContent, `${r.lines} lines · ${r.pieces} pieces`, 145, 157 + i * 57, 15);
				textAt(this.drawerContent, formatTime(r.seconds), w - 106, 157 + i * 57, 15);
				textAt(
					this.drawerContent,
					new Date(r.date).toLocaleDateString('en-US'),
					145,
					178 + i * 57,
					11,
					0x9faebb
				);
			});
		}
		this.styleDrawer();
	}
	private styleDrawer() {
		const visit = (node: Container) => {
			if (node instanceof Text) node.style.fill = node.style.fill === mint || node.style.fill === this.theme.accent ? this.theme.accent : this.theme.text;
			if (node instanceof Sprite) node.tint = this.theme.muted;
			node.children.forEach(visit);
		};
		visit(this.drawerContent);
	}
	private settingsSearch?: SettingsSearch;
	private searchEmpty?: Text;
	private searchEntries: { node: Container; y: number; text: string }[] = [];
	private settingsContentHeight = 1010;
	private settingsScroll = new ScrollController(this.motion);
	private settingsSelection = new Graphics();
	private settingsScrollbar = new Graphics();
	private scrollSettings(offset: number) {
		this.endDrag();
		this.settingsScroll.scrollTo(offset);
	}
	private buildSettingsSections() {
		this.drawerContent.removeChildren().forEach((c) => c.destroy({ children: true }));
		const w = Math.min(700, this.width - 36);
		const viewportHeight = this.height - 124;
		this.settingsScroll.max = Math.max(0, this.settingsContentHeight - viewportHeight);
		this.settingsScroll.offset = Math.min(this.settingsScroll.offset, this.settingsScroll.max);
		this.settingsScroll.target = this.settingsScroll.offset;
		this.settingsSurface = new Graphics();
		this.drawerContent.addChild(this.settingsSurface);
		this.drawSettingsSurface();

		this.settingsSearch ??= new SettingsSearch(this.host, m.ui_search_settings(), query => this.filterSettings(query), () => this.closePanel());
		this.toolButton(this.drawerContent, 'close', w - 30, 37, () => this.closePanel());
		this.settingsSelection = new Graphics()
			.roundRect(8, 0, 154, 52, 6)
			.fill({ color: this.theme.accent, alpha: 0.1 })
			.roundRect(0, 10, 3, 32, 1)
			.fill(this.theme.accent);
		this.drawerContent.addChild(this.settingsSelection);
		const sections = [
			[m.ui_display(), 'full', 0],
			[m.ui_controls(), 'keys', 365],
			[m.ui_sound(), 'sound', 805]
		] as const;
		sections.forEach(([title, glyph, offset], i) => {
			const y = 143 + i * 64;
			const row = new Container();
			row.y = y;
			const symbol = icon(glyph, 22);
			symbol.position.set(34, 0);
			row.addChild(symbol);
			textAt(row, title, 65, -10, 16);
			row.hitArea = new Rectangle(0, -26, 170, 52);
			row.eventMode = 'static';
			row.cursor = 'pointer';
			row.on('pointertap', () => { if (this.settingsSearch) this.settingsSearch.input.value = ''; this.filterSettings(''); this.scrollSettings(offset); });
			row.on('pointerenter', () => this.sound.play('hover'));
			this.drawerContent.addChild(row);
		});
		const viewport = new Container();
		viewport.position.set(170, 116);
		viewport.eventMode = 'static';
		viewport.hitArea = new Rectangle(0, 0, w - 170, viewportHeight);
		const mask = new Graphics().rect(170, 116, w - 170, viewportHeight).fill(0xffffff);
		this.drawerContent.addChild(viewport, mask);
		viewport.mask = mask;
		viewport.addChild(this.settingsRows);
		viewport.on('wheel', (event) => {
			event.preventDefault();
			this.scrollSettings(this.settingsScroll.target + event.deltaY / this.stage.scale.y);
		});
		textAt(this.settingsRows, m.ui_display_and_gameplay(), 33, 12, 24);
		this.settingRow(
			m.ui_interface_motion(),
			this.settings.motion ? m.ui_on() : m.ui_off(),
			65,
			() => this.toggle('motion')
		);
		this.settingRow(m.ui_ghost_piece(), this.settings.ghost ? m.ui_on() : m.ui_off(), 126, () =>
			this.toggle('ghost')
		);
		this.settingRow(m.ui_board_grid(), this.settings.grid ? m.ui_on() : m.ui_off(), 187, () =>
			this.toggle('grid')
		);
		this.settingRow(m.ui_zen_gravity(), this.settings.gravity ? m.ui_on() : m.ui_off(), 248, () =>
			this.toggle('gravity')
		);
		this.settingRow(m.ui_dark_mode(), this.settings.dark ? m.ui_on() : m.ui_off(), 309, () => this.toggle('dark'));
		textAt(this.settingsRows, m.ui_controls(), 33, 377, 24);
		this.sliderRow(m.ui_repeat_delay_das(), 'das', 430, 80, 250, 10, ' ms');
		textAt(
			this.settingsRows,
			m.ui_delay_before_a_held_key_starts_repeating(),
			34,
			486,
			12,
			0x65717c
		);
		this.sliderRow(m.ui_repeat_interval_arr(), 'arr', 545, 10, 100, 10, ' ms');
		textAt(this.settingsRows, m.ui_lower_values_repeat_movement_faster(), 34, 601, 12, 0x65717c);
		textAt(this.settingsRows, m.ui_left_right_move_z_x_rotate_space_drop(), 34, 671, 14);
		textAt(this.settingsRows, m.ui_c_hold_ctrl_z_undo_esc_pause(), 34, 705, 14);
		textAt(this.settingsRows, m.ui_sound(), 33, 817, 24);
		this.sliderRow(m.ui_effects_volume(), 'volume', 870, 0, 100, 5, '%');
		textAt(this.settingsRows, m.ui_menu_and_gameplay_effects(), 34, 928, 14, 0x65717c);
		this.settingsScrollbar = new Graphics();
		this.settingsScrollbar.x = w - 7;
		this.drawerContent.addChild(this.settingsScrollbar);		const collect = (node: Container): string => node instanceof Text ? node.text : node.children.map(collect).join(' ');
		this.searchEntries = this.settingsRows.children.map(node => ({ node, y: node.y, text: collect(node).toLowerCase() }));
		this.searchEmpty = textAt(this.settingsRows, m.ui_no_matching_settings(), 33, 12, 16, this.theme.muted);
		this.filterSettings(this.settingsSearch.input.value);
		
	}
	private drawSettingsSurface() {
		const w = Math.min(700, this.width - 36);
		this.settingsSurface?.clear().rect(0, 0, w, this.height).fill(this.theme.surface).rect(0, 0, 170, this.height).fill(this.theme.sidebar);
	}
	private updateTheme() {
		if (this.lastTheme === this.themeTransition.value) return;
		this.lastTheme = this.themeTransition.value;
		this.backdrop.clear().rect(0, 0, this.width, this.height).fill(this.theme.background);
		if (this.panel === 'settings') { this.drawSettingsSurface(); this.styleDrawer(); }
		for (const text of [this.titleText, this.linesText, this.timeText, this.piecesText]) text.style.fill = this.theme.text;
		this.modeDescription.style.fill = this.theme.muted;
		if (this.pauseControl) this.pauseControl.tint = this.theme.text;
		this.ambient.dark = this.themeTransition.value;
	}	private filterSettings(query: string) {
		const words = query.toLowerCase().trim().split(/\s+/).filter(Boolean);
		let y = 12;
		for (const entry of this.searchEntries) {
			entry.node.visible = words.every(word => entry.text.includes(word));
			entry.node.y = words.length ? y : entry.y;
			if (entry.node.visible) y += entry.node.height + 14;
		}
		this.settingsContentHeight = words.length ? y + 20 : 1010;
		if (this.searchEmpty) this.searchEmpty.visible = words.length > 0 && !this.searchEntries.some(entry => entry.node.visible);
		this.settingsScroll.max = Math.max(0, this.settingsContentHeight - (this.height - 124));
		this.settingsScroll.offset = this.settingsScroll.target = 0;
		this.focus = -1;
	}
	private settingRow(title: string, value: string, y: number, run: () => void) {
		const w = Math.min(530, this.width - 36),
			root = new Container();
		root.y = y;
		const bg = new Graphics()
			.roundRect(20, 0, w - 40, 48, 4)
			.fill({ color: this.theme.text, alpha: this.settings.dark ? 0.025 : 0.025 });
		root.addChild(bg);
		textAt(root, title, 34, 14, 16);
		const v = textAt(root, value, w - 37, 14, 16, mint);
		v.anchor.x = 1;
		root.eventMode = 'static';
		root.cursor = 'pointer';
		root.hitArea = new Rectangle(20, 0, w - 40, 48);
		root.on('pointerenter', () => {
			this.motion.to(bg, { alpha: 2 }, 180);
			this.sound.play('hover');
		});
		root.on('pointerleave', () => this.motion.to(bg, { alpha: 1 }, 250));
		const toggle = new Graphics();
		let on = value === m.ui_on();
		const state = { position: on ? 1 : 0, fill: on ? 1 : 0 };
		v.visible = false;
		const draw = () => {
			const width = 42 + 14 * state.position;
			const right = w - 39;
			toggle
				.clear()
				.roundRect(right - width, 16, width, 16, 8)
				.stroke({ color: this.theme.accent, width: 3.2, alpha: 0.65 + 0.35 * Math.max(0, Math.min(1, state.fill)) })
				.roundRect(right - width, 16, width, 16, 8)
				.fill({ color: this.theme.accent, alpha: Math.max(0, Math.min(1, state.fill)) });
		};
		draw();
		this.toggleDraws.push(draw);
		root.addChild(toggle);
		const activate = () => {
			this.sound.play('open');
			run();
			on = !on;
			this.motion.to(state, { position: on ? 1 : 0 }, on ? 200 : 120, on ? ease.outElasticQuarter : ease.outExpo);
			this.motion.to(state, { fill: on ? 1 : 0 }, 250, ease.outQuint);
		};
		root.on('pointertap', activate);
		this.settingsRows.addChild(root);
		this.panelActions.push(activate);
		this.panelAdjust.push(null);
		this.panelFocus.push(bg);
	}
	private sliderRow(
		title: string,
		key: 'volume' | 'das' | 'arr',
		y: number,
		min: number,
		max: number,
		step: number,
		unit: string
	) {
		const w = Math.min(530, this.width - 36),
			root = new Container();
		root.y = y;
		const bg = new Graphics()
			.roundRect(20, 0, w - 40, 48, 4)
			.fill({ color: this.theme.text, alpha: this.settings.dark ? 0.025 : 0.025 });
		const track = new Graphics();
		root.addChild(bg, track);
		textAt(root, title, 34, 14, 16);
		const value = textAt(root, '', 34, 30, 13, mint);
		value.anchor.x = 0;
		const left = 270,
			right = w - 37;
		const redraw = () => {
			const x = left + ((right - left) * (this.settings[key] - min)) / (max - min);
			track
				.clear()
				.roundRect(left, 4, right - left, 40, 5)
				.fill(0x526171)
				.roundRect(left, 4, Math.max(1, x - left), 40, 5)
				.fill(this.theme.accent)
				.roundRect(Math.max(left, Math.min(right - 10, x - 5)), 8, 10, 32, 5)
				.fill(0xffffff);
			value.text = `${this.settings[key]}${unit}`;
		};
		const set = (v: number) => {
			this.settings[key] = Math.min(max, Math.max(min, Math.round(v / step) * step));
			redraw();
		};
		const adjust = (direction: number) => {
			set(this.settings[key] + direction * step);
			this.saveSettings();
		};
		redraw();
		root.eventMode = 'static';
		root.cursor = 'pointer';
		root.hitArea = new Rectangle(20, 0, w - 40, 48);
		root.on('pointerdown', (e) => {
			const rect = this.host.getBoundingClientRect();
			const x0 = root.toGlobal({ x: left, y: 0 }).x + rect.left;
			const length = (right - left) * this.stage.scale.x;
			this.sliderDrag = (clientX) => set(min + ((clientX - x0) / length) * (max - min));
			this.sliderDrag(e.clientX);
			this.saveSettings();
		});
		this.settingsRows.addChild(root);
		this.panelActions.push(() => adjust(1));
		this.panelAdjust.push(adjust);
		this.panelFocus.push(bg);
	}
	private toggle(key: 'dark' | 'motion' | 'ghost' | 'grid' | 'gravity') {
		this.settings[key] = !this.settings[key];
		if (key === 'dark') this.motion.to(this.themeTransition, { value: this.settings.dark ? 1 : 0 }, 450, ease.outQuint);
		this.saveSettings();
	}
	private saveSettings() {
		this.sound.volume = this.settings.volume;
		this.motionPreference();
		this.dirty = true;
		try {
			localStorage.setItem('ochimono.settings.v1', JSON.stringify(this.settings));
		} catch {
			this.toast(m.ui_unable_to_save_settings_in_this_browser());
		}
	}
	private toast(message: string) {
		this.notice.text = message;
		this.notice.alpha = 1;
		this.motion.to(this.notice, { alpha: 0 }, 400, ease.outQuint, 1600);
	}

	private keyDown = (e: KeyboardEvent) => {
		if (e.altKey || e.metaKey || e.key === 'F5' || e.key === 'F12') return;
		const key = e.code;
		if (
			![
				'Tab',
				'Escape',
				'Enter',
				'Space',
				'ArrowLeft',
				'ArrowRight',
				'ArrowDown',
				'ArrowUp',
				'KeyZ',
				'KeyX',
				'KeyC',
				'ShiftLeft',
				'ShiftRight',
				'KeyR',
				'KeyP',
				'KeyS'
			].includes(key)
		)
			return;
		e.preventDefault();
		this.interact();
		if (e.repeat) return;
		if (this.panel) {
			if (key === 'Escape') this.closePanel();
			else if (key === 'Tab' || key === 'ArrowDown' || key === 'ArrowUp') {
				const dir = e.shiftKey || key === 'ArrowUp' ? -1 : 1;
				const n = this.panelActions.length;
				if (!n || !this.panelFocus.some(item => item.parent.visible)) return;
				do { this.focus = (this.focus + dir + n) % n; } while (!this.panelFocus[this.focus].parent.visible && this.panelFocus.some(item => item.parent.visible));
				this.panelFocus.forEach((b, i) => (b.alpha = i === this.focus ? 3 : 1));
				if (this.panel === 'settings') {
					const y = this.panelFocus[this.focus].parent.y;
					if (
						y < this.settingsScroll.offset ||
						y + 48 > this.settingsScroll.offset + this.height - 124
					)
						this.scrollSettings(y - 24);
				}
			} else if (key === 'Enter' || key === 'Space') {
				const index = this.focus;
				this.panelActions[index]?.();
				this.focus = index;
				this.panelFocus.forEach((b, i) => (b.alpha = i === index ? 3 : 1));
			} else if (key === 'ArrowLeft' || key === 'ArrowRight') {
				this.panelAdjust[this.focus]?.(key === 'ArrowLeft' ? -1 : 1);
			}
			return;
		}
		if (this.screen === 'menu') {
			if (key === 'Escape') {
				this.sound.play('back');
				this.setMenu(this.menuState === 'play' ? 'top' : 'initial');
				return;
			}
			if (this.menuState === 'initial') {
				this.activateLogo();
				return;
			}
			if (
				key === 'ArrowUp' ||
				key === 'ArrowDown' ||
				key === 'ArrowLeft' ||
				key === 'ArrowRight' ||
				key === 'Tab'
			) {
				const dir = key === 'ArrowUp' || key === 'ArrowLeft' || e.shiftKey ? -1 : 1;
				this.focus = (this.focus + dir + this.buttons.length) % this.buttons.length;
				this.buttons.forEach((b, i) => b.hover(i === this.focus));
				return;
			}
			if (key === 'Enter' || key === 'Space') {
				if (this.focus >= 0) this.buttons[this.focus]?.trigger();
				else this.buttons[0]?.trigger();
			}
			if (key === 'KeyP') this.setMenu('play');
			if (key === 'KeyS') this.openPanel('settings');
			return;
		}
		if (key === 'Escape') {
			if (this.finished) this.home();
			else this.pause(!this.paused);
			return;
		}
		if (this.paused || this.finished) {
			if (key === 'Tab' || key === 'ArrowUp' || key === 'ArrowDown' || key === 'ArrowLeft' || key === 'ArrowRight') {
				const n = this.pauseButtons.length,
					dir = e.shiftKey || key === 'ArrowUp' || key === 'ArrowLeft' ? -1 : 1;
				do { this.focus = (this.focus + dir + n) % n; } while (!this.panelFocus[this.focus].parent.visible && this.panelFocus.some(item => item.parent.visible));
				this.pauseButtons.forEach((b, i) => b.hover(i === this.focus));
			}
			if (key === 'Enter' || key === 'Space') {
				if (this.focus >= 0) this.pauseButtons[this.focus]?.trigger();
				else if (!this.finished) this.pause(false);
			}
			if (key === 'KeyR') this.start(this.mode);
			return;
		}
		if (key === 'ArrowLeft' || key === 'ArrowRight' || key === 'ArrowDown')
			this.held.set(key, { next: performance.now() + this.settings.das });
		this.input(key, e.ctrlKey);
	};
	private keyUp = (e: KeyboardEvent) => {
		this.held.delete(e.code);
	};
	private input(key: string, ctrl = false) {
		let changed = false;
		switch (key) {
			case 'ArrowLeft':
				changed = this.game.move(-1);
				break;
			case 'ArrowRight':
				changed = this.game.move(1);
				break;
			case 'ArrowDown':
				changed = this.game.move(0, 1);
				break;
			case 'ArrowUp':
			case 'KeyX':
				changed = this.game.rotate();
				break;
			case 'KeyZ':
				changed = ctrl && this.mode === 'zen' ? this.game.undo() : this.game.rotate(-1);
				break;
			case 'KeyC':
			case 'ShiftLeft':
			case 'ShiftRight':
				this.game.hold();
				changed = true;
				break;
			case 'Space': {
				const cleared = this.game.drop();
				this.sound.play(cleared ? 'clear' : 'drop');
				this.visual.drop = 1;
				this.motion.to(this.visual, { drop: 0 }, 350, ease.outQuint);
				changed = true;
				this.gravity = 0;
				break;
			}
		}
		if (changed) {
			this.dirty = true;
			if (key !== 'Space') this.sound.play('move');
		}
		this.checkEnd();
	}
	private checkEnd() {
		if (
			!this.finished &&
			(this.game.state.over || (this.mode === 'sprint' && this.game.state.lines >= 40))
		) {
			this.finished = true;
			this.saveRecord();
			this.pause(true);
		}
	}
	private drawGame() {
		this.dirty = false;
		const g = this.blocks.clear(),
			p = this.previews.clear();
		const cell = 26,
			bx = -130,
			by = -222;
		this.board
			.clear()
			.rect(bx - 2, by - 2, 264, 524)
			.fill({ color: 0x202428, alpha: 0.92 })
			.rect(bx, by, 260, 520)
			.stroke({ color: 0x899198, alpha: 0.3, width: 1 });
		if (this.settings.grid) {
			for (let x = 1; x < 10; x++)
				this.board
					.moveTo(bx + x * cell, by)
					.lineTo(bx + x * cell, by + 520)
					.stroke({ color: 0xffffff, alpha: 0.05, width: 1 });
			for (let y = 1; y < 20; y++)
				this.board
					.moveTo(bx, by + y * cell)
					.lineTo(bx + 260, by + y * cell)
					.stroke({ color: 0xffffff, alpha: 0.05, width: 1 });
		}
		const block = (
			target: Graphics,
			x: number,
			y: number,
			color: string,
			size = cell,
			ghost = false
		) => {
			if (ghost)
				target.rect(x + 2, y + 2, size - 4, size - 4).stroke({ color, width: 1, alpha: 0.48 });
			else
				target
					.rect(x + 1, y + 1, size - 2, size - 2)
					.fill(color)
					.rect(x + 2, y + 1, size - 4, 2)
					.fill({ color: 0xffffff, alpha: 0.3 })
					.rect(x + 2, y + size - 4, size - 4, 3)
					.fill({ color: 0x000000, alpha: 0.14 });
		};
		this.game.state.board.forEach((row, y) =>
			row.forEach((v, x) => {
				if (v) block(g, bx + x * cell, by + y * cell, colors[v]);
			})
		);
		const s = this.game.state;
		if (!s.over) {
			const ghost = this.game.ghostY();
			s.matrix.forEach((row, y) =>
				row.forEach((v, x) => {
					if (!v) return;
					if (this.settings.ghost && ghost + y >= 0)
						block(g, bx + (s.x + x) * cell, by + (ghost + y) * cell, colors[s.piece], cell, true);
					if (s.y + y >= 0) block(g, bx + (s.x + x) * cell, by + (s.y + y) * cell, colors[s.piece]);
				})
			);
		}
		const mini = (piece: Piece, x: number, y: number) =>
			shapes[piece].forEach((r, dy) =>
				r.forEach((v, dx) => {
					if (v) block(p, x + dx * 21, y + dy * 21, colors[piece], 21);
				})
			);
		if (s.hold) mini(s.hold, -275, -181);
		s.queue.slice(0, 5).forEach((piece, i) => mini(piece, 181, -182 + i * 82));
		this.linesText.text = this.mode === 'sprint' ? `${s.lines}/40` : `${s.lines}`;
		this.piecesText.text = String(s.placed);
	}
	private resize = () => {
		if (!this.initialized || this.disposed) return;
		const scale = Math.min(this.host.clientHeight / 720, this.host.clientWidth / 960);
		if (scale <= 0) return;
		this.width = this.host.clientWidth / scale;
		this.height = this.host.clientHeight / scale;
		this.stage.scale.set(scale);
		this.toolbarUI.resize(this.width);
		this.bottom.position.set(this.width / 2, this.height - 50);
		this.notice.position.set(this.width / 2, this.height - 28);
		this.shade.clear().rect(0, 0, this.width, this.height).fill(0x040915);
		this.backdrop.clear().rect(0, 0, this.width, this.height).fill(this.theme.background);
		this.ambient.dark = this.themeTransition.value;
		this.ambient.resize(this.width, this.height);
		for (const text of [this.titleText, this.linesText, this.timeText, this.piecesText]) text.style.fill = this.theme.text;
		this.modeDescription.style.fill = this.theme.muted;
		if (this.pauseControl) this.pauseControl.tint = this.theme.text;
		if (this.panel) this.buildDrawer();
		if (this.paused) this.buildPause();
		this.dirty = true;
	};
	private tick = () => {
		const now = performance.now(),
			elapsed = (now - this.lastTick) / 1000,
			dt = Math.min(elapsed, 0.05);
		this.lastTick = now;
		this.motion.update(now);
		this.updateTheme();
		if (this.panel) for (const draw of this.toggleDraws) draw();
		if (this.panel === 'settings') {
			const offset = this.settingsScroll.offset;
			this.settingsRows.y = -offset;
			const section =
				offset >= this.settingsScroll.max - 1 && offset > 0 ? 2 : offset >= 365 ? 1 : 0;
			this.settingsSelection.y = 117 + section * 64;
			const height = this.height - 124;
			const thumb = height * Math.min(1, height / this.settingsContentHeight);
			this.settingsScrollbar.clear();
			if (this.settingsScroll.max > 0)
				this.settingsScrollbar
					.roundRect(0, 116 + ((height - thumb) * offset) / this.settingsScroll.max, 3, thumb, 1.5)
					.fill({ color: this.theme.accent, alpha: 0.45 });
		}
		this.sound.volume = this.settings.volume;
		const v = this.visual,
			w = this.width,
			h = this.height;
		const factor = 1 - Math.exp(-dt * 5);
		const motion = this.motion.reduced ? 0 : 1;
		this.parallax.x += (this.pointer.x * 14 * motion - this.parallax.x) * factor;
		this.parallax.y += (this.pointer.y * 10 * motion - this.parallax.y) * factor;
		this.shade.alpha = v.dim;
		this.menu.alpha = v.menuAlpha;
		this.menu.x = v.menuX;
		this.menu.visible = v.menuAlpha > 0.001;
		this.menu.eventMode = this.screen === 'menu' && !this.panel ? 'auto' : 'none';
		const menuWidth = Math.min(720, w - 80);
		const rowHeight = Math.min(106, (h - 160) / 4.5);
		this.band.clear();
		this.ribbon.alpha = 1;
		this.ribbon.scale.set(1);
		this.ribbon.position.set(w - menuWidth - 40, 40 + (h - 40) / 2);
		for (const group of [this.oldButtons, this.buttons]) {
			const heights = group.map((b) =>
				Math.max(0, rowHeight * b.visual.width * (1 + b.visual.hover * 0.5))
			);
			let y = -heights.reduce((sum, height) => sum + height, 0) / 2;
			group.forEach((b, i) => {
				b.root.position.set(0, y);
				b.draw(menuWidth, heights[i]);
				y += heights[i];
			});
		}
		this.ambient.view.visible = v.menuAlpha > 0.001;
		this.ambient.view.alpha = v.menuAlpha;
		if (this.ambient.view.visible) this.ambient.tick(dt, this.motion.reduced);
		this.toolbar.alpha = v.toolbarAlpha;
		for (const child of this.toolbar.children.slice(1)) child.tint = 0xffffff;
		this.toolbar.y = -40 * (1 - v.toolbar);
		this.toolbar.visible = v.toolbar > 0.001;
		this.toolbar.eventMode = this.panel || this.screen === 'game' ? 'none' : 'auto';
		this.bottom.alpha = v.bottom;
		const gameScale = Math.min((w - 48) / 650, (h - 32) / 650);
		this.gameView.scale.set(gameScale);
		this.gameView.position.set(w / 2 + v.gameX, h / 2 - 16 * gameScale + v.drop * 3 * motion);
		this.gameView.alpha = v.gameAlpha;
		this.gameView.visible = v.gameAlpha > 0.001;
		this.gameView.eventMode =
			this.screen === 'game' && !this.paused && !this.panel ? 'auto' : 'none';
		this.pauseView.alpha = v.pause;
		this.pauseView.visible = v.pause > 0.001;
		this.pauseView.eventMode = this.paused && !this.panel ? 'auto' : 'none';
		this.sessionMenu?.layout(w, h, v.pause, this.themeTransition.value);
		this.drawer.visible = v.drawer > 0.001;
		this.drawer.alpha = 1;
		this.drawerShade.alpha = v.drawer;
		this.drawerContent.x = -740 * (1 - v.drawer);
		const bounds = this.host.getBoundingClientRect();
		this.settingsSearch?.layout(bounds.left + (this.drawerContent.x + 190) * this.stage.scale.x, bounds.top + 40 * this.stage.scale.y, (Math.min(700, w - 36) - 235) * this.stage.scale.x, this.stage.scale.y, this.theme.text, this.panel === 'settings');
		this.drawer.eventMode = this.panel ? 'auto' : 'none';
		this.dropGlow.clear();
		if (v.drop > 0.01) this.dropGlow.rect(-130, 294, 260, 4).fill({ color: menuColors.solo, alpha: v.drop });
		if (this.screen === 'game' && !this.paused && !this.panel && !this.finished) {
			this.time += elapsed;
			for (const [key, held] of this.held) {
				if (now >= held.next) {
					this.input(key);
					held.next = now + (key === 'ArrowDown' ? 25 : this.settings.arr);
				}
			}
			if (this.mode === 'sprint' || this.settings.gravity) {
				this.gravity += dt;
				if (this.gravity >= 0.8) {
					this.gravity = 0;
					if (!this.game.move(0, 1)) this.game.lock();
					this.dirty = true;
					this.checkEnd();
				}
			}
		}
		if (this.dirty) this.drawGame();
		if (now - this.lastClock > 80) {
			this.timeText.text = formatTime(this.time);
			this.toolbarUI.tick(now);
			this.lastClock = now;
		}
	};
	destroy() {
		this.settingsSearch?.destroy();
		this.disposed = true;
		this.observer?.disconnect();
		this.motion.clear();
		this.sound.destroy();
		this.host.removeEventListener('keydown', this.keyDown);
		window.removeEventListener('keyup', this.keyUp);
		window.removeEventListener('pointerup', this.endDrag);
		window.removeEventListener('blur', this.blur);
		document.removeEventListener('visibilitychange', this.visibility);
		this.host.removeEventListener('pointermove', this.pointerMove);
		this.host.removeEventListener('pointerdown', this.pointerDown);
		this.media.removeEventListener('change', this.motionPreference);
		if (this.initialized) {
			this.app.ticker.remove(this.tick);
			this.app.destroy(true, { children: true });
		}
	}
}
