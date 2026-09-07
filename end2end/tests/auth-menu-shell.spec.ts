import { test, expect } from "@playwright/test";

/**
 * Light shell check after lepton cycle-break: host injects AppBarUserMenu
 * via provide_shell_auth_menu. Anonymous visitors should see sign-in/sign-up
 * (user menu items and/or the shell auth dialog).
 */
test("app bar auth menu is present for anonymous session", async ({ page }) => {
  await page.goto("/welcome");
  await expect(page.getByTestId("user-avatar")).toBeVisible({ timeout: 60_000 });

  // Prefer opening the avatar menu; ignore intercepting backdrops.
  await page.keyboard.press("Escape");
  await page.getByTestId("user-avatar").click({ force: true });

  const menuSignIn = page.getByTestId("user-menu-signin");
  const menuSignUp = page.getByTestId("user-menu-signup");
  const dialogSignIn = page.getByRole("button", { name: "Sign In" });
  const dialogSignUp = page.getByRole("button", { name: "Sign Up" });

  await expect
    .poll(
      async () =>
        (await menuSignIn.count()) +
        (await menuSignUp.count()) +
        (await dialogSignIn.count()) +
        (await dialogSignUp.count()),
      {
        message:
          "expected AppBarUserMenu sign-in/up items or shell Sign In/Sign Up controls",
        timeout: 30_000,
      },
    )
    .toBeGreaterThan(0);
});
