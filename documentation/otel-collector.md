# OpenTelemetry Collector

## C'est quoi ?

L'OTel Collector est un **proxy/agrégateur de télémétrie**. Il reçoit les données (traces, métriques, logs) depuis tes applications, les transforme si besoin, et les envoie vers un ou plusieurs backends (Zipkin, Prometheus, Jaeger, etc.).

Sans collector, chaque service devrait connaître et parler directement à chaque backend. Avec le collector, chaque service envoie tout au même endroit (le collector), et c'est lui qui dispatch.

```
service-auth  ──┐
service-users ──┤──► OTel Collector ──► Zipkin (traces)
service-notes ──┤                   └──► Prometheus (métriques)
service-utils ──┘
```

## Les 3 concepts clés

### Receivers (entrées)
Ce que le collector accepte. Dans ce projet, on utilise OTLP (OpenTelemetry Protocol) :

```yaml
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317   # Les services Rust envoient ici
      http:
        endpoint: 0.0.0.0:4318   # Alternatif HTTP
```

OTLP est le protocole natif d'OpenTelemetry. Les crates Rust (`opentelemetry-otlp`) parlent directement ce protocole.

### Processors (transformation)
Ce que le collector fait avec les données avant de les exporter. Le processor `batch` regroupe les spans/métriques en lots pour envoyer moins de requêtes réseau :

```yaml
processors:
  batch:   # groupe les données, envoie par paquets
```

### Exporters (sorties)
Vers où le collector envoie les données :

```yaml
exporters:
  zipkin:
    endpoint: "http://zipkin:9411/api/v2/spans"   # traces → Zipkin
  prometheus:
    endpoint: "0.0.0.0:8889"   # métriques exposées, Prometheus vient scraper
  debug:
    verbosity: normal   # affiche dans les logs du collector (utile pour debug)
```

### Pipelines (assemblage)
On relie receivers → processors → exporters par type de signal :

```yaml
service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [batch]
      exporters: [zipkin, debug]   # les traces vont dans Zipkin ET dans les logs
    metrics:
      receivers: [otlp]
      processors: [batch]
      exporters: [prometheus]
```

## Pourquoi deux images collector ?

- `otel/opentelemetry-collector` : image officielle minimale
- `otel/opentelemetry-collector-contrib` : image avec tous les plugins community (dont l'exporter Zipkin, Prometheus, etc.)

Ce projet utilise `contrib` car l'exporter Zipkin n'est pas dans l'image de base.

## Comment vérifier que le collector reçoit des données ?

```bash
docker logs rusve-otel-collector
```

Tu devrais voir des lignes du type :
```
Traces    1          # nombre de traces reçues
Spans     5          # nombre de spans
```

Si tu vois `0` partout, les services n'arrivent pas à se connecter (vérifier que `OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317` est bien défini et que le container tourne).

## Port mapping

| Port hôte | Port container | Usage |
|-----------|---------------|-------|
| 4317      | 4317          | OTLP gRPC (services → collector) |
| 4318      | 4318          | OTLP HTTP |
| 8889      | 8889          | Prometheus scrape endpoint |
