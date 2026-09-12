//! Virus-scan upload scenarios (embedded host):
//! - meson-virus-scan-upload-available-happy
//! - meson-virus-scan-upload-quarantine-sad
//! - meson-virus-scan-photon-push-happy
//!
//! Requires UF_E2E_SEED_TOKEN. Uses AlwaysCleanScanner on embedded by default
//! for the happy path; quarantine-sad needs MESON_E2E_INFECTED=1 when the host
//! installs AlwaysInfectedScanner.

import { test, expect } from "@playwright/test";
import {
  assertAuthenticatedShell,
  seedVerifiedUser,
  signIn,
} from "./support/auth";
import { gotoHydrated } from "./support/hydration";

const hasSeedToken = Boolean(process.env.UF_E2E_SEED_TOKEN?.trim());

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

test.describe("Meson virus scan", () => {
  test.beforeEach(() => {
    test.skip(!hasSeedToken, "UF_E2E_SEED_TOKEN required for Meson virus-scan probes");
  });

  test("meson-virus-scan-upload-available-happy", async ({ page, request }) => {
    const credentials = await seedVerifiedUser(request);
    await signIn(page, credentials, "/meson");
    await assertAuthenticatedShell(page);

    await expect(page.getByTestId("meson-app-root")).toBeAttached({
      timeout: 60_000,
    });

    const upload = await uploadProfilePhoto(page, "scan-ok.png", TINY_PNG);
    expect(upload.ok()).toBeTruthy();

    await gotoHydrated(page, "/meson");
    // AlwaysClean promotes quickly; list badge should reach Available.
    await expect(page.getByText(/Available|Pending/i).first()).toBeVisible({
      timeout: 60_000,
    });
    await expect(page.getByText(/Available/i).first()).toBeVisible({
      timeout: 60_000,
    });
  });

  test("meson-virus-scan-upload-quarantine-sad", async ({ page, request }) => {
    test.skip(
      process.env.MESON_E2E_INFECTED !== "1",
      "Set MESON_E2E_INFECTED=1 with AlwaysInfectedScanner on host for this sad path",
    );
    const credentials = await seedVerifiedUser(request);
    await signIn(page, credentials, "/meson");
    await assertAuthenticatedShell(page);

    const upload = await uploadProfilePhoto(page, "bad.png", TINY_PNG);
    expect(upload.ok()).toBeTruthy();

    await gotoHydrated(page, "/meson");
    await expect(page.getByText(/Quarantined/i).first()).toBeVisible({
      timeout: 60_000,
    });
  });

  test("meson-virus-scan-photon-push-happy", async ({ page, request }) => {
    const credentials = await seedVerifiedUser(request);
    await signIn(page, credentials, "/meson");
    await assertAuthenticatedShell(page);

    await expect(page.getByTestId("meson-app-root")).toBeAttached({
      timeout: 60_000,
    });

    const before = page.url();
    const upload = await uploadProfilePhoto(page, "push-ok.png", TINY_PNG);
    expect(upload.ok()).toBeTruthy();

    // Stay on /meson; Photon refetch should surface status without full navigation.
    await expect(page.getByText(/Available|Pending/i).first()).toBeVisible({
      timeout: 60_000,
    });
    expect(page.url()).toBe(before);
  });
});
