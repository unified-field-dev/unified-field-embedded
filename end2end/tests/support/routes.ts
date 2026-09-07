/** Surface kind for a canonical embedded-host route. */
export type EmbeddedRouteKind =
  | "mounted"
  | "platform"
  | "auth"
  | "redirect"
  | "notFound";

/** Frozen embedded route contract from plan TM-01. */
export type EmbeddedRouteContract = {
  id: string;
  path: string;
  kind: EmbeddedRouteKind;
  expect?: string;
  redirectTo?: string;
  ownerRoot?: string;
  guardOutsideLayout?: boolean;
};

const mounted = (
  id: string,
  path: string,
  expect: string,
): EmbeddedRouteContract => ({ id, path, kind: "mounted", expect });

const platform = (
  id: string,
  path: string,
  ownerRoot: string,
  guardOutsideLayout = false,
): EmbeddedRouteContract => ({
  id,
  path,
  kind: "platform",
  ownerRoot,
  guardOutsideLayout,
});

const sharedRoutes: EmbeddedRouteContract[] = [
  {
    id: "embed-root-redirect",
    path: "/",
    kind: "redirect",
    redirectTo: "/apps",
  },
  { id: "embed-explicit-404", path: "/404", kind: "notFound" },
  mounted("embed-counter", "/counter", "counter-container"),
  mounted(
    "embed-counter-high-scores",
    "/counter/high-scores",
    "counter-container",
  ),
  mounted("embed-counter-admin", "/counter/admin", "counter-container"),
  mounted("embed-welcome", "/welcome", "featured-apps-card"),
  mounted("embed-welcome-admin", "/welcome/admin", "user-avatar"),
  mounted("embed-apps", "/apps", "apps-launcher-search"),
  mounted("embed-apps-counter", "/apps/counter", "user-avatar"),
  mounted("embed-orbital", "/orbital", "component-preview-container"),
  mounted(
    "embed-orbital-components",
    "/orbital/components",
    "preview-catalog-shell",
  ),
  mounted("embed-orbital-shell", "/orbital/shell", "preview-catalog-shell"),
  mounted("embed-orbital-button", "/orbital/button", "preview-catalog-shell"),
  mounted("embed-notifications", "/notifications", "user-avatar"),
  {
    id: "embed-user-root",
    path: "/user",
    kind: "redirect",
    redirectTo: "/user/account-settings",
  },
  mounted("embed-user-profile", "/user/profile", "user-avatar"),
  mounted("embed-user-appearance", "/user/appearance", "user-avatar"),
  mounted(
    "embed-user-account-settings",
    "/user/account-settings",
    "user-avatar",
  ),
  mounted(
    "embed-user-confirm-account",
    "/user/confirm-account",
    "confirm-account-container",
  ),
  {
    id: "embed-auth-root",
    path: "/auth",
    kind: "redirect",
    redirectTo: "/auth/signin",
  },
  {
    id: "embed-auth-signin",
    path: "/auth/signin",
    kind: "auth",
    expect: "signin-container",
  },
  {
    id: "embed-auth-signup",
    path: "/auth/signup",
    kind: "auth",
    expect: "signup-container",
  },
  {
    id: "embed-auth-logout",
    path: "/auth/logout",
    kind: "auth",
    expect: "logout-button",
  },
  {
    id: "embed-auth-oauth-callback",
    path: "/auth/oauth/callback",
    kind: "auth",
    expect: "auth-routes-layout-root",
  },
  {
    id: "embed-auth-reset-request",
    path: "/auth/reset/request",
    kind: "auth",
    expect: "auth-routes-layout-root",
  },
  {
    id: "embed-auth-reset-confirm",
    path: "/auth/reset/confirm",
    kind: "auth",
    expect: "auth-routes-layout-root",
  },
];

