#!/usr/bin/env node
// Landscape link health check.
//
// Walks every published landscape entry, checks whether its website still
// resolves, and tracks a per-entry failure streak in a small JSON state file.
// After a configurable number of consecutive failing runs an entry is
// unpublished (reversible), never deleted, so a human can review before any
// permanent removal.
//
// The DB helpers below are intentionally a small self-contained copy of the
// ones in server.mjs so this checker can run as a standalone job without
// importing the HTTP server.
//
// Environment:
//   DATABASE_URL or TERN_CONF        database connection (same as the MCP server)
//   HEALTHCHECK_ACTOR_USER_ID        uuid recorded as the actor for unpublish audit logs (required to apply)
//   HEALTHCHECK_APPLY                "true" to actually unpublish; otherwise dry run (default dry run)
//   HEALTHCHECK_FAILURE_THRESHOLD    consecutive failing runs before unpublishing (default 3)
//   HEALTHCHECK_STATE_FILE           streak state file (default ~/.config/ocg/landscape-health-state.json)
//   HEALTHCHECK_TIMEOUT_MS           per-request timeout in ms (default 15000)
//   HEALTHCHECK_CONCURRENCY          parallel checks (default 12)

import { spawn } from "node:child_process";
import { readFile, writeFile, mkdir } from "node:fs/promises";
import { dirname, join } from "node:path";

const UUID_RE = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
const HOME = process.env.HOME || "";
const ACTOR = process.env.HEALTHCHECK_ACTOR_USER_ID || "";
const APPLY = process.env.HEALTHCHECK_APPLY === "true";
const THRESHOLD = Math.max(1, Number.parseInt(process.env.HEALTHCHECK_FAILURE_THRESHOLD || "3", 10));
const TIMEOUT_MS = Math.max(1000, Number.parseInt(process.env.HEALTHCHECK_TIMEOUT_MS || "15000", 10));
const CONCURRENCY = Math.max(1, Number.parseInt(process.env.HEALTHCHECK_CONCURRENCY || "12", 10));
const STATE_FILE =
  process.env.HEALTHCHECK_STATE_FILE || join(HOME, ".config/ocg/landscape-health-state.json");
const USER_AGENT =
  "Mozilla/5.0 (compatible; goup-landscape-health/1.0; +https://goup.vc/landscape)";

// A live server, even one that blocks bots, means the domain is still owned and
// running, so these are treated as alive. The goal is to catch abandoned or
// parked domains, not to punish sites that reject automated clients.
const ALIVE_STATUSES = new Set([
  200, 201, 202, 203, 204, 206, 301, 302, 303, 307, 308, 401, 403, 405, 429,
]);
// Hard signals that a site is gone rather than briefly unavailable.
const DEAD_STATUSES = new Set([404, 410, 451]);

// A domain can answer with 200 and still be gone: registrars and parking
// services serve a for-sale or lander page in place of the old product. These
// signals catch the common cases so an exited startup is not treated as alive
// just because someone still holds the domain.
const PARKING_HOST_SIGNALS = [
  "sedoparking.com", "bodis.com", "dan.com", "afternic.com", "hugedomains.com",
  "parkingcrew.net", "above.com", "uniregistry.com", "domainmarket.com",
  "sav.com", "voodoo.com", "cashparking.com", "domain.com/lander",
];
const PARKING_BODY_SIGNALS = [
  "/lander", "window.location.href=\"/lander", "domain is for sale",
  "buy this domain", "this domain is for sale", "this domain may be for sale",
  "the domain name", "parked free", "parkingcrew", "sedoparking", "hugedomains",
  "is for sale", "domain for sale", "checkout the full domain details",
];
const MAX_BODY_BYTES = 200000;

// Reads a parked/for-sale or empty placeholder verdict from a 200 response.
function looksParkedOrEmpty(finalUrl, body) {
  const host = (() => {
    try {
      return new URL(finalUrl).host.toLowerCase();
    } catch {
      return "";
    }
  })();
  if (PARKING_HOST_SIGNALS.some((s) => host.includes(s) || finalUrl.toLowerCase().includes(s))) {
    return "parked_host";
  }
  const low = (body || "").toLowerCase();
  if (PARKING_BODY_SIGNALS.some((s) => low.includes(s))) return "parked_page";
  // A tiny body with no title and no real content is a placeholder, not a product.
  const stripped = low.replace(/\s+/g, "");
  if (stripped.length < 512 && !low.includes("<title") && !low.includes("<body")) {
    return "empty_page";
  }
  return null;
}

