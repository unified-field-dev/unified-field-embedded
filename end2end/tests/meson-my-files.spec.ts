//! Validating My Files scenarios:
//! - meson-my-files-list-happy (empty list for fresh seed user)
//! - meson-my-files-anonymous-gate-sad (anonymous auth gate; no list surface)
//!
//! Peer IDOR is covered by L1 meson integration (`meson-my-files-idor-sad`), not this UI gate.

import { test, expect } from "@playwright/test";
import {
  assertAnonymousShell,
  assertAuthenticatedShell,
  seedVerifiedUser,
  signIn,
} from "./support/auth";
import { gotoHydrated } from "./support/hydration";

const hasSeedToken = Boolean(process.env.UF_E2E_SEED_TOKEN?.trim());

test.describe("Meson My Files", () => {
  test.beforeEach(() => {
    test.skip(!hasSeedToken, "UF_E2E_SEED_TOKEN required for Meson My Files probes");
  });

  test("meson-my-files-list-happy", async ({ page, request }) => {
    const credentials = await seedVerifiedUser(request);
    await signIn(page, credentials, "/meson");
    await assertAuthenticatedShell(page);

    await expect(page.getByTestId("meson-app-root")).toBeAttached({
      timeout: 60_000,
    });
    await expect(page.getByTestId("meson-my-files")).toBeAttached({
      timeout: 60_000,
    });
    await expect(page.getByTestId("auth-required-empty-state")).toHaveCount(0);

    // Fresh seed user has no uploads — EmptyState is the validating outcome.
    await expect(page.getByText("No files yet")).toBeVisible({
      timeout: 60_000,
    });
    await expect(page.getByText(/Showing 0 files/)).toBeVisible({
      timeout: 30_000,
    });
  });

  test("meson-my-files-anonymous-gate-sad", async ({ page }) => {
    await gotoHydrated(page, "/meson");
    await assertAnonymousShell(page);
    await expect(page.getByTestId("auth-required-empty-state")).toBeAttached({
      timeout: 60_000,
    });
    // List surface must not render for anonymous visitors.
    await expect(page.getByTestId("meson-my-files")).toHaveCount(0);
  });
});
