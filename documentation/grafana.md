# Grafana — Dashboards & Visualisation

## C'est quoi ?

Grafana est un outil de **visualisation**. Il ne stocke aucune donnée lui-même — il se connecte à des sources de données (Prometheus, Zipkin, Loki, etc.) et affiche des dashboards.

Dans ce projet, Grafana agrège :
- Les **métriques** depuis Prometheus
- Les **traces** depuis Zipkin

Accès : **http://localhost:3001** (accès anonyme admin configuré, pas de login nécessaire)

## Datasources préconfigurées

Les datasources sont provisionnées automatiquement au démarrage via `grafana/provisioning/datasources/datasources.yml` — tu n'as rien à configurer manuellement.

```yaml
datasources:
  - name: Prometheus
    type: prometheus
    url: http://prometheus:9090
    isDefault: true    # sélectionnée par défaut dans les panels

  - name: Zipkin
    type: zipkin
    url: http://zipkin:9411
```

`prometheus` et `zipkin` sont les noms des services Docker, résolus par le réseau Docker interne.

## Créer un dashboard

### Métriques (Prometheus)

1. Sidebar gauche → **Dashboards** → **New** → **New Dashboard**
2. **Add visualization**
3. Sélectionner datasource **Prometheus**
4. Dans le champ **Metrics**, taper une requête PromQL :

```promql
rate(otelcol_receiver_accepted_spans_total[1m])
```

5. Choisir le type de visualisation (Time series, Stat, Bar gauge, etc.)
6. **Apply** → **Save dashboard**

### Traces (Zipkin)

1. Sidebar gauche → **Explore** (icône boussole)
2. Sélectionner datasource **Zipkin**
3. Chercher par service, operation, tag, ou durée
4. Les traces s'affichent en waterfall comme dans l'UI Zipkin native

## Panels utiles pour ce projet

### Spans reçus par le collector
```promql
rate(otelcol_receiver_accepted_spans_total[1m])
```
Type : **Time series**

### Total spans traités
```promql
otelcol_receiver_accepted_spans_total
```
Type : **Stat** (affiche le chiffre total)

### Spans droppés (erreurs)
```promql
otelcol_receiver_refused_spans_total
```
Type : **Stat** avec threshold rouge si > 0

## Provisioning (automatisation)

Pour que Grafana démarre avec des dashboards préconfigurés (pas seulement des datasources), on peut aussi provisionner des dashboards en JSON dans `grafana/provisioning/dashboards/`. Non fait dans ce projet mais utile en équipe pour partager les dashboards via git.

Structure :
```
grafana/
  provisioning/
    datasources/
      datasources.yml    ← déjà créé
    dashboards/
      dashboards.yml     ← config du provider
      my-dashboard.json  ← export JSON d'un dashboard Grafana
```

Pour exporter un dashboard : **Dashboard** → icône partage → **Export** → **Save to file**.

## Pourquoi Grafana plutôt que l'UI Prometheus ?

L'UI Prometheus (`http://localhost:9090`) permet de faire des requêtes PromQL ad-hoc mais ne permet pas de sauvegarder des dashboards, ni de combiner plusieurs datasources dans une même vue. Grafana comble ce manque.

## Stack complète en un coup d'oeil

```
Services Rust
    │
    │  OTLP gRPC (traces + métriques)
    ▼
OTel Collector :4317
    ├── traces ──► Zipkin :9411 ◄── Grafana (datasource Zipkin)
    └── métriques exposées :8889
                    ▲
                    │ scrape toutes les 10s
                Prometheus :9090 ◄── Grafana (datasource Prometheus, défaut)
```
