import { expect, type APIRequestContext, type Page } from "@playwright/test";
import { gotoHydrated, waitForHydrated } from "./hydration";

export type SeedCredentials = {
  email: string;
  password: string;
};

type SeedAck = {
  scenario?: string;
  email?: string;
};

const DEFAULT_PASSWORD = "CorrectHorseBattery1!";

function seedToken(): string {
  const token = process.env.UF_E2E_SEED_TOKEN?.trim();
  if (!token) {
    throw new Error(
      "UF_E2E_SEED_TOKEN is required for seedVerifiedUser (e2e-host-conformance builds only)",
    );
  }
  return token;
}

function syntheticEmail(): string {
  const stamp = `${Date.now()}-${Math.random().toString(36).slice(2, 10)}`;
  return `embedded-e2e-${stamp}@example.test`;
}

/** Seed a verified user through the bearer-protected test route. */
export async function seedVerifiedUser(
  request: APIRequestContext,
  opts?: { email?: string; password?: string },
): Promise<SeedCredentials> {
  const email = opts?.email ?? syntheticEmail();
  const password = opts?.password ?? DEFAULT_PASSWORD;
  const response = await request.post("/api/test/seed-data", {
    headers: { Authorization: `Bearer ${seedToken()}` },
    data: { scenario: "auth_basic_user", email, password },
  });
  if (!response.ok()) {
    const body = (await response.text()).slice(0, 500);
    throw new Error(
      `seed-data failed: status=${response.status()} body=${body}`,
    );
  }
  const ack = (await response.json()) as SeedAck;
  return { email: ack.email ?? email, password };
}

/** Clear portaled dialogs and backdrops that intercept avatar clicks. */
export async function dismissAuthOverlay(page: Page): Promise<void> {
  for (let i = 0; i < 3; i += 1) {
    await page.keyboard.press("Escape").catch(() => undefined);
  }
  const visibleBackdrop = page
    .locator(".orbital-backdrop")
    .filter({ visible: true });
  const count = await visibleBackdrop.count().catch(() => 0);
  for (let i = 0; i < count; i += 1) {
    await visibleBackdrop
      .nth(i)
      .click({ force: true, timeout: 2_000 })
      .catch(() => undefined);
  }
}

/** Assert the portaled production sign-in form is interactive. */
export async function assertSignInSurface(page: Page): Promise<void> {
  await expect(page.getByTestId("signin-container")).toBeAttached({
    timeout: 60_000,
  });
  await expect(page.getByTestId("signin-email")).toBeVisible({
    timeout: 60_000,
  });
}

/** Assert the portaled production sign-up form is interactive. */
export async function assertSignUpSurface(page: Page): Promise<void> {
  await expect(page.getByTestId("signup-container")).toBeAttached({
    timeout: 60_000,
  });
  await expect(page.getByTestId("signup-email")).toBeVisible({
    timeout: 60_000,
  });
}

/** Open the app-bar avatar menu. */
export async function openUserMenu(page: Page): Promise<void> {
  await dismissAuthOverlay(page);
  const avatar = page.getByTestId("user-avatar");
  await expect(avatar).toBeVisible({ timeout: 60_000 });
  await avatar.click({ force: true, timeout: 15_000 });
}

/** Authenticated chrome exposes Notifications and no Guest avatar. */
export async function assertAuthenticatedShell(page: Page): Promise<void> {
  await expect(page.getByTestId("user-avatar")).toBeVisible({ timeout: 60_000 });
  await expect(page.getByRole("button", { name: "Notifications" })).toBeVisible({
    timeout: 60_000,
  });
  await expect(page.getByRole("img", { name: "Guest" })).toHaveCount(0);
}

/** Anonymous chrome: Guest avatar, or an auth-required gate (no Notifications). */
export async function assertAnonymousShell(page: Page): Promise<void> {
  await expect(page.getByRole("button", { name: "Notifications" })).toHaveCount(
    0,
  );
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
  await expect(page.getByTestId("user-avatar")).toBeVisible({ timeout: 60_000 });
  await expect(page.getByRole("img", { name: "Guest" })).toBeVisible({
    timeout: 30_000,
  });
}

/** A row of uf-help's `uf.help.tour_steps` localStorage list. */
type HelpVisit = {
  route: string;
  feature_highlight: string;
  spotlight: string | null;
  replay: boolean;
};

const WELCOME_TOUR_STEPS = [
  ["welcome-featured", "welcome-featured-card"],
  ["welcome-featured-view-all", "welcome-featured-view-all"],
  ["welcome-recent", "welcome-recent-apps-card"],
  ["welcome-most-used", "welcome-most-used-card"],
  ["welcome-popular", "welcome-popular-apps-card"],
] as const;

/**
 * Mark the `/welcome` Help tour seen before any page loads.
 *
 * A new user's first `/welcome` visit auto-opens the uf-welcome spotlight tour
 * once its visit list resolves. That lands after the shell is already
 * authenticated, so it can take the avatar click and leave the user menu shut.
 */
export async function markWelcomeTourSeen(page: Page): Promise<void> {
  await page.addInitScript((steps) => {
    try {
      const key = "uf.help.tour_steps";
      const rows: HelpVisit[] = JSON.parse(localStorage.getItem(key) ?? "[]");
      for (const [featureHighlight, spotlight] of steps) {
        const seen = rows.some(
          (r) => r.route === "/welcome" && r.feature_highlight === featureHighlight,
        );
        if (!seen) {
          rows.push({
            route: "/welcome",
            feature_highlight: featureHighlight,
            spotlight,
            replay: false,
          });
        }
      }
      localStorage.setItem(key, JSON.stringify(rows));
    } catch {
      /* storage unavailable: the tour may show */
    }
  }, WELCOME_TOUR_STEPS);
}

/** Prove the menu reflects the authenticated host session. */
export async function assertAuthenticatedMenu(page: Page): Promise<void> {
  await openUserMenu(page);
  await expect(page.getByTestId("user-menu-profile")).toBeVisible({
    timeout: 30_000,
  });
  await expect(page.getByTestId("user-menu-logout")).toBeAttached({
    timeout: 15_000,
  });
  await page.keyboard.press("Escape").catch(() => undefined);
  await dismissAuthOverlay(page);
}

/** Sign in through the production form and wait for authenticated chrome. */
export async function signIn(
  page: Page,
  credentials: SeedCredentials,
  referer = "/welcome",
): Promise<void> {
  const query =
    referer && referer !== "/" ? `?referer=${encodeURIComponent(referer)}` : "";
  await gotoHydrated(page, `/auth/signin${query}`);
  await assertSignInSurface(page);
  await page
    .getByTestId("signin-email")
    .locator('input[name="email"]')
    .fill(credentials.email);
  await page
    .getByTestId("signin-password")
    .locator('input[name="password"]')
    .fill(credentials.password);
  await page.getByTestId("signin-submit").getByRole("button").click();
  await expect
    .poll(async () => page.url().includes(referer), { timeout: 60_000 })
    .toBe(true);
  await waitForHydrated(page);
  await dismissAuthOverlay(page);
  await assertAuthenticatedShell(page);
}

/** Sign out through the dedicated logout route. */
export async function signOut(page: Page): Promise<void> {
  await gotoHydrated(page, "/auth/logout");
  const logoutButton = page
    .getByTestId("logout-button")
    .getByRole("button");
  await expect(logoutButton).toBeAttached({ timeout: 60_000 });
  await logoutButton.click({ force: true });
  await dismissAuthOverlay(page);
  await gotoHydrated(page, "/welcome");
  await assertAnonymousShell(page);
}
