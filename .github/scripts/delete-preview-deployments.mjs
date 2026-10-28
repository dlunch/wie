const baseUrl = `https://api.cloudflare.com/client/v4/accounts/${process.env.CLOUDFLARE_ACCOUNT_ID}/pages/projects/wie-dev/deployments`;
const branch = `pr-${process.env.PR_NUMBER}`;

async function request(path, method = "GET") {
  const response = await fetch(`${baseUrl}${path}`, {
    method,
    headers: { Authorization: `Bearer ${process.env.CLOUDFLARE_API_TOKEN}` },
  });
  const body = await response.json();
  if (!response.ok || !body.success) {
    throw new Error(`Cloudflare ${method} ${path} failed (${response.status}): ${JSON.stringify(body.errors)}`);
  }
  return body.result;
}

// Collect every page before deleting so removal cannot shift the remaining pages.
const ids = [];
for (let page = 1; ; page++) {
  const deployments = await request(`?env=preview&page=${page}`);
  if (deployments.length === 0) break;
  for (const deployment of deployments) {
    if (deployment.environment === "preview" && deployment.deployment_trigger.metadata.branch === branch) {
      ids.push(deployment.id);
    }
  }
}

for (const id of ids) {
  await request(`/${id}?force=true`, "DELETE");
  console.log(`Deleted ${branch} deployment ${id}`);
}
