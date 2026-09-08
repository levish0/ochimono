import { expect, test } from '@playwright/test';

test.use({ locale: 'en-US' });

test('loads WASM and plays, undoes, pauses and saves a Zen session', async ({ page }) => {
	const errors: string[] = [];
	page.on('pageerror', (error) => errors.push(error.message));
	const wasm = page.waitForResponse((response) => response.url().endsWith('.wasm'));
	await page.goto('/');
	expect((await wasm).ok()).toBe(true);
	await expect(page.getByRole('status')).toHaveCount(0);
	const host = page.getByRole('application');
	await expect(host.locator('canvas')).toBeVisible();
	await host.focus();
	await page.keyboard.press('Enter');
	await page.keyboard.press('p');
	await page.keyboard.press('Enter');
	await expect(page.locator('#game-status')).toContainText('Zen started');
	await page.keyboard.press('ArrowLeft');
	await page.keyboard.press('x');
	await page.keyboard.press('Space');
	await page.keyboard.press('Control+z');
	await page.keyboard.press('Space');
	await page.keyboard.press('Escape');
	await expect(page.locator('#game-status')).toContainText('Paused');
	await page.keyboard.press('Escape');
	await expect(page.locator('#game-status')).toContainText('Resumed');
	await page.keyboard.press('Escape');
	for (let i = 0; i < 3; i++) await page.keyboard.press('ArrowDown');
	await page.keyboard.press('Enter');
	const records = await page.evaluate(() =>
		JSON.parse(localStorage.getItem('ochimono.records.v1') ?? '[]')
	);
	expect(records).toHaveLength(1);
	expect(records[0]).toMatchObject({ mode: 'zen', pieces: 1, lines: 0 });
	expect(records[0].seconds).toBeGreaterThanOrEqual(0);
	expect(errors).toEqual([]);
});
