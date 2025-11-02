import { test, expect } from '@playwright/test';

test.use({
	viewport: {
		height: 1080,
		width: 1920
	}
});

test('test', async ({ page }) => {
	const url = "http://localhost:3000"

	await page.goto(`${url}`);
	await page.getByTestId('add-catering').click();
	await page.getByTestId('catering-save').click();
	await expect(page.getByTestId('snackbar-root')).toContainText('Podano nieprawidłowy czas lub datę');
	await page.getByTestId('catering-start').fill('2025-10-08');
	await expect(page.getByTestId('snackbar-root')).toContainText('Podano nieprawidłowy czas lub datę');
	await page.getByTestId('catering-start').fill('2025-01-01');
	await page.getByTestId('catering-end').fill('2024-09-11');
	await expect(page.getByTestId('snackbar-root')).toContainText('Podano nieprawidłowy czas lub datę');
	await page.getByTestId('catering-end').fill('2025-11-11');
	await page.getByTestId('catering-save').click();
	await page.getByTestId('catering-cancellation').click();
	await page.getByTestId('catering-cancellation').press('ArrowUp');
	await page.getByTestId('catering-cancellation').fill('06:30');
	await page.getByTestId('catering-save').click();
	await expect(page.getByTestId('snackbar-root')).toContainText('Nie udało się stworzyć cateringu error deserializing server function arguments: missing field `meals`');
	await page.getByRole('textbox', { name: 'Posiłki' }).click();
	await page.getByRole('textbox', { name: 'Posiłki' }).fill('Śniadanie');
	await page.getByRole('textbox', { name: 'Posiłki' }).press('Enter');
	await page.getByRole('textbox', { name: 'Posiłki' }).fill('Obiad');
	await page.getByRole('textbox', { name: 'Posiłki' }).press('Enter');
	await page.getByRole('textbox', { name: 'Posiłki' }).fill('Podwieczorek');
	await page.getByRole('textbox', { name: 'Posiłki' }).press('Enter');
	await page.getByRole('textbox', { name: 'Posiłki' }).fill('Kolacja');
	await page.getByRole('textbox', { name: 'Posiłki' }).press('Enter');
	await page.getByRole('button').nth(4).click();
	await page.getByTestId('catering-name').click();
	await page.getByTestId('catering-name').fill('Test Catering no. 1');
	await page.getByTestId('catering-save').click();
	await expect(page.getByTestId('snackbar-root')).toContainText('Nie udało się stworzyć cateringu error running server function: Catering needs to specify at least one day of week');
	await page.getByTestId('dow-Mon').click();
	await page.getByTestId('dow-Tue').click();
	await page.getByTestId('dow-Wed').click();
	await page.getByTestId('dow-Thu').click();
	await page.getByTestId('dow-Fri').click();
	await page.getByTestId('catering-save').click();
	await expect(page.getByTestId('snackbar-root')).toContainText('Dodano nowy catering');

	const attendanceExtractor = /attendance\/(.*)\/(.*)\/(.*)/;

	await page.getByTestId("group-tree").getByText("test catering no. 1").click();

	const [_, target, year, month] = attendanceExtractor.exec(page.url()) ?? [];

	await expect(page.getByTestId("group-tree")).toContainText('test catering no. 1');

	await page.goto(`${url}/attendance/${target}/2025/10`);

	await expect(page.getByTestId('meal-name-śniadanie-2025-10-01')).toContainText('śniadanie');
	await expect(page.getByTestId('meal-name-obiad-2025-10-01')).toContainText('obiad');
	await expect(page.getByTestId('meal-name-podwieczorek-2025-10-01')).toContainText('podwieczorek');
	await expect(page.getByTestId('meal-count-śniadanie-2025-10-01')).toContainText('0');
	await expect(page.getByTestId('meal-count-obiad-2025-10-01')).toContainText('0');
	await expect(page.getByTestId('meal-count-podwieczorek-2025-10-01')).toContainText('0');
	await page.getByRole('button').nth(2).click();
	await page.getByRole('textbox', { name: 'Nazwa' }).click();
	await page.getByRole('textbox', { name: 'Nazwa' }).fill('Grupa 1');
	await page.getByTestId('add-group-save').click();
	await expect(page.getByTestId('snackbar-root')).toContainText('Dodano grupę');
	await page.getByTestId(`tree-expand-button-${target}`).click();

	await page.getByTestId('group-tree').getByText("grupa 1").click();

	await page.getByRole('button').nth(3).click();
	await page.getByRole('textbox', { name: 'Imię' }).click();
	await page.getByRole('textbox', { name: 'Imię' }).fill('Jan');
	await page.getByRole('textbox', { name: 'Imię' }).press('Tab');
	await page.getByRole('textbox', { name: 'Nazwisko' }).fill('Nowak');
	await page.getByRole('textbox', { name: 'Nazwisko' }).press('Tab');
	await page.locator('#alergie').press('Tab');
	await page.locator('#guardians').fill('Anna Nowak');
	await page.locator('#guardians').press('Enter');
	await page.locator('#guardians').fill('Jan Nowak');
	await page.locator('#guardians').press('Enter');
	await page.getByRole('textbox', { name: 'Imię' }).click();
	await page.getByRole('textbox', { name: 'Imię' }).fill('John');
	await page.getByRole('button', { name: 'Dodaj', exact: true }).click();
	await expect(page.getByTestId('snackbar-root')).toContainText('Dodano ucznia');
	await expect(page.getByTestId('meal-count-śniadanie-2025-10-01')).toContainText('1');
	await expect(page.getByTestId('meal-count-obiad-2025-10-01')).toContainText('1');
	await expect(page.getByTestId('meal-count-podwieczorek-2025-10-01')).toContainText('1');
});
