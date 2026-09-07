import { test, expect } from "@playwright/test";
import { gotoHydrated } from "./support/hydration";

/**
 * L5 composition smoke: PhotonRoutes mount under /photon.
 * Full ops catalog (admin KPIs, topics, events) lives in photon-uf-app-e2e.
 * Anonymous visitors should still see the Photon shell chrome and auth gate.
 */
test("e2e.l5.photon_shell_smoke", async ({ page }) => {
  await gotoHydrated(page, "/photon");
  await expect(page.getByTestId("photon-app-root")).toBeAttached({
    timeout: 60_000,
  });
  // Compact nav can stay aria-hidden behind the auth dialog; attachment is enough.
  await expect(page.getByTestId("nav-photon-dashboard")).toBeAttached({
    timeout: 30_000,
  });
  await expect(page.getByTestId("nav-photon-topics")).toBeAttached();
  await expect(page.getByTestId("nav-photon-subscriptions")).toBeAttached();
  await expect(page.getByTestId("nav-photon-events")).toBeAttached();
  await expect(page.getByTestId("photon-dashboard")).toHaveCount(0);
  await expect(page.getByTestId("auth-required-empty-state")).toBeAttached({
    timeout: 60_000,
  });
});
