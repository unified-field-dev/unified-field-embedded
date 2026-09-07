import { test, expect } from "@playwright/test";
import { gotoHydrated } from "./support/hydration";

test.describe("EMBED-RT host fallbacks", () => {
  test("EMBED-RT-52-explicit-and-catchall-use-404", async ({ page }) => {
    for (const path of [
      "/404",
      "/another-missing-route-abc",
      "/spectra/another-missing-route-abc",
    ]) {
      await gotoHydrated(page, path);
      await expect(page.getByTestId("unified-field-not-found-page")).toBeVisible({
        timeout: 60_000,
      });
      await expect(
        page.getByTestId("unified-field-coming-soon-page"),
      ).toHaveCount(0);
    }
  });
});
