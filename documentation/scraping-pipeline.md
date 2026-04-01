# Pipeline de Scraping — Architecture & Plan

## Contexte

Ce document décrit l'architecture recommandée pour ajouter un pipeline de scraping
à l'application Rusve. L'objectif est de collecter des données depuis des sources
externes, de les stocker dans une base intermédiaire (`db_scraping`), de les valider
via une interface admin, puis de pousser les données approuvées vers une base
principale (`db_data`).

---

## Architecture cible

```
┌──────────────────────────────────────────────────────────────┐
│  SCRAPER  (Python + Playwright)                              │
│  localhost CLI  OU  container Docker                         │
│  scrapers/run.py --source <url> --type <article|product|...>│
└────────────────────┬─────────────────────────────────────────┘
                     │ INSERT via psycopg (connexion directe DB)
                     ▼
┌──────────────────────────────────────────────────────────────┐
│  db_scraping  (PostgreSQL 18, port 5441)                     │
│  Tables : scrape_jobs, scrape_items                          │
│  Zone de staging — données brutes, non validées              │
└────────────────────┬─────────────────────────────────────────┘
                     │ gRPC (x-authorization JWT)
                     ▼
┌──────────────────────────────────────────────────────────────┐
│  service-scraper  (Rust / Tonic, port 50054)                 │
│  Lit db_scraping, valide, approuve, pousse vers db_data      │
└────────────────────┬─────────────────────────────────────────┘
                     │ gRPC (depuis SvelteKit server-side)
                     ▼
┌──────────────────────────────────────────────────────────────┐
│  SvelteKit  /admin/scraper  (role = ADMIN)                   │
│  Jobs list · Preview items · Approve/Reject · Push batch     │
└────────────────────┬─────────────────────────────────────────┘
                     │ push des items APPROVED
                     ▼
┌──────────────────────────────────────────────────────────────┐
│  db_data  (PostgreSQL 18, port 5442)                         │
│  Table : data_items (JSONB flexible, tracé vers db_scraping) │
└──────────────────────────────────────────────────────────────┘
```

---

## Pourquoi cette architecture

### db_scraping intermédiaire (staging)

| Bénéfice | Détail |
|---------|--------|
| **Isolation** | Les données brutes n'atteignent jamais la DB principale |
| **Rejeu** | Re-valider sans re-scraper |
| **Audit** | Historique complet, même des items rejetés |
| **Data quality gate** | Validation automatique + validation humaine avant push |
| **Rollback** | Push erroné → la source reste intacte dans db_scraping |

### Contrainte scraping : pas de Chromium

**Le scraping se fait exclusivement par :**
- **HTML/CSS classique** — `httpx` + `BeautifulSoup`/`lxml` pour parser le DOM statique
- **API interne** — analyse des appels réseau du site, appel direct aux endpoints JSON

Pas de Playwright, pas de Chromium, pas de rendu JS headless.

**Conséquence directe sur l'architecture :**

| Aspect | Avec Chromium | Sans Chromium (notre cas) |
|--------|--------------|--------------------------|
| RAM par scraper | 300–500 MB | ~20–50 MB |
| CPU | Spike important | Négligeable |
| Peut tourner sur le VPS | ⚠️ À brider | ✅ Sans contrainte |
| Détection anti-bot | Difficile à contourner | Headers HTTP suffisent |

→ **Le scraper peut tourner directement dans un container Docker sur le VPS** sans impact
sur les autres services. Le débat localhost vs VPS devient une question de commodité,
pas de ressources.

### Python pour le scraper

| Critère | Python | Rust |
|---------|--------|------|
| HTML statique (BS4/lxml) | ✅ | ✅ reqwest + scraper crate |
| API interne (JSON/REST) | ✅ httpx | ✅ reqwest + serde |
| Rapidité d'écriture | ✅ | ⚠️ Plus verbeux |
| Écosystème (headers, cookies, retry) | ✅ httpx natif | ✅ reqwest |

