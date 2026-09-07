import { test, expect, type Page } from "@playwright/test";
import { assertSignInSurface, assertSignUpSurface } from "./support/auth";
import { attachFailureProbe } from "./support/diagnostics";
import { gotoHydrated } from "./support/hydration";
import {
  EMBEDDED_ROUTES,
  type EmbeddedRouteContract,
} from "./support/routes";

async function assertRouteSurface(
  page: Page,
  route: EmbeddedRouteContract,
): Promise<void> {
  if (route.kind === "redirect") {
    expect(route.redirectTo).toBeTruthy();
    const pathname = new URL(page.url()).pathname;
    const userAuthGate =
      route.path === "/user" && pathname === "/auth/signin";
    expect(pathname === route.redirectTo || userAuthGate).toBe(true);
    if (pathname === "/auth/signin") {
      await assertSignInSurface(page);
    }
    return;
  }

  if (route.kind === "notFound") {
    await expect(page.getByTestId("unified-field-not-found-page")).toBeVisible({
      timeout: 60_000,
    });
    await expect(page.getByTestId("unified-field-coming-soon-page")).toHaveCount(
      0,
    );
    return;
  }

  if (route.kind === "platform") {
    if (route.guardOutsideLayout) {
      await expect(page.getByTestId(route.ownerRoot!)).toHaveCount(0);
    } else {
      await expect(page.getByTestId(route.ownerRoot!)).toBeAttached({
        timeout: 60_000,
      });
    }
    await expect(page.getByTestId("auth-required-empty-state")).toBeAttached({
      timeout: 60_000,
    });
    return;
  }

  if (route.kind === "auth") {
    if (route.expect === "signin-container") {
      await assertSignInSurface(page);
    } else if (route.expect === "signup-container") {
      await assertSignUpSurface(page);
    } else {
      await expect(page.getByTestId(route.expect!)).toBeAttached({
        timeout: 60_000,
      });
    }
    return;
  }

  const expected = page.getByTestId(route.expect!);
  if ((await expected.count()) > 0) {
    await expect(expected.first()).toBeAttached({ timeout: 60_000 });
    return;
  }
  const authGate = page.getByTestId("auth-required-empty-state");
  const signInDialog = page
    .getByRole("dialog")
    .filter({ hasText: /Sign in required/i });
  if ((await authGate.count()) > 0 || (await signInDialog.count()) > 0) {
    await expect(authGate.or(signInDialog).first()).toBeAttached({
      timeout: 60_000,
    });
    return;
  }
  if (page.url().includes("/auth/signin")) {
    await assertSignInSurface(page);
    return;
  }
  await expect(page.getByTestId("user-avatar")).toBeVisible({
    timeout: 60_000,
  });
}

test.describe("EMBED-RT route smoke matrix", () => {
  test("EMBED-RT-00-route-contract-ids-and-paths-are-unique", () => {
    const ids = EMBEDDED_ROUTES.map((route) => route.id);
    const paths = EMBEDDED_ROUTES.map((route) => route.path);
    expect(new Set(ids).size).toBe(ids.length);
    expect(new Set(paths).size).toBe(paths.length);
  });

  for (const route of EMBEDDED_ROUTES) {
    test(route.id, async ({ page, baseURL }) => {
      const probe = attachFailureProbe(page, baseURL);
      try {
        await gotoHydrated(page, route.path);
        await assertRouteSurface(page, route);
        probe.assertClean();
      } finally {
        probe.detach();
      }
    });
  }
});
