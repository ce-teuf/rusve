#!/usr/bin/env python3
"""
Rusve scraper — static HTML or internal API scraping.
No Chromium/Playwright — httpx + BeautifulSoup only.

Usage:
    python run.py --url <URL> --type <source_type> [--source-id <UUID>]

Examples:
    # Scrape an HTML page
    python run.py --url https://example.com/articles --type article

    # Call an internal JSON API
    python run.py --url https://api.example.com/products --type product --api

Environment:
    SCRAPING_DB_URL   PostgreSQL connection string for db_scraping
                      Default: postgresql://postgres:12345@localhost:5441/scraping
"""

import argparse
import json
import os
import sys
import uuid
from datetime import datetime, timezone

import httpx
import psycopg
from bs4 import BeautifulSoup


DB_URL = os.environ.get(
    "SCRAPING_DB_URL",
    "postgresql://postgres:12345@localhost:5441/scraping",
)

HEADERS = {
    "User-Agent": (
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 "
        "(KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36"
    ),
    "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
    "Accept-Language": "en-US,en;q=0.5",
}


# ── DB helpers ───────────────────────────────────────────────

def create_job(conn, source_url: str, source_type: str, source_id: str | None) -> str:
    job_id = str(uuid.uuid4())
    conn.execute(
        """
        INSERT INTO scrape_jobs (id, source_id, source_url, source_type, status)
        VALUES (%s, %s, %s, %s, 'RUNNING')
        """,
        (job_id, source_id, source_url, source_type),
    )
    return job_id


def finish_job(conn, job_id: str, item_count: int, error: str = ""):
    status = "FAILED" if error else "DONE"
    conn.execute(
        "UPDATE scrape_jobs SET status=%s, item_count=%s, error=%s WHERE id=%s",
        (status, item_count, error, job_id),
    )


def insert_item(conn, job_id: str, raw_data: dict) -> str:
    item_id = str(uuid.uuid4())
    conn.execute(
        """
        INSERT INTO scrape_items (id, job_id, raw_data, validation_status)
        VALUES (%s, %s, %s, 'PENDING')
        """,
        (item_id, job_id, json.dumps(raw_data)),
    )
    return item_id


# ── Scrapers ─────────────────────────────────────────────────

def scrape_html(url: str) -> list[dict]:
    """
    Minimal HTML scraper — parses the page and returns a list of raw dicts.
    Adapt the selectors to the actual target site structure.
    """
    with httpx.Client(headers=HEADERS, follow_redirects=True, timeout=30) as client:
        resp = client.get(url)
        resp.raise_for_status()

    soup = BeautifulSoup(resp.text, "lxml")
    items = []

    # Generic fallback: scrape all <article> or <div class="item"> blocks.
    # Replace with site-specific selectors.
    blocks = soup.find_all("article") or soup.find_all(class_="item")
    if not blocks:
        # Last resort: treat the whole page as a single item
        blocks = [soup]

    for block in blocks:
        title_el = block.find(["h1", "h2", "h3"])
        link_el = block.find("a", href=True)
        content_el = block.find("p")

        item = {
            "scraped_at": datetime.now(timezone.utc).isoformat(),
            "source_url": url,
            "title": title_el.get_text(strip=True) if title_el else "",
            "url": link_el["href"] if link_el else url,
            "content": content_el.get_text(strip=True) if content_el else "",
        }
        items.append(item)

    return items


def scrape_api(url: str) -> list[dict]:
    """
    Fetches a JSON endpoint and wraps the response.
    The API response can be a list or an object with a list field.
    """
    with httpx.Client(headers={**HEADERS, "Accept": "application/json"}, follow_redirects=True, timeout=30) as client:
        resp = client.get(url)
        resp.raise_for_status()
        data = resp.json()

    if isinstance(data, list):
        return [{"scraped_at": datetime.now(timezone.utc).isoformat(), **item} for item in data]

    # Common API patterns: {"data": [...]} or {"results": [...]} or {"items": [...]}
    for key in ("data", "results", "items", "records"):
        if key in data and isinstance(data[key], list):
            return [{"scraped_at": datetime.now(timezone.utc).isoformat(), **item} for item in data[key]]

    # Wrap the whole response as a single item
    return [{"scraped_at": datetime.now(timezone.utc).isoformat(), **data}]


# ── Main ─────────────────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(description="Rusve scraper")
    parser.add_argument("--url", required=True, help="Target URL to scrape")
    parser.add_argument("--type", required=True, dest="source_type", help="Source type (article, product, ...)")
    parser.add_argument("--source-id", default=None, help="UUID of the scrape_sources row (optional)")
    parser.add_argument("--api", action="store_true", help="Target is a JSON API endpoint")
    args = parser.parse_args()

    print(f"[scraper] Connecting to DB: {DB_URL}")
    print(f"[scraper] Target: {args.url} (type={args.source_type}, api={args.api})")

    with psycopg.connect(DB_URL) as conn:
        conn.autocommit = False
        job_id = create_job(conn, args.url, args.source_type, args.source_id)
        print(f"[scraper] Job created: {job_id}")

        try:
            if args.api:
                raw_items = scrape_api(args.url)
            else:
                raw_items = scrape_html(args.url)

            count = 0
            for raw in raw_items:
                insert_item(conn, job_id, raw)
                count += 1

            finish_job(conn, job_id, count)
            conn.commit()
            print(f"[scraper] Done — {count} items inserted")

        except Exception as exc:
            error_msg = str(exc)
            print(f"[scraper] Error: {error_msg}", file=sys.stderr)
            finish_job(conn, job_id, 0, error=error_msg)
            conn.commit()
            sys.exit(1)


if __name__ == "__main__":
    main()