const platformRoutes: EmbeddedRouteContract[] = [
  platform("embed-valence", "/valence", "valence-app-root"),
  platform("embed-valence-schema", "/valence/schema", "valence-app-root"),
  platform(
    "embed-valence-schema-detail",
    "/valence/schema/example-schema",
    "valence-app-root",
  ),
  platform(
    "embed-valence-iter-run",
    "/valence/schema/example-schema/iter/example-run",
    "valence-app-root",
  ),
  platform(
    "embed-valence-deletion-run",
    "/valence/schema/example-schema/deletion/example-run",
    "valence-app-root",
  ),
  platform(
    "embed-valence-entity",
    "/valence/schema/example-schema/id/example-entity",
    "valence-app-root",
  ),
  platform("embed-valence-traits", "/valence/traits", "valence-app-root"),
  platform(
    "embed-valence-trait",
    "/valence/traits/example-trait",
    "valence-app-root",
  ),
  platform("embed-valence-iters", "/valence/iters", "valence-app-root"),
  platform(
    "embed-valence-deletions",
    "/valence/deletions",
    "valence-app-root",
  ),
  platform("embed-chronon", "/chronon", "chronon-app-root"),
  platform("embed-chronon-jobs", "/chronon/jobs", "chronon-app-root"),
  platform("embed-chronon-job-new", "/chronon/jobs/new", "chronon-app-root"),
  platform(
    "embed-chronon-job",
    "/chronon/jobs/example-job",
    "chronon-app-root",
  ),
  platform("embed-chronon-runs", "/chronon/runs", "chronon-app-root"),
  platform(
    "embed-chronon-run",
    "/chronon/runs/example-run",
    "chronon-app-root",
  ),
  platform("embed-chronon-scripts", "/chronon/scripts", "chronon-app-root"),
  platform("embed-photon", "/photon", "photon-app-root"),
  platform("embed-photon-topics", "/photon/topics", "photon-app-root"),
  platform(
    "embed-photon-topic",
    "/photon/topics/example-topic",
    "photon-app-root",
  ),
  platform(
    "embed-photon-subscriptions",
    "/photon/subscriptions",
    "photon-app-root",
  ),
  platform(
    "embed-photon-subscription",
    "/photon/subscriptions/example-id",
    "photon-app-root",
  ),
  platform("embed-photon-events", "/photon/events", "photon-app-root"),
  platform(
    "embed-photon-event",
    "/photon/events/example-id",
    "photon-app-root",
  ),
  platform("embed-spectra", "/spectra", "spectra-app-root"),
  platform("embed-spectra-schema", "/spectra/schema", "spectra-app-root"),
  platform(
    "embed-spectra-schema-detail",
    "/spectra/schema/example-schema",
    "spectra-app-root",
  ),
  platform(
    "embed-spectra-metric-explore",
    "/spectra/metric/example-metric/explore",
    "spectra-app-root",
  ),
  platform(
    "embed-spectra-event-explore",
    "/spectra/schema/example-schema/explore",
    "spectra-app-root",
  ),
  platform("embed-boson", "/boson", "boson-app-root"),
  platform("embed-boson-tasks", "/boson/tasks", "boson-app-root"),
  platform(
    "embed-boson-task",
    "/boson/tasks/example-task",
    "boson-app-root",
  ),
  platform(
    "embed-boson-task-config",
    "/boson/tasks/example-task/config",
    "boson-app-root",
  ),
  platform("embed-boson-queue", "/boson/queue", "boson-app-root"),
  platform("embed-boson-runs", "/boson/runs", "boson-app-root"),
  platform("embed-boson-run", "/boson/runs/example-run", "boson-app-root"),
  platform("embed-permission", "/permission", "permission-app-root"),
  platform(
    "embed-permissions",
    "/permission/permissions",
    "permission-app-root",
  ),
  platform(
    "embed-permission-detail",
    "/permission/permissions/example-id",
    "permission-app-root",
  ),
  platform(
    "embed-permission-create",
    "/permission/create-permission",
    "permission-app-root",
  ),
  platform(
    "embed-domain-create",
    "/permission/create-domain",
    "permission-app-root",
  ),
  platform(
    "embed-permission-groups",
    "/permission/groups",
    "permission-app-root",
  ),
  platform(
    "embed-permission-group",
    "/permission/groups/example-id",
    "permission-app-root",
  ),
  platform(
    "embed-permission-group-create",
    "/permission/create-group",
    "permission-app-root",
  ),
  platform(
    "embed-permission-requests",
    "/permission/requests",
    "permission-app-root",
  ),
  platform(
    "embed-permission-request",
    "/permission/requests/example-id",
    "permission-app-root",
  ),
  platform("embed-secrets", "/secrets", "neutrino-app-root", true),
  platform("embed-secrets-acl", "/secrets/acl", "neutrino-app-root", true),
  platform("embed-tag", "/tag", "tag-app-root", true),
  platform("embed-tag-create", "/tag/create", "tag-app-root", true),
  platform("embed-tag-detail", "/tag/example-id", "tag-app-root", true),
];

const fallbackRoutes: EmbeddedRouteContract[] = [
  {
    id: "embed-unknown-top-level",
    path: "/no-such-route-xyz",
    kind: "notFound",
  },
  {
    id: "embed-unknown-nested",
    path: "/spectra/no-such-route-xyz",
    kind: "notFound",
  },
];

/** Canonical route matrix for embedded host-conformance smokes. */
export const EMBEDDED_ROUTES: EmbeddedRouteContract[] = [
  ...sharedRoutes,
  ...platformRoutes,
  ...fallbackRoutes,
];
