import { test, expect } from "@playwright/test";
import { stripQuery } from "./support/diagnostics";

test.describe("TM-07/SM-04 diagnostics redaction", () => {
  test("stripQuery drops search and hash", () => {
    expect(stripQuery("http://127.0.0.1:3000/auth/signin?token=secret#frag")).toBe(
      "/auth/signin",
    );
  });

  test("stripQuery keeps pathname for relative-looking URLs", () => {
    expect(stripQuery("/apps?x=1")).toBe("/apps");
  });
});
