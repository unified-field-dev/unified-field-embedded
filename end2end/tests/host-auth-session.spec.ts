import { test, expect } from "@playwright/test";
import {
  assertAnonymousShell,
  assertAuthenticatedMenu,
  assertAuthenticatedShell,
  seedVerifiedUser,
  signIn,
  signOut,
} from "./support/auth";
import { gotoHydrated } from "./support/hydration";

const hasSeedToken = Boolean(process.env.UF_E2E_SEED_TOKEN?.trim());

const authenticatedRoutes = [
  ["/valence", "valence-app-root"],
  ["/chronon", "chronon-app-root"],
  ["/photon", "photon-app-root"],
  ["/spectra", "spectra-app-root"],
  ["/boson", "boson-app-root"],
  ["/permission", "permission-app-root"],
  ["/secrets", "neutrino-app-root"],
  ["/tag", "tag-app-root"],
] as const;

test.describe("EMBED-AUTH host session", () => {
  test.beforeEach(() => {
    test.skip(!hasSeedToken, "UF_E2E_SEED_TOKEN required for auth session probes");
  });

  test("EMBED-AUTH-01-session-propagates-across-apps", async ({
    page,
    request,
  }) => {
    const credentials = await seedVerifiedUser(request);
    await signIn(page, credentials, "/welcome");
    await assertAuthenticatedMenu(page);

    for (const path of ["/welcome", "/apps", "/user/account-settings"]) {
      await gotoHydrated(page, path);
      await assertAuthenticatedShell(page);
      await expect(page.getByTestId("auth-required-empty-state")).toHaveCount(0);
    }

    for (const [path, ownerRoot] of authenticatedRoutes) {
      await gotoHydrated(page, path);
      await assertAuthenticatedShell(page);
      await expect(page.getByTestId(ownerRoot)).toBeAttached({
        timeout: 60_000,
      });
      await expect(page.getByTestId("auth-required-empty-state")).toHaveCount(0);
    }
  });

  test("EMBED-AUTH-02-invalid-signin-stays-anonymous", async ({ page }) => {
    await gotoHydrated(page, "/auth/signin");
    await page
      .getByTestId("signin-email")
      .locator('input[name="email"]')
      .fill("not-seeded-user@example.test");
    await page
      .getByTestId("signin-password")
      .locator('input[name="password"]')
      .fill("WrongPassword1!");
    await page.getByTestId("signin-submit").getByRole("button").click();
    await expect(page.getByTestId("signin-error")).toBeVisible({
      timeout: 60_000,
    });

    await gotoHydrated(page, "/welcome");
    await assertAnonymousShell(page);
    await gotoHydrated(page, "/");
    await expect(page).toHaveURL(/\/apps(?:\/|$)/, { timeout: 60_000 });
    await assertAnonymousShell(page);
  });

  test("EMBED-AUTH-10-logout-denies-all-app-families", async ({
    page,
    request,
  }) => {
    const credentials = await seedVerifiedUser(request);
    await signIn(page, credentials, "/photon");
    await assertAuthenticatedShell(page);
    await signOut(page);

    // Mirror PLATFORM_ROOTS: secrets/tag gate outside layout (no owner root).
    for (const [path, ownerRoot, guardOutsideLayout] of [
      ["/photon", "photon-app-root", false],
      ["/permission", "permission-app-root", false],
      ["/secrets", "neutrino-app-root", true],
      ["/tag", "tag-app-root", true],
    ] as const) {
      await gotoHydrated(page, path);
      await assertAnonymousShell(page);
      await expect(page.getByTestId("auth-required-empty-state")).toBeAttached({
        timeout: 60_000,
      });
      if (guardOutsideLayout) {
        await expect(page.getByTestId(ownerRoot)).toHaveCount(0);
      } else {
        await expect(page.getByTestId(ownerRoot)).toBeAttached({
          timeout: 60_000,
        });
      }
    }

    await gotoHydrated(page, "/user/account-settings");
    await expect(page).toHaveURL(/\/auth\/signin/, { timeout: 60_000 });
  });

  test("E2E-SEED-01-invalid-token-contained", async ({ request }) => {
    const response = await request.post("/api/test/seed-data", {
      headers: { Authorization: "Bearer definitely-wrong" },
      data: {
        scenario: "auth_basic_user",
        email: "rejected-seed@example.test",
        password: "CorrectHorseBattery1!",
      },
    });
    expect(response.status()).toBe(401);
    expect(await response.text()).toBe("unauthorized");
  });
});
