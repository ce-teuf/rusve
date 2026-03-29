# Prometheus — Métriques

## C'est quoi ?

Prometheus est une **base de données de séries temporelles** (time-series) pour les métriques. Il collecte périodiquement des valeurs numériques (compteurs, jauges, histogrammes) et permet de les interroger.

Contrairement à Zipkin (traces ponctuelles), Prometheus répond à des questions comme :
- "Combien de requêtes par seconde mon service-auth reçoit-il ?"
- "Quelle est la latence p99 des appels gRPC vers service-notes ?"
- "Mon OTel Collector a-t-il droppé des spans ?"

## Le modèle pull (scraping)

Prometheus fonctionne en **pull** : c'est lui qui va chercher les métriques à intervalles réguliers, pas les services qui les poussent.

```
Prometheus ──► scrape http://otel-collector:8889/metrics toutes les 10s
               └── récupère les métriques au format texte Prometheus
```

Dans ce projet, le flux est :
```
Services Rust ──OTLP──► OTel Collector ──expose:8889──► Prometheus scrape
```

Le collector reçoit les métriques via OTLP et les expose au format Prometheus sur le port 8889. Prometheus vient les chercher.

### Configuration scraping (prometheus.yml)

```yaml
global:
  scrape_interval: 10s   # intervalle par défaut pour tous les jobs

scrape_configs:
  - job_name: otel-collector
    static_configs:
      - targets: ['otel-collector:8889']   # où scraper
```

`otel-collector` est le nom du service Docker, résolu par le réseau Docker.

## Les types de métriques

### Counter (compteur)
Valeur qui ne fait qu'augmenter. Ex: nombre total de requêtes.

```promql
otelcol_receiver_accepted_spans_total   # spans reçus par le collector
```

Pour avoir un taux : `rate(metric[5m])` = moyenne sur 5 minutes.

### Gauge (jauge)
Valeur qui monte et descend. Ex: mémoire utilisée, connexions actives.

### Histogram
Distribution de valeurs. Ex: latences (permet de calculer p50, p95, p99).

## Interroger avec PromQL

L'UI est accessible sur **http://localhost:9090**.

### Requêtes utiles pour ce projet

Vérifier que le collector reçoit des données :
```promql
otelcol_receiver_accepted_spans_total
```

Taux de spans acceptés (par seconde, moyenne sur 1 min) :
```promql
rate(otelcol_receiver_accepted_spans_total[1m])
```

Spans refusés (erreurs) :
```promql
otelcol_receiver_refused_spans_total
```

Métriques exportées vers Zipkin :
```promql
otelcol_exporter_sent_spans_total
```

### Syntaxe de base

```promql
# Filtrer par label
otelcol_receiver_accepted_spans_total{receiver="otlp"}

# Taux sur une fenêtre glissante
rate(metric_name[5m])

# Agréger plusieurs séries
sum(rate(metric_name[5m]))
```

## Alerting (pour info)

Prometheus peut déclencher des alertes via Alertmanager quand une condition est remplie. Non configuré dans ce projet (dev local), mais en prod on définirait des règles comme :
```yaml
- alert: HighErrorRate
  expr: rate(errors_total[5m]) > 0.1
```

## Persistance

Prometheus stocke ses données sur disque (dans `/prometheus` dans le container). Ce projet n'a pas de volume nommé pour Prometheus — les données sont perdues au redémarrage du container. Pour la prod, ajouter un volume.


Les 4 services compilent. Voici ce qui a été implémenté et comment observer dans Prometheus :                                                               
                                                                                                                                                              
  ---                                                                                                                                                         
  Ce qui a été ajouté                                                                                                                                         
                                                                                                                                                              
  2 métriques dans tous les services :                                                                                                                        
                                                                                                                                                              
  ┌─────────────────────────────────────────┬───────────┬───────────────────┐                                                                                 
  │                Métrique                 │   Type    │      Labels       │                                                                                 
  ├─────────────────────────────────────────┼───────────┼───────────────────┤                                                                                 
  │ grpc_requests_total                     │ Counter   │ {method, status}  │                                                                                 
  ├─────────────────────────────────────────┼───────────┼───────────────────┤
  │ grpc_request_duration_ms                │ Histogram │ {method}          │                                                                                 
  ├─────────────────────────────────────────┼───────────┼───────────────────┤
  │ http_requests_total (service-auth)      │ Counter   │ {handler, status} │                                                                                 
  ├─────────────────────────────────────────┼───────────┼───────────────────┤                                                                                 
  │ http_request_duration_ms (service-auth) │ Histogram │ {handler}         │
  └─────────────────────────────────────────┴───────────┴───────────────────┘                                                                                 
                                                            
  Les métriques sont envoyées toutes les 15 secondes via OTLP → OTel Collector → Prometheus.                                                                  
  
  ---                                                                                                                                                         
  Observer dans Prometheus (http://localhost:9090)          
                                                  
  Nombre total de requêtes par méthode :
  grpc_requests_total                                                                                                                                         
                     
  Taux de requêtes par seconde (fenêtre 1 min) :                                                                                                              
  rate(grpc_requests_total[1m])                                                                                                                               
                                                                                                                                                              
  Latence moyenne par méthode :                                                                                                                               
  rate(grpc_request_duration_ms_sum[1m]) / rate(grpc_request_duration_ms_count[1m])                                                                           
                                                                                   
  Requêtes en erreur seulement :                                                                                                                              
  grpc_requests_total{status="error"}                                                                                                                         
                                                                                                                                                              
  Latence p95 (histogramme) :                                                                                                                                 
  histogram_quantile(0.95, rate(grpc_request_duration_ms_bucket[5m]))    