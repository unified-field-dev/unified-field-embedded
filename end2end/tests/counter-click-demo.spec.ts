import { test, expect } from "@playwright/test";

/**
 * Slim embedded-host demo: open counter, click once, assert the global count increases.
 * Full counter suite lives in unified-field-dev/counter-app.
 */
test("counter click demo", async ({ page }) => {
  await page.goto("/counter");
  await expect(page.getByTestId("counter-container")).toBeVisible({ timeout: 60_000 });

  const global = page.getByTestId("global-counter");
  await expect(global).toBeVisible({ timeout: 30_000 });

  const parseGlobal = async (): Promise<number> => {
    const text = (await global.textContent()) ?? "";
    const m = text.match(/Global count:\s*(\d+)/i);
    if (!m) throw new Error(`Could not parse global count from: ${JSON.stringify(text)}`);
    return parseInt(m[1], 10);
  };

  const inc = page.getByTestId("increment-button").locator("button");
  await expect(inc).toBeVisible({ timeout: 45_000 });

  // Wait out SSR/hydrate 0-flash: metrics and button must agree on a stable value.
  let initial = -1;
  await expect
    .poll(async () => {
      const g1 = await parseGlobal();
      const label = (await inc.textContent()) ?? "";
      if (!label.includes(`Click Me: ${g1}`)) return false;
      await page.waitForTimeout(250);
      const g2 = await parseGlobal();
      if (g1 !== g2) return false;
      initial = g2;
      return true;
    }, {
      message: "counter UI did not settle after load",
      timeout: 30_000,
    })
    .toBe(true);

  await inc.click();

  // Accept any increase (other clients/tests may race the shared global counter).
  await expect
    .poll(parseGlobal, {
      message: "global counter did not increment after click",
      timeout: 30_000,
    })
    .toBeGreaterThan(initial);
});