// ---------------------------------------------------------------------------
// DB access (self-contained copy of the server helpers)
// ---------------------------------------------------------------------------

async function readTernConfig(path) {
  const raw = await readFile(path, "utf8");
  const config = {};
  let inDatabaseSection = false;
  for (const line of raw.split("\n")) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("#")) continue;
    if (trimmed.startsWith("[")) {
      inDatabaseSection = trimmed === "[database]";
      continue;
    }
    if (!inDatabaseSection) continue;
    const match = trimmed.match(/^([A-Za-z0-9_]+)\s*=\s*(.*)$/);
    if (!match) continue;
    config[match[1]] = match[2].replace(/^"(.*)"$/, "$1");
  }
  return config;
}

async function buildPsqlCommand() {
  if (process.env.DATABASE_URL) {
    return { args: [process.env.DATABASE_URL], env: process.env };
  }
  const configPath = process.env.TERN_CONF || join(HOME, ".config/ocg/tern.conf");
  const config = await readTernConfig(configPath);
  const args = [];
  if (config.host) args.push("-h", config.host);
  if (config.port) args.push("-p", config.port);
  if (config.user) args.push("-U", config.user);
  if (config.database) args.push("-d", config.database);
  return { args, env: { ...process.env, PGPASSWORD: process.env.PGPASSWORD || config.password || "" } };
}

async function runPsql(sql) {
  const { args, env } = await buildPsqlCommand();
  return new Promise((resolve, reject) => {
    const child = spawn("psql", [...args, "-X", "-A", "-t", "-v", "ON_ERROR_STOP=1", "-c", sql], {
      env,
      stdio: ["ignore", "pipe", "pipe"],
    });
    const stdout = [];
    const stderr = [];
    child.stdout.on("data", (chunk) => stdout.push(chunk));
    child.stderr.on("data", (chunk) => stderr.push(chunk));
    child.on("error", reject);
    child.on("close", (code) => {
      if (code === 0) {
        resolve(Buffer.concat(stdout).toString("utf8"));
        return;
      }
      reject(new Error(Buffer.concat(stderr).toString("utf8").trim() || `psql exited with code ${code}`));
    });
  });
}

async function fetchPublishedEntries() {
  const sql = `
select coalesce(json_agg(row_to_json(t)), '[]')::text
from (
  select landscape_entry_id, alliance_id, name, slug, website_url
  from landscape_entry
  where published = true
    and website_url is not null
    and btrim(website_url) <> ''
  order by created_at
) t;`;
  const raw = (await runPsql(sql)).trim();
  return JSON.parse(raw || "[]");
}

async function unpublishEntry(actorUserId, allianceId, entryId) {
  if (!UUID_RE.test(actorUserId) || !UUID_RE.test(allianceId) || !UUID_RE.test(entryId)) {
    throw new Error("invalid uuid passed to unpublishEntry");
  }
  const sql = `select update_landscape_entry_published('${actorUserId}'::uuid, '${allianceId}'::uuid, '${entryId}'::uuid, false);`;
  await runPsql(sql);
}

// ---------------------------------------------------------------------------
// Liveness check
// ---------------------------------------------------------------------------

function normalizeUrl(raw) {
  const url = String(raw).trim();
  if (!url) return null;
  if (!/^https?:\/\//i.test(url)) return `https://${url}`;
  return url;
}

async function probe(url) {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), TIMEOUT_MS);
  try {
    const resp = await fetch(url, {
      method: "GET",
      redirect: "follow",
      signal: controller.signal,
      headers: { "User-Agent": USER_AGENT, Accept: "text/html,*/*" },
    });
    let body = "";
    if (ALIVE_STATUSES.has(resp.status)) {
      try {
        body = (await resp.text()).slice(0, MAX_BODY_BYTES);
      } catch {
        body = "";
      }
    }
    return { kind: "status", status: resp.status, finalUrl: resp.url || url, body };
  } catch (error) {
    const code = error && (error.cause?.code || error.code || error.name);
    return { kind: "network", code: String(code || "error") };
  } finally {
    clearTimeout(timer);
  }
}

