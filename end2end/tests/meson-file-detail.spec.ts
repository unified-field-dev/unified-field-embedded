//! Validating File Detail scenarios:
//! - meson-file-detail-image-preview-happy
//! - meson-file-detail-not-found-sad
//! - meson-file-detail-peer-id-sad
//! - meson-file-detail-anonymous-gate-sad
//!
//! Text/unsupported preview modes are covered by meson-app's Layer-1
//! render tests (no browser needed); this file only covers what requires a
//! real browser + real auth: the image branch, the IDOR-safe not-found
//! collapse (both missing and foreign ids), and the anonymous deep-link gate.

import { test, expect } from "@playwright/test";
import {
  assertAnonymousShell,
  assertAuthenticatedShell,
  seedVerifiedUser,
  signIn,
} from "./support/auth";
import { gotoHydrated } from "./support/hydration";

const hasSeedToken = Boolean(process.env.UF_E2E_SEED_TOKEN?.trim());

/** Boson + promote can take minutes on a cold host. */
const SCAN_TIMEOUT_MS = 300_000;

const TINY_PNG = Buffer.from([
  0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49,
  0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02,
  0x00, 0x00, 0x00, 0x90, 0x77, 0x53, 0xde, 0x00, 0x00, 0x00, 0x0c, 0x49, 0x44,
  0x41, 0x54, 0x08, 0xd7, 0x63, 0xf8, 0xcf, 0xc0, 0x00, 0x00, 0x00, 0x03, 0x00,
  0x01, 0x00, 0x05, 0xfe, 0xd4, 0xef, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e,
  0x44, 0xae, 0x42, 0x60, 0x82,
]);

async function uploadProfilePhoto(
  page: import("@playwright/test").Page,
  name: string,
  buffer: Buffer,
) {
  return page.request.post("/api/files/upload", {
    multipart: {
      file: {
        name,
        mimeType: "image/png",
        buffer,
      },
    },
  });
}

/** Upload, wait for the list badge, and return the row's `/meson/files/:id` href. */
async function uploadAndGetDetailHref(
  page: import("@playwright/test").Page,
  name: string,
): Promise<string> {
  const upload = await uploadProfilePhoto(page, name, TINY_PNG);
  expect(upload.ok()).toBeTruthy();
  await gotoHydrated(page, "/meson");
  await expect(page.getByTestId("meson-file-status-badge").first()).toBeVisible({
    timeout: SCAN_TIMEOUT_MS,
  });
  const href = await page
    .getByTestId("meson-my-files")
    .locator("a")
    .first()
    .getAttribute("href");
  if (!href) {
    throw new Error("expected a file link in the My Files table after upload");
  }
  return href;
}

test.describe("Meson File Detail", () => {
  test.beforeEach(() => {
    test.skip(!hasSeedToken, "UF_E2E_SEED_TOKEN required for Meson File Detail probes");
  });

  test("meson-file-detail-image-preview-happy", async ({ page, request }) => {
    const credentials = await seedVerifiedUser(request);
    await signIn(page, credentials, "/meson");
    await assertAuthenticatedShell(page);

    const href = await uploadAndGetDetailHref(page, "detail-image.png");
    await gotoHydrated(page, href);

    await expect(page.getByTestId("meson-file-detail")).toBeAttached({
      timeout: 60_000,
    });
    await expect(page.getByText("Back to My Files")).toBeVisible();
    await expect(page.getByText(/not_found:/)).toHaveCount(0);
  });

  test("meson-file-detail-not-found-sad", async ({ page, request }) => {
    const credentials = await seedVerifiedUser(request);
    await signIn(page, credentials, "/meson");
    await assertAuthenticatedShell(page);

    await gotoHydrated(page, "/meson/files/e2e_meson_file%3Adefinitely-not-a-real-id");
    await expect(page.getByTestId("meson-file-detail")).toBeAttached({
      timeout: 60_000,
    });
    await expect(page.getByText(/not_found:/)).toBeVisible({ timeout: 60_000 });
  });

  test("meson-file-detail-peer-id-sad", async ({ page, request, browser }) => {
    const ownerCredentials = await seedVerifiedUser(request);
    await signIn(page, ownerCredentials, "/meson");
    await assertAuthenticatedShell(page);

    const peerContext = await browser.newContext();
    const peerPage = await peerContext.newPage();
    const peerCredentials = await seedVerifiedUser(peerContext.request);
    await signIn(peerPage, peerCredentials, "/meson");
    const peerHref = await uploadAndGetDetailHref(peerPage, "peer-file.png");
    await peerContext.close();

    // Same shape of not-found response as a genuinely missing id — IDOR-safe collapse.
    await gotoHydrated(page, peerHref);
    await expect(page.getByTestId("meson-file-detail")).toBeAttached({
      timeout: 60_000,
    });
    await expect(page.getByText(/not_found:/)).toBeVisible({ timeout: 60_000 });
  });

  test("meson-file-detail-anonymous-gate-sad", async ({ page }) => {
    await gotoHydrated(page, "/meson/files/e2e_meson_file%3Asome-id");
    await assertAnonymousShell(page);
    await expect(page.getByTestId("auth-required-empty-state")).toBeAttached({
      timeout: 60_000,
    });
    await expect(page.getByTestId("meson-file-detail")).toHaveCount(0);
  });
});
