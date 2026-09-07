import type { Page, Request, Response } from "@playwright/test";

type FailureClass =
  | "pageerror"
  | "console-error"
  | "requestfailed"
  | "response-5xx";

type FailureRecord = {
  class: FailureClass;
  pathname: string;
  detail: string;
};

const RELEVANT_RESOURCE_TYPES = new Set([
  "document",
  "script",
  "stylesheet",
  "fetch",
  "xhr",
  "other",
]);

/** Strip query strings from same-origin URLs for safe diagnostics. */
export function stripQuery(url: string): string {
  try {
    const parsed = new URL(url);
    parsed.search = "";
    parsed.hash = "";
    return parsed.pathname;
  } catch {
    return url.split("?")[0]?.split("#")[0] ?? url;
  }
}

function pathnameOnly(url: string, baseURL: string): string | null {
  try {
    const parsed = new URL(url, baseURL);
    if (parsed.origin !== new URL(baseURL).origin) {
      return null;
    }
    return stripQuery(parsed.toString());
  } catch {
    return null;
  }
}

function isExpectedNavigationNoise(text: string): boolean {
  const lower = text.toLowerCase();
  return (
    lower.includes("net::err_aborted") ||
    lower.includes("navigation cancelled") ||
    lower.includes("navigation canceled") ||
    lower.includes("target closed") ||
    lower.includes("frame was detached")
  );
}

function isWasmRequest(url: string): boolean {
  return url.includes(".wasm") || url.includes("/pkg");
}

/**
 * Split WASM chunk fetches often abort with empty response when a redirect
 * remounts the shell mid-load (TM-04 integrity should ignore that noise).
 */
function isExpectedLazyChunkNoise(text: string, url?: string): boolean {
  const lower = text.toLowerCase();
  const emptyOrReset =
    lower.includes("err_empty_response") ||
    lower.includes("err_connection_reset") ||
    lower.includes("err_connection_closed") ||
    lower.includes("err_aborted");
  if (url != null && isWasmRequest(url) && emptyOrReset) {
    return true;
  }
  if (emptyOrReset && (lower.includes(".wasm") || lower.includes("/pkg"))) {
    return true;
  }
  if (lower.includes("dynamically imported module")) {
    return true;
  }
  if (
    lower.includes("typeerror: failed to fetch") ||
    lower === "failed to fetch" ||
    lower.startsWith("failed to fetch\n") ||
    lower.startsWith("failed to fetch ")
  ) {
    return true;
  }
  if (lower.includes("failed to load resource") && emptyOrReset) {
    return true;
  }
  return false;
}

function isExpectedWebsocketNoise(text: string, resourceType?: string): boolean {
  const lower = text.toLowerCase();
  return (
    resourceType === "websocket" ||
    (lower.includes("websocket") &&
      (lower.includes("closed") ||
        lower.includes("close") ||
        lower.includes("connection lost")))
  );
}

/** Collect same-origin browser and network failures during a route smoke. */
export class FailureProbe {
  private readonly records: FailureRecord[] = [];
  private readonly baseURL: string;
  private readonly handlers: Array<() => void> = [];

  private constructor(private readonly page: Page, baseURL: string) {
    this.baseURL = baseURL.replace(/\/$/, "");
  }

  /** Attach listeners to the page; call `detach()` in a finally block. */
  static attach(page: Page, baseURL?: string): FailureProbe {
    const probe = new FailureProbe(
      page,
      baseURL ??
        process.env.PLAYWRIGHT_BASE_URL ??
        "http://127.0.0.1:3000",
    );
    probe.install();
    return probe;
  }

  private push(className: FailureClass, url: string, detail: string): void {
    const pathname = pathnameOnly(url, this.baseURL);
    if (pathname) {
      this.records.push({
        class: className,
        pathname,
        detail: detail.slice(0, 240),
      });
    }
  }

  private install(): void {
    const onPageError = (error: Error) => {
      if (isExpectedLazyChunkNoise(error.message)) {
        return;
      }
      this.records.push({
        class: "pageerror",
        pathname: stripQuery(this.page.url()),
        detail: error.message.slice(0, 240),
      });
    };
    const onConsole = (msg: import("@playwright/test").ConsoleMessage) => {
      const text = msg.text();
      if (
        msg.type() === "error" &&
        !isExpectedNavigationNoise(text) &&
        !isExpectedWebsocketNoise(text) &&
        !isExpectedLazyChunkNoise(text)
      ) {
        this.records.push({
          class: "console-error",
          pathname: stripQuery(this.page.url()),
          detail: text.slice(0, 240),
        });
      }
    };
    const onRequestFailed = (request: Request) => {
      const failure = request.failure()?.errorText ?? "request failed";
      const resourceType = request.resourceType();
      if (
        isExpectedNavigationNoise(failure) ||
        isExpectedWebsocketNoise(failure, resourceType) ||
        isExpectedLazyChunkNoise(failure, request.url())
      ) {
        return;
      }
      if (
        RELEVANT_RESOURCE_TYPES.has(resourceType) ||
        isWasmRequest(request.url())
      ) {
        this.push("requestfailed", request.url(), `${resourceType}: ${failure}`);
      }
    };
    const onResponse = (response: Response) => {
      if (response.status() < 500) {
        return;
      }
      const resourceType = response.request().resourceType();
      if (
        RELEVANT_RESOURCE_TYPES.has(resourceType) ||
        isWasmRequest(response.url())
      ) {
        this.push(
          "response-5xx",
          response.url(),
          `${resourceType}: HTTP ${response.status()}`,
        );
      }
    };

    this.page.on("pageerror", onPageError);
    this.page.on("console", onConsole);
    this.page.on("requestfailed", onRequestFailed);
    this.page.on("response", onResponse);
    this.handlers.push(() => this.page.off("pageerror", onPageError));
    this.handlers.push(() => this.page.off("console", onConsole));
    this.handlers.push(() => this.page.off("requestfailed", onRequestFailed));
    this.handlers.push(() => this.page.off("response", onResponse));
  }

  /** Remove listeners installed by `attach`. */
  detach(): void {
    for (const off of this.handlers.splice(0)) {
      off();
    }
  }

  /** Throw when any classified failure was recorded. */
  assertClean(): void {
    if (this.records.length > 0) {
      const lines = this.records.map(
        (record) =>
          `${record.class} ${record.pathname} (${record.detail})`,
      );
      throw new Error(`browser integrity failures:\n${lines.join("\n")}`);
    }
  }
}

/** Attach a route-smoke failure probe. */
export function attachFailureProbe(
  page: Page,
  baseURL?: string,
): FailureProbe {
  return FailureProbe.attach(page, baseURL);
}
