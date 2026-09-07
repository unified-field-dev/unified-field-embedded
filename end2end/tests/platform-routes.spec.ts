import { test, expect, type Page } from "@playwright/test";
import { gotoHydrated } from "./support/hydration";

const PLATFORM_ROOTS = [
  ["valence", "valence-app-root", false],
  ["chronon", "chronon-app-root", false],
  ["photon", "photon-app-root", false],
  ["spectra", "spectra-app-root", false],
  ["boson", "boson-app-root", false],
  ["permission", "permission-app-root", false],
  ["secrets", "neutrino-app-root", true],
  ["tag", "tag-app-root", true],
] as const;

async function assertOnlyOwner(
  page: Page,
  ownerRoot: string,
  guardOutsideLayout: boolean,
): Promise<void> {
  if (guardOutsideLayout) {
    await expect(page.getByTestId(ownerRoot)).toHaveCount(0);
  } else {
    await expect(page.getByTestId(ownerRoot)).toBeAttached({ timeout: 60_000 });
  }
  for (const [, root] of PLATFORM_ROOTS) {
    if (root !== ownerRoot) {
      await expect(page.getByTestId(root)).toHaveCount(0);
    }
  }
}

test.describe("EMBED-RT platform ownership", () => {
  test("EMBED-RT-10-platform-root-ownership", async ({ page }) => {
    for (const [slug, ownerRoot, guardOutsideLayout] of PLATFORM_ROOTS) {
      await gotoHydrated(page, `/${slug}`);
      await assertOnlyOwner(page, ownerRoot, guardOutsideLayout);
      await expect(page.getByTestId("auth-required-empty-state")).toBeAttached({
        timeout: 60_000,
      });
    }
  });

  test("EMBED-RT-11-schema-and-dashboard-collision-contained", async ({
    page,
  }) => {
    for (const [path, ownerRoot] of [
      ["/valence/schema", "valence-app-root"],
      ["/chronon", "chronon-app-root"],
      ["/spectra/schema", "spectra-app-root"],
    ] as const) {
      await gotoHydrated(page, path);
      await assertOnlyOwner(page, ownerRoot, false);
    }
  });

  test("pw-l5-spectra-auth-gate-anonymous", async ({ page }) => {
    await gotoHydrated(page, "/spectra");
    await expect(page.getByTestId("auth-required-empty-state")).toBeAttached({
      timeout: 60_000,
    });
    await expect(page.getByTestId("spectra-home-page")).toHaveCount(0);
  });

  test("pw-l5-spectra-schema-index-anonymous", async ({ page }) => {
    await gotoHydrated(page, "/spectra/schema");
    await expect(page.getByTestId("auth-required-empty-state")).toBeAttached({
      timeout: 60_000,
    });
  });
});