Les deux langages fonctionnent. Python reste recommandé pour la **rapidité d'itération**
(analyser une nouvelle API, tester des headers, parser un DOM) — Rust peut prendre le
relais si la performance devient un critère.

### service-scraper en Rust

Suit exactement le pattern des 4 services existants (lib.rs · migrations.rs ·
*_service.rs · *_db.rs). Avantages : typage fort pour la validation, intégration
OTel/gRPC native, même pattern d'auth JWT.

### Où faire tourner le scraper

Sans Chromium, les trois options sont toutes viables :

| Mode | Usage | Avantage |
|------|-------|---------|
| **Container VPS** (recommandé prod) | Cron auto, tout dans le même réseau Docker | Simplicité — accès direct à db_scraping sans exposer de port |
| **Localhost** | Dev, debug, analyse d'une nouvelle source | Feedback immédiat, logs dans le terminal |
| **GitHub Actions** | Si tu veux zéro charge sur le VPS | Gratuit, mais nécessite d'exposer db_scraping ou une API intermédiaire |

---

## Modes d'intégration par source

Chaque source (site scrapé) possède sa propre configuration d'intégration.
L'admin définit ces règles une seule fois dans l'interface — elles s'appliquent
ensuite à tous les jobs de cette source.

### Deux modes

| Mode | Comportement | Usage typique |
|------|-------------|---------------|
| **AUTO** | Les items qui passent les contraintes sont poussés automatiquement selon un schedule cron | Source fiable, données bien structurées, volume élevé |
| **MANUAL** | Tous les items attendent une validation humaine avant le push | Source peu connue, données critiques, modération nécessaire |

### Cycle de vie selon le mode

```
───── Mode AUTO ──────────────────────────────────────────────────────────────

PENDING  ──► check field_rules (à la création de l'item)
               ├── toutes les règles OK  ──► VALID
               │                                └──► push auto à 2h00 (cron) ──► PUSHED
               └── au moins une règle KO ──► INVALID
                                                └──► visible dans l'UI admin
                                                     (override possible → APPROVED → PUSHED)

───── Mode MANUAL ────────────────────────────────────────────────────────────

PENDING  ──► validation auto (field_rules, pour info seulement)
               ├── OK   ──► VALID    ┐
               └── KO   ──► INVALID  ┘  attendent une action admin
                                            ├── APPROVED ──► push manuel ──► PUSHED
                                            └── REJECTED ──► archivé
```

> En mode AUTO, les items INVALID ne bloquent pas le pipeline — ils restent
> dans db_scraping pour audit. L'admin peut les corriger manuellement si besoin.

---

### Table `scrape_sources` — registre des sources

```sql
CREATE TABLE scrape_sources (
    id               UUID PRIMARY KEY,
    created          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated          TIMESTAMPTZ NOT NULL DEFAULT now(),
    name             TEXT NOT NULL,                     -- label lisible : "Mon Blog"
    source_url       TEXT NOT NULL UNIQUE,
    source_type      TEXT NOT NULL,                     -- 'article', 'product', ...
    integration_mode TEXT NOT NULL DEFAULT 'MANUAL',    -- 'AUTO' | 'MANUAL'
    auto_schedule    TEXT NOT NULL DEFAULT '',           -- cron : '0 2 * * *' = 2h00
    field_rules      JSONB NOT NULL DEFAULT '[]',       -- contraintes (voir ci-dessous)
    active           BOOLEAN NOT NULL DEFAULT true
);
```

### Structure `field_rules` (JSONB)

Tableau de contraintes appliquées à chaque item scraped depuis cette source.
La logique : **si un champ `required` est absent ou invalide → item = INVALID**.

```json
[
  { "field": "title",        "required": true  },
  { "field": "url",          "required": true,  "format": "url" },
  { "field": "content",      "required": true,  "min_length": 100 },
  { "field": "published_at", "required": false, "format": "date_iso" },
  { "field": "price",        "required": false, "type": "number", "min": 0 }
]
```