// Returns { alive: boolean, detail: string }. Transient problems (timeout,
// 5xx, generic network blips) are retried once and, if still failing, count as
// a soft failure for this run rather than a definite death.
async function checkEntry(entry) {
  const primary = normalizeUrl(entry.website_url);
  if (!primary) return { alive: true, detail: "no-url" };

  const attempt = async (url) => {
    const result = await probe(url);
    if (result.kind === "status") {
      if (ALIVE_STATUSES.has(result.status)) {
        const parked = looksParkedOrEmpty(result.finalUrl, result.body);
        if (parked) return { alive: false, detail: parked };
        return { alive: true, detail: `http_${result.status}` };
      }
      if (DEAD_STATUSES.has(result.status)) return { alive: false, detail: `http_${result.status}` };
      return { alive: false, detail: `http_${result.status}`, transient: result.status >= 500 };
    }
    // DNS not found means the domain is gone; other network errors are softer.
    const hardDead = ["ENOTFOUND", "ERR_TLS_CERT_ALTNAME_INVALID"].includes(result.code);
    return { alive: false, detail: result.code, transient: !hardDead };
  };

  let outcome = await attempt(primary);
  // http -> https fallback for a plain http site, and one retry for transient failures.
  if (!outcome.alive && outcome.transient) {
    outcome = await attempt(primary);
  }
  return outcome;
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

async function loadState() {
  try {
    return JSON.parse(await readFile(STATE_FILE, "utf8"));
  } catch {
    return {};
  }
}

async function saveState(state) {
  await mkdir(dirname(STATE_FILE), { recursive: true });
  await writeFile(STATE_FILE, JSON.stringify(state, null, 2));
}

async function mapWithConcurrency(items, limit, worker) {
  const results = new Array(items.length);
  let cursor = 0;
  const runners = Array.from({ length: Math.min(limit, items.length) }, async () => {
    while (cursor < items.length) {
      const index = cursor++;
      results[index] = await worker(items[index], index);
    }
  });
  await Promise.all(runners);
  return results;
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function main() {
  const nowIso = new Date().toISOString();
  const entries = await fetchPublishedEntries();
  const state = await loadState();
  const liveIds = new Set(entries.map((e) => e.landscape_entry_id));

  // Drop state for entries that no longer exist or are already unpublished.
  for (const id of Object.keys(state)) {
    if (!liveIds.has(id)) delete state[id];
  }

  const checks = await mapWithConcurrency(entries, CONCURRENCY, async (entry) => {
    const outcome = await checkEntry(entry);
    return { entry, outcome };
  });

  const toUnpublish = [];
  let aliveCount = 0;
  let failingCount = 0;

  for (const { entry, outcome } of checks) {
    const id = entry.landscape_entry_id;
    const prior = state[id] || { fails: 0 };

    if (outcome.alive) {
      state[id] = { fails: 0, last_status: outcome.detail, last_checked_at: nowIso, last_ok_at: nowIso };
      aliveCount += 1;
      continue;
    }

    const fails = (prior.fails || 0) + 1;
    state[id] = {
      fails,
      last_status: outcome.detail,
      last_checked_at: nowIso,
      last_ok_at: prior.last_ok_at || null,
    };
    failingCount += 1;
    if (fails >= THRESHOLD) {
      toUnpublish.push({ entry, fails, detail: outcome.detail });
    }
  }

  const unpublished = [];
  const wouldUnpublish = [];
  for (const item of toUnpublish) {
    const { entry, fails, detail } = item;
    if (APPLY && UUID_RE.test(ACTOR)) {
      try {
        await unpublishEntry(ACTOR, entry.alliance_id, entry.landscape_entry_id);
        delete state[entry.landscape_entry_id];
        unpublished.push({ name: entry.name, slug: entry.slug, fails, detail });
      } catch (error) {
        wouldUnpublish.push({ name: entry.name, slug: entry.slug, fails, detail, error: String(error.message || error) });
      }
    } else {
      wouldUnpublish.push({ name: entry.name, slug: entry.slug, fails, detail });
    }
  }

  await saveState(state);

  const report = {
    checked_at: nowIso,
    mode: APPLY && UUID_RE.test(ACTOR) ? "apply" : "dry-run",
    threshold: THRESHOLD,
    total_checked: entries.length,
    alive: aliveCount,
    failing_this_run: failingCount,
    unpublished_count: unpublished.length,
    would_unpublish_count: wouldUnpublish.length,
    unpublished,
    would_unpublish: wouldUnpublish,
  };
  process.stdout.write(`${JSON.stringify(report, null, 2)}\n`);

  if (report.mode === "dry-run" && (unpublished.length || wouldUnpublish.length)) {
    process.stderr.write(
      "Dry run: set HEALTHCHECK_APPLY=true and HEALTHCHECK_ACTOR_USER_ID to unpublish the entries above.\n",
    );
  }
}

main().catch((error) => {
  process.stderr.write(`landscape health check failed: ${error && error.stack ? error.stack : error}\n`);
  process.exit(1);
});
