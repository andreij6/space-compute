#!/usr/bin/env node
import { execFileSync, execSync } from 'node:child_process';
import { readFileSync, mkdirSync } from 'node:fs';
import { chromium } from 'playwright';
import { playAudit } from 'playwright-lighthouse';

const repoRoot = new URL('../../..', import.meta.url).pathname;
const outDir = `${repoRoot}docs/demos/T9.5`;
const THRESHOLDS = { accessibility: 90, performance: 80 };
mkdirSync(outDir, { recursive: true });

function icp(args) {
  return execFileSync('icp', args, { cwd: repoRoot, stdio: 'pipe' }).toString().trim();
}

function localFrontendUrl() {
  const status = JSON.parse(execSync('icp network status -e local --json', { cwd: repoRoot, encoding: 'utf8' }));
  const ids = JSON.parse(readFileSync(`${repoRoot}.icp/cache/mappings/local.ids.json`, 'utf8'));
  const gateway = new URL(status.gateway_url);
  return `${gateway.protocol}//${ids.frontend}.localhost:${gateway.port}`;
}

const baseURL = localFrontendUrl();
const PORT = 9223;

async function signInAndSpawn(page, context, label) {
  await page.goto(`${baseURL}/signin`);
  const [ii] = await Promise.all([
    context.waitForEvent('page'),
    page.getByRole('button', { name: 'Sign in with Internet Identity' }).click(),
  ]);
  ii.on('dialog', (d) => d.accept(String(Date.now() % 1_000_000)));
  await ii.waitForURL(/id\.ai\.localhost/);
  const create = ii.getByRole('button', { name: /^Create/ }).first();
  const createWithPasskey = ii.getByRole('button', { name: 'Create with passkey' });
  await create.waitFor({ state: 'visible' });
  if (!(await createWithPasskey.isVisible())) await create.click();
  await createWithPasskey.click();
  await ii.getByRole('textbox').fill(`${label}-${Date.now()}`);
  await ii.getByRole('button', { name: 'Create identity' }).click();
  await ii.getByRole('button', { name: 'Continue', exact: true }).click({ timeout: 30_000 });
  await page.waitForURL(/\/spawn$/, { timeout: 30_000 });

  const name = `Lighthouse-${Date.now() % 100_000}`;
  await page.getByLabel('Agent name').fill(name);
  const continueButton = page.getByRole('button', { name: 'Continue' });
  await continueButton.waitFor({ state: 'visible' });
  await continueButton.click();

  const accountInput = page.getByLabel('Send ICP to this account');
  await accountInput.waitFor({ state: 'visible' });
  const accountId = await accountInput.inputValue();
  icp(['token', 'transfer', '1', accountId, '-e', 'local', '--identity', 'sc-user']);

  await page.getByRole('button', { name: "I've sent it" }).click();
  await page.getByRole('status').filter({ hasText: 'Done.' }).waitFor({ timeout: 60_000 });
  await page.getByRole('button', { name: 'Go to dashboard' }).click();
  await page.waitForURL(/\/dashboard$/, { timeout: 30_000 });
}

async function audit(page, path, name) {
  await page.goto(`${baseURL}${path}`);
  await page.waitForLoadState('networkidle');
  const result = await playAudit({
    page,
    port: PORT,
    thresholds: THRESHOLDS,
    ignoreError: true,
    reports: { formats: { html: true, json: true }, directory: outDir, name },
  });
  const scores = Object.fromEntries(
    Object.keys(THRESHOLDS).map((k) => [k, Math.round(result.lhr.categories[k].score * 100)]),
  );
  console.log(`${name}: ${JSON.stringify(scores)}`);
  return scores;
}

async function firstDiscoveryPath(page) {
  await page.goto(`${baseURL}/discoveries`);
  await page.waitForLoadState('networkidle');
  const href = await page.locator('main a[href^="/d/"]').first().getAttribute('href', { timeout: 5_000 }).catch(() => null);
  return href ?? '/d/SC-0000-000000';
}

async function main() {
  const browser = await chromium.launch({ args: [`--remote-debugging-port=${PORT}`] });
  const context = await browser.newContext();

  const results = {};

  const landing = await context.newPage();
  results.landing = await audit(landing, '/', 'landing');
  await landing.close();

  const detail = await context.newPage();
  results.discoveryDetail = await audit(detail, await firstDiscoveryPath(detail), 'discovery-detail');
  await detail.close();

  const dashboardPage = await context.newPage();
  await signInAndSpawn(dashboardPage, context, 'e2e-lighthouse');
  results.dashboard = await audit(dashboardPage, '/dashboard', 'dashboard');
  await dashboardPage.close();

  await browser.close();

  console.log(JSON.stringify(results, null, 2));
  const failed = Object.entries(results).flatMap(([pageName, scores]) =>
    Object.entries(THRESHOLDS)
      .filter(([k, min]) => scores[k] < min)
      .map(([k]) => `${pageName}.${k}=${scores[k]}`),
  );
  if (failed.length > 0) {
    console.error(`Below threshold: ${failed.join(', ')}`);
    process.exit(1);
  }
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