| Contrainte | Type | Description |
|-----------|------|-------------|
| `required` | bool | Le champ doit être présent et non vide |
| `format` | string | `"url"` · `"email"` · `"date_iso"` |
| `type` | string | `"number"` · `"boolean"` |
| `min_length` | int | Longueur minimale (strings) |
| `min` / `max` | number | Borne numérique |

---

### Scheduler (service-scraper)

Au démarrage de `service-scraper`, toutes les sources `AUTO` avec un `auto_schedule`
sont enregistrées dans un scheduler Tokio (crate `tokio-cron-scheduler`).

```
Démarrage service-scraper
  └── charge toutes les sources active=true, mode=AUTO
        └── pour chaque source : enregistre un cron job
              └── à l'heure programmée :
                    1. sélectionne tous les items VALID de cette source
                    2. pousse vers db_data
                    3. marque PUSHED dans db_scraping
                    4. log OTel (metric: auto_push_total, items count)

Quand une source est modifiée via l'UI admin :
  └── gRPC UpdateSource → service-scraper recharge son scheduler
```

Crate à ajouter dans `service-scraper/Cargo.toml` :
```toml
tokio-cron-scheduler = "0.13"
```

---

## Schéma db_scraping

```sql
CREATE TABLE scrape_sources (
    id               UUID PRIMARY KEY,
    created          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated          TIMESTAMPTZ NOT NULL DEFAULT now(),
    name             TEXT NOT NULL,
    source_url       TEXT NOT NULL UNIQUE,
    source_type      TEXT NOT NULL,
    integration_mode TEXT NOT NULL DEFAULT 'MANUAL',
    auto_schedule    TEXT NOT NULL DEFAULT '',
    field_rules      JSONB NOT NULL DEFAULT '[]',
    active           BOOLEAN NOT NULL DEFAULT true
);

CREATE TABLE scrape_jobs (
    id          UUID PRIMARY KEY,
    created     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated     TIMESTAMPTZ NOT NULL DEFAULT now(),
    source_id   UUID REFERENCES scrape_sources(id),  -- NULL si job manuel sans source
    source_url  TEXT NOT NULL,
    source_type TEXT NOT NULL,
    status      TEXT NOT NULL,          -- RUNNING | DONE | FAILED
    item_count  INT NOT NULL DEFAULT 0,
    error       TEXT NOT NULL DEFAULT ''
);

CREATE TABLE scrape_items (
    id                UUID PRIMARY KEY,
    created           TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated           TIMESTAMPTZ NOT NULL DEFAULT now(),
    job_id            UUID NOT NULL REFERENCES scrape_jobs(id),
    raw_data          JSONB NOT NULL,
    validation_status TEXT NOT NULL DEFAULT 'PENDING',
    validation_errors JSONB NOT NULL DEFAULT '[]',
    pushed_at         TIMESTAMPTZ,
    pushed_target     TEXT NOT NULL DEFAULT ''
);
```

`raw_data JSONB` est volontairement flexible : chaque scraper peut pousser
n'importe quelle structure. C'est `service-scraper` qui applique les `field_rules`
de la source associée.

---

## Pipeline data quality — résumé des statuts

Le statut d'un item évolue selon le mode de sa source (voir section ci-dessus).
Dans les deux cas, les statuts possibles sont :

| Statut | Description |
|--------|-------------|
| `PENDING` | Vient d'être scrapé, field_rules pas encore évalués |
| `VALID` | Toutes les contraintes `required` sont satisfaites |
| `INVALID` | Au moins une contrainte `required` a échoué (`validation_errors` rempli) |
| `APPROVED` | Validé par un admin (mode MANUAL) ou en attente de push auto (mode AUTO) |
| `REJECTED` | Rejeté par un admin — archivé, ne sera jamais poussé |
| `PUSHED` | Intégré dans `db_data` (`pushed_at` + `pushed_target` remplis) |

