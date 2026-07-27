#!/usr/bin/env python3
"""Build and validate the discovery catalog from pinned, public upstream data."""

from __future__ import annotations

import argparse
import csv
import datetime as dt
import json
import re
import sys
import urllib.error
import urllib.request
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from urllib.parse import urlsplit, urlunsplit


ROOT = Path(__file__).resolve().parent
TODAY = dt.date.today().isoformat()
TIMEOUT_SECONDS = 20
WORKERS = 24

# These commits are intentionally pinned so rebuilds have stable inputs.
JOBS_REPOSITORY = "Feashliaa/job-board-aggregator"
JOBS_COMMIT = "bc0fa75f98b6151177752f7d1ad54d9bd1b0a355"
EVENTS_REPOSITORY = "dmitryvinn/tech-conferences"
EVENTS_COMMIT = "dc8cbe0611405a56849e2e7a9c45addde099d6a9"
CONFS_TECH_REPOSITORY = "tech-conferences/conference-data"
CONFS_TECH_COMMIT = "56efdfcf195b7531ebda8f56a3c514002ddd2973"

JOBS_INPUTS = {
    "greenhouse": "data/greenhouse_companies.json",
    "lever": "data/lever_companies.json",
    "ashby": "data/ashby_companies.json",
    "workday": "data/workday_companies.json",
    "bamboohr": "data/bamboohr_companies.json",
    "icims": "data/icims_companies.json",
    "paylocity": "data/paylocity_companies_clean.json",
}


def fetch_json(repository: str, commit: str, path: str) -> object:
    url = f"https://raw.githubusercontent.com/{repository}/{commit}/{path}"
    with urllib.request.urlopen(url, timeout=TIMEOUT_SECONDS) as response:
        return json.load(response)


def conference_paths() -> list[str]:
    """Return current and recent confs.tech topic files at the pinned revision."""
    tree_url = (
        f"https://api.github.com/repos/{CONFS_TECH_REPOSITORY}/git/trees/"
        f"{CONFS_TECH_COMMIT}?recursive=1"
    )
    request = urllib.request.Request(tree_url, headers={"User-Agent": "goup-discovery-catalog/1.0"})
    with urllib.request.urlopen(request, timeout=TIMEOUT_SECONDS) as response:
        tree = json.load(response)
    return sorted(
        entry["path"]
        for entry in tree["tree"]
        if entry.get("type") == "blob"
        and re.fullmatch(r"conferences/202[5-7]/[^/]+\.json", entry.get("path", ""))
    )


def canonicalize(url: str) -> str:
    parsed = urlsplit(url.strip())
    if parsed.scheme not in {"http", "https"} or not parsed.netloc:
        raise ValueError(f"not an HTTP(S) URL: {url!r}")
    return urlunsplit((parsed.scheme, parsed.netloc.lower(), parsed.path.rstrip("/") or "/", "", ""))


def validate_url(url: str) -> tuple[str, int] | None:
    request = urllib.request.Request(url, headers={"User-Agent": "goup-discovery-catalog/1.0"})
    try:
        with urllib.request.urlopen(request, timeout=TIMEOUT_SECONDS) as response:
            return canonicalize(response.url), response.status
    except (urllib.error.HTTPError, urllib.error.URLError, OSError, TimeoutError, ValueError):
        return None


def slug_label(value: str) -> str:
    return re.sub(r"[-_.]+", " ", value).strip().title()


def job_url(ats: str, slug: str) -> str | None:
    slug = str(slug).strip()
    if not slug:
        return None
    templates = {
        "greenhouse": "https://job-boards.greenhouse.io/{slug}",
        "lever": "https://jobs.lever.co/{slug}",
        "ashby": "https://jobs.ashbyhq.com/{slug}",
        "bamboohr": "https://{slug}.bamboohr.com/careers",
        "paylocity": "https://recruiting.paylocity.com/recruiting/jobs/All/{slug}",
    }
    return templates.get(ats, "").format(slug=slug) or None


def scalar_values(data: object) -> list[str]:
    if isinstance(data, list):
        return [str(item) for item in data if isinstance(item, (str, int))]
    if isinstance(data, dict):
        return [str(item) for item in data.values() if isinstance(item, (str, int))]
    return []


def verify_candidates(candidates: list[dict[str, str]], limit: int) -> list[dict[str, str]]:
    accepted: list[dict[str, str]] = []
    seen: set[str] = set()
    # Process in stable batches. executor.map preserves candidate order, so the
    # selected prefix does not depend on which server happens to reply first.
    for offset in range(0, len(candidates), 300):
        batch = candidates[offset : offset + 300]
        with ThreadPoolExecutor(max_workers=WORKERS) as pool:
            results = list(pool.map(lambda row: validate_url(row["url"]), batch))
        for row, result in zip(batch, results):
            if not result:
                continue
            url, status = result
            if url in seen:
                continue
            seen.add(url)
            accepted_row = row.copy()
            accepted_row["url"] = url
            accepted_row["verified_at"] = TODAY
            accepted_row["provenance"] += f"; live-http={status}"
            accepted.append(accepted_row)
            if len(accepted) >= limit:
                return accepted
    return accepted


