import assert from "node:assert/strict";
import { mock, test } from "node:test";

process.env.CLOUDFLARE_ACCOUNT_ID = "test-account";
process.env.CLOUDFLARE_API_TOKEN = "test-token";
process.env.PR_NUMBER = "42";

test("deletes every matching preview after listing all pages", async () => {
  const deployment = (id, branch, environment = "preview") => ({
    id, environment, deployment_trigger: { metadata: { branch } },
  });
  const pages = [
    [deployment("first", "pr-42"), deployment("other", "pr-43")],
    [deployment("production", "pr-42", "production"), deployment("last", "pr-42")],
    [],
  ];
  const deleted = [];
  let listed = 0;
  const fetch = mock.method(globalThis, "fetch", async (url, options) => {
    const parsed = new URL(url);
    assert.equal(parsed.origin, "https://api.cloudflare.com");
    assert.equal(options.headers.Authorization, "Bearer test-token");
    let result;
    if (options.method === "GET") {
      assert.equal(parsed.pathname, "/client/v4/accounts/test-account/pages/projects/wie-dev/deployments");
      assert.equal(parsed.searchParams.get("env"), "preview");
      assert.equal(Number(parsed.searchParams.get("page")), ++listed);
      assert.deepEqual(deleted, []);
      result = pages[listed - 1];
    } else {
      assert.equal(options.method, "DELETE");
      assert.equal(listed, pages.length);
      assert.equal(parsed.searchParams.get("force"), "true");
      deleted.push(parsed.pathname.split("/").at(-1));
      result = null;
    }
    return Response.json({ success: true, result });
  });
  try {
    await import("./delete-preview-deployments.mjs?pagination");
    assert.deepEqual(deleted, ["first", "last"]);
  } finally {
    fetch.mock.restore();
  }
});

for (const [name, status, success] of [["HTTP", 403, false], ["API", 200, false]]) {
  test(`propagates ${name} failures without deleting deployments`, async () => {
    const fetch = mock.method(globalThis, "fetch", async () =>
      Response.json({ success, errors: [{ message: "denied" }] }, { status }));
    try {
      await assert.rejects(import(`./delete-preview-deployments.mjs?${name}`), /denied/);
      assert.equal(fetch.mock.callCount(), 1);
    } finally {
      fetch.mock.restore();
    }
  });
}