Les `field_rules` sont évaluées dans `service-scraper/src/scraper_validation.rs`
en lisant la config de la `scrape_source` associée au job.

---

## db_data — base principale

Table générique au départ, normalisable plus tard selon le use case :

```sql
CREATE TABLE data_items (
    id             UUID PRIMARY KEY,
    created        TIMESTAMPTZ NOT NULL DEFAULT now(),
    source_type    TEXT NOT NULL,
    data           JSONB NOT NULL,          -- données validées et approuvées
    scrape_item_id UUID NOT NULL            -- traçabilité vers db_scraping
);
```

Port : **5442**. Ne pas réutiliser `db_notes` ou `db_users` — garder la
séparation des responsabilités. `db_data` peut évoluer vers des tables
métier normalisées une fois le modèle de données stabilisé.

---

## Contrat Protobuf — `proto/scraper.proto`

```protobuf
syntax = "proto3";
package scraper;

service ScraperService {
    // ── Gestion des sources (config par site) ─────────────────────
    rpc ListSources     (Empty)   returns (stream Source);
    rpc GetSource       (Id)      returns (Source);
    rpc CreateSource    (Source)  returns (Source);
    rpc UpdateSource    (Source)  returns (Source);  // recharge le scheduler
    rpc DeleteSource    (Id)      returns (Empty);

    // ── Jobs ──────────────────────────────────────────────────────
    rpc ListJobs        (Page)        returns (stream JobResponse);
    rpc GetJobById      (Id)          returns (Job);

    // ── Items ─────────────────────────────────────────────────────
    rpc ListItems       (ItemFilter)  returns (stream ItemResponse);
    rpc ApproveItem     (Id)          returns (Item);
    rpc RejectItem      (Id)          returns (Item);
    rpc ApproveAllValid (Id)          returns (Count);  // tous VALID d'un job
    rpc PushApproved    (Id)          returns (Count);  // push manuel des APPROVED
}

message Source {
    string id               = 1;
    string created          = 2;
    string updated          = 3;
    string name             = 4;
    string source_url       = 5;
    string source_type      = 6;
    string integration_mode = 7;  // "AUTO" | "MANUAL"
    string auto_schedule    = 8;  // cron : "0 2 * * *" = tous les jours à 2h
    string field_rules      = 9;  // JSON sérialisé
    bool   active           = 10;
}

message ItemFilter {
    string job_id  = 1;
    string status  = 2;  // "" = tous | "PENDING" | "VALID" | "INVALID" | ...
    int32  offset  = 3;
    int32  limit   = 4;
}
```

---

## SvelteKit — Admin UI

Nouveau groupe de routes `(admin)`, protégé au niveau du layout :

```
src/routes/(admin)/
  +layout.server.ts             ← redirect si user.role !== ADMIN (2)
  scraper/
    +page.svelte                ← dashboard : jobs récents + stats par source
    +page.server.ts             ← rpc ListJobs + ListSources
    sources/
      +page.svelte              ← liste des sources configurées
      +page.server.ts           ← rpc ListSources
      [sourceId]/
        +page.svelte            ← config source : mode, schedule, field_rules
        +page.server.ts         ← rpc GetSource + UpdateSource
    [jobId]/
      +page.svelte              ← items filtrables, actions item et batch
      +page.server.ts           ← rpc GetJobById + ListItems
```

**Vues :**

- **Dashboard** (`/admin/scraper`) : tableau des jobs récents, stats VALID/INVALID/PUSHED par source, badge mode AUTO/MANUAL
- **Sources list** (`/admin/scraper/sources`) : nom · URL · mode (badge AUTO vert / MANUAL orange) · schedule humain ("Tous les jours à 2h") · toggle actif/inactif
- **Source config** (`/admin/scraper/sources/[sourceId]`) :
  - Toggle `AUTO` / `MANUAL`
  - Champ cron avec préview lisible ("0 2 * * *" → "Tous les jours à 2h00")
  - Éditeur de `field_rules` : tableau avec add/remove/configure (champ · required · format · min_length...)
  - Bouton "Save" → gRPC UpdateSource → rechargement du scheduler
