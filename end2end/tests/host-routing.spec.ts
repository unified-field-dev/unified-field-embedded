import { test, expect } from "@playwright/test";
import { assertSignInSurface } from "./support/auth";
import { gotoHydrated } from "./support/hydration";

test.describe("EMBED-RT host routing contracts", () => {
  test("EMBED-RT-20-redirect-contracts", async ({ page }) => {
    await gotoHydrated(page, "/");
    await expect(page).toHaveURL(/\/apps(?:\/|$)/, { timeout: 60_000 });
    const appsSurface = page
      .getByTestId("apps-launcher-search")
      .or(page.getByTestId("auth-required-empty-state"))
      .or(page.getByRole("dialog").filter({ hasText: /Sign in required/i }));
    await expect(appsSurface.first()).toBeAttached({ timeout: 60_000 });

    await gotoHydrated(page, "/auth");
    await expect(page).toHaveURL(/\/auth\/signin/, { timeout: 60_000 });
    await assertSignInSurface(page);

    await gotoHydrated(page, "/user");
    await expect(page).toHaveURL(
      /\/user\/account-settings|\/auth\/signin/,
      { timeout: 60_000 },
    );
  });

  test("EMBED-RT-50-unknown-path-not-found", async ({ page }) => {
    await gotoHydrated(page, "/no-such-route-xyz");
    await expect(page.getByTestId("unified-field-not-found-page")).toBeVisible({
      timeout: 60_000,
    });
    await expect(page.getByTestId("unified-field-coming-soon-page")).toHaveCount(
      0,
    );
  });

  test("EMBED-RT-51-coming-soon-is-not-fallback", async ({ page }) => {
    await gotoHydrated(page, "/coming-soon");
    await expect(page.getByTestId("unified-field-not-found-page")).toBeVisible({
      timeout: 60_000,
    });
    await expect(page.getByTestId("unified-field-coming-soon-page")).toHaveCount(
      0,
    );
  });
});
