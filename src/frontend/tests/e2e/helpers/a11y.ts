import AxeBuilder from '@axe-core/playwright';
import { expect, type Page } from '@playwright/test';

export function trackConsoleErrors(page: Page): string[] {
  const errors: string[] = [];
  page.on('pageerror', (err) => errors.push(String(err)));
  page.on('console', (msg) => {
    if (msg.type() === 'error') errors.push(msg.text());
  });
  return errors;
}

export async function expectAccessible(page: Page, errors: string[]): Promise<void> {
  expect(errors, `console errors on ${page.url()}`).toEqual([]);
  const results = await new AxeBuilder({ page }).analyze();
  const serious = results.violations.filter((v) => v.impact === 'serious' || v.impact === 'critical');
  expect(serious, `axe serious/critical violations on ${page.url()}: ${JSON.stringify(serious, null, 2)}`).toEqual([]);
}

export async function visitAndCheck(page: Page, path: string): Promise<void> {
  const errors = trackConsoleErrors(page);
  await page.goto(path);
  await page.waitForLoadState('networkidle');
  await expectAccessible(page, errors);
}