- **Job detail** (`/admin/scraper/[jobId]`) : barre de filtres par statut · préview JSON formaté · approve/reject item · "Approve all valid" · "Push approved" (MANUAL uniquement — en AUTO le push est automatique)

---

## Fichiers à créer

| Chemin | Description |
|--------|-------------|
| `services/service-scraper/` | Nouveau service Rust (calqué sur service-notes) + dep `tokio-cron-scheduler` |
| `proto/scraper.proto` | Définitions gRPC scraper |
| `scrapers/run.py` | Script Python principal |
| `scrapers/requirements.txt` | Dépendances Python |
| `scrapers/Dockerfile` | Image pour mode Docker |
| `docker-compose.scraper.yml` | db_scraping + scraper container |
| `clients/webapp/src/routes/(admin)/` | Admin UI SvelteKit |
| `documentation/scraping-pipeline.md` | Ce fichier |

## Fichiers à modifier

| Chemin | Changement |
|--------|-----------|
| `docker-compose.db.yml` | Ajouter db_scraping (5441) et db_data (5442) |
| `docker-compose.app.yml` | Ajouter service-scraper |
| `docker-compose.prod.yml` | Ajouter service-scraper + db_scraping + db_data |
| `proto/main.proto` | Import scraper.proto |
| `proto.sh` | Générer bindings scraper |
| `Makefile` | Targets : `db-scraping`, `scraper`, `watch-scraper` |
| `clients/webapp/src/lib/server/grpc.ts` | Ajouter `scraperService` |
| `CLAUDE.md` | Documenter service-scraper |

---

## Ordre d'implémentation recommandé

```
Phase 1 — Infrastructure
  1. docker-compose.db.yml   → db_scraping (5441) + db_data (5442)
  2. Makefile                → targets db-scraping, scraper

Phase 2 — Scraper Python
  3. scrapers/run.py         → scrape + INSERT scrape_jobs/scrape_items dans db_scraping
  4. scrapers/Dockerfile     → image production

Phase 3 — service-scraper (Rust)
  5. proto/scraper.proto     → Source + Job + Item + RPCs → proto.sh
  6. service-scraper/        → migrations (scrape_sources + scrape_jobs + scrape_items)
  7. service-scraper/        → gRPC CRUD sources + list jobs/items + approve/reject/push
  8. service-scraper/        → scheduler tokio-cron-scheduler (auto push)
  9. docker-compose.app.yml  → ajouter service-scraper

Phase 4 — Admin UI (SvelteKit)
  10. (admin)/+layout.server.ts         → guard ADMIN
  11. (admin)/scraper/sources/          → CRUD sources + config mode/schedule/field_rules
  12. (admin)/scraper/                  → dashboard jobs + stats
  13. (admin)/scraper/[jobId]/          → préview items + actions approve/reject/push

Phase 5 — Push vers db_data
  14. db_data schema   → data_items table
  15. PushApproved     → écrit dans db_data, marque PUSHED dans db_scraping
  16. Auto-push logic  → scheduler appelle PushApproved pour les sources AUTO
```

---

## Variables d'environnement — service-scraper

```bash
PORT=50054
RUST_LOG=info
DATABASE_URL=postgres://postgres:12345@rusve-db-scraping:5432/scraping
DATA_DATABASE_URL=postgres://postgres:12345@rusve-db-data:5432/data
JWT_SECRET=<same as other services>
OTEL_EXPORTER_OTLP_ENDPOINT=http://otel-collector:4317
```

## Variables d'environnement — scraper Python

```bash
SCRAPING_DB_URL=postgresql://postgres:12345@localhost:5441/scraping
# ou depuis Docker :
SCRAPING_DB_URL=postgresql://postgres:12345@rusve-db-scraping:5432/scraping
```
