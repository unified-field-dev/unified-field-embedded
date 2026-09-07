import { expect, type Page } from "@playwright/test";

type BootState = "ready" | "error" | "loading";

async function readBootState(page: Page): Promise<BootState> {
  return page.evaluate(() => {
    const html = document.documentElement;
    if (html.getAttribute("data-orbital-boot-state") === "error") {
      return "error";
    }
    if (html.getAttribute("data-orbital-hydrated") === "true") {
      return "ready";
    }
    return "loading";
  });
}

/**
 * Wait until Orbital marks the document hydrated (`data-orbital-hydrated=true`).
 * TM-01 / TM-04: hydrate is mandatory before surface asserts.
 */
export async function waitForHydrated(
  page: Page,
  timeoutMs = 240_000,
): Promise<void> {
  await expect
    .poll(async () => readBootState(page), { timeout: timeoutMs })
    .toBe("ready");
  const overlay = page.getByTestId("orbital-boot-overlay");
  if ((await overlay.count()) > 0) {
    await expect(overlay).toHaveCount(0, { timeout: 60_000 });
  }
}

/**
 * Navigate and wait for Orbital hydrate (TM-01 / TM-04).
 * On boot error or stuck loading, reload once, then require hydrate again.
 * Auth-gate asserts belong in specs after this returns — not as a hydrate substitute.
 */
export async function gotoHydrated(
  page: Page,
  path: string,
  timeoutMs = 240_000,
): Promise<void> {
  await page.goto(path, { waitUntil: "domcontentloaded" });
  try {
    await waitForHydrated(page, timeoutMs);
  } catch (first) {
    await page.reload({ waitUntil: "domcontentloaded" });
    try {
      await waitForHydrated(page, Math.min(timeoutMs, 120_000));
    } catch (second) {
      throw second instanceof Error ? second : first;
    }
  }
}