def write_csv(path: Path, rows: list[dict[str, str]]) -> None:
    fields = ["url", "label", "category", "region", "verified_at", "provenance"]
    with path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=fields)
        writer.writeheader()
        writer.writerows(rows)


def build_jobs() -> int:
    candidates: list[dict[str, str]] = []
    for ats, source_path in JOBS_INPUTS.items():
        data = fetch_json(JOBS_REPOSITORY, JOBS_COMMIT, source_path)
        for slug in scalar_values(data):
            url = job_url(ats, slug)
            if url:
                candidates.append(
                    {
                        "url": url,
                        "label": slug_label(slug),
                        "category": "enterprise-careers",
                        "region": "global",
                        "verified_at": "",
                        "provenance": f"{JOBS_REPOSITORY}@{JOBS_COMMIT}:{source_path}; ats={ats}",
                    }
                )
    rows = verify_candidates(candidates, 1000)
    write_csv(ROOT / "jobs-sources.csv", rows)
    return len(rows)


def build_events() -> int:
    data = fetch_json(EVENTS_REPOSITORY, EVENTS_COMMIT, "conferences.json")
    if not isinstance(data, list):
        raise ValueError("conference input is not a list")
    candidates = []
    for event in data:
        if not isinstance(event, dict) or not isinstance(event.get("url"), str):
            continue
        topics = event.get("audienceTypes") or event.get("tags") or ["general-tech"]
        category = str(topics[0]) if isinstance(topics, list) and topics else "general-tech"
        candidates.append(
            {
                "url": event["url"],
                "label": str(event.get("name") or event["url"]),
                "category": category,
                "region": str(event.get("region") or "global"),
                "verified_at": "",
                "provenance": f"{EVENTS_REPOSITORY}@{EVENTS_COMMIT}:conferences.json; official-event-url",
            }
        )
    for path in conference_paths():
        records = fetch_json(CONFS_TECH_REPOSITORY, CONFS_TECH_COMMIT, path)
        if not isinstance(records, list):
            continue
        category = Path(path).stem
        for event in records:
            if not isinstance(event, dict) or not isinstance(event.get("url"), str):
                continue
            candidates.append(
                {
                    "url": event["url"],
                    "label": str(event.get("name") or event["url"]),
                    "category": category,
                    "region": str(event.get("country") or ("online" if event.get("online") else "global")),
                    "verified_at": "",
                    "provenance": (
                        f"{CONFS_TECH_REPOSITORY}@{CONFS_TECH_COMMIT}:{path}; "
                        "official-event-url"
                    ),
                }
            )
    rows = verify_candidates(candidates, 1000)
    write_csv(ROOT / "events-sources.csv", rows)
    return len(rows)


def validate_catalog(path: Path, expected: int, category: str | None = None) -> list[str]:
    with path.open(newline="", encoding="utf-8") as handle:
        rows = list(csv.DictReader(handle))
    errors = []
    if len(rows) != expected:
        errors.append(f"{path.name}: expected {expected} rows, found {len(rows)}")
    urls = [row["url"] for row in rows]
    if len(urls) != len(set(urls)):
        errors.append(f"{path.name}: URLs are not unique")
    for row in rows:
        if category and row.get("category") != category:
            errors.append(f"{path.name}: invalid category {row.get('category')!r}")
        try:
            canonicalize(row["url"])
        except ValueError as error:
            errors.append(f"{path.name}: {error}")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--build", action="store_true", help="fetch, verify, and write both catalogs")
    parser.add_argument("--jobs", action="store_true", help="fetch, verify, and write only the jobs catalog")
    parser.add_argument("--events", action="store_true", help="fetch, verify, and write only the events catalog")
    parser.add_argument("--validate", action="store_true", help="validate row count and unique HTTP(S) URLs")
    args = parser.parse_args()
    if not any((args.build, args.jobs, args.events, args.validate)):
        parser.error("choose --build, --jobs, --events, or --validate")

    counts: dict[str, int] = {}
    if args.build or args.jobs:
        counts["jobs-sources.csv"] = build_jobs()
    if args.build or args.events:
        counts["events-sources.csv"] = build_events()
    if counts:
        print("built " + ", ".join(f"{name}={count}" for name, count in counts.items()))

    if args.validate:
        errors = []
        errors.extend(validate_catalog(ROOT / "jobs-sources.csv", 1000, "enterprise-careers"))
        errors.extend(validate_catalog(ROOT / "events-sources.csv", 1000))
        if errors:
            print("\n".join(errors), file=sys.stderr)
            return 1
        print("catalog validation passed: 1,000 unique HTTP(S) URLs per CSV")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
