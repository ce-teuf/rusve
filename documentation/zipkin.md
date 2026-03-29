# Zipkin — Traces distribuées

## C'est quoi ?

Zipkin est un système de **distributed tracing** : il te permet de visualiser le chemin complet d'une requête à travers plusieurs services.

Dans un système distribué, une action utilisateur (ex: "charger la page notes") peut déclencher :
1. Une requête HTTP vers service-auth
2. Un appel gRPC vers service-users
3. Un appel gRPC vers service-notes
4. service-notes appelle service-users pour chaque note

Sans tracing, si quelque chose est lent tu ne sais pas où. Zipkin te montre exactement quelle étape a pris combien de temps.

## Concepts fondamentaux

### Trace
Une **trace** représente le voyage complet d'une requête. Elle a un `trace_id` unique (128 bits).

```
Trace abc123
└── oauth_callback (service-auth, 245ms)
    └── create_user (service-users, 12ms)
```

### Span
Un **span** est une opération unitaire dans la trace. Chaque span contient :
- un nom (ex: `create_user`)
- le service qui l'a émis
- timestamps début/fin
- le `parent_span_id` (pour construire l'arbre)
- des tags/annotations optionnels

### Propagation W3C TraceContext
Pour relier les spans entre services, on propage le contexte via des headers HTTP ou metadata gRPC :

```
traceparent: 00-{trace_id}-{parent_span_id}-01
```

Dans ce projet, quand service-auth appelle service-users en gRPC, il injecte ce header dans la metadata tonic. service-users l'extrait et crée son span comme enfant. C'est ce que font `MetadataInjector` / `MetadataExtractor` dans les `lib.rs`.

## Utiliser l'UI Zipkin

Accès : **http://localhost:9411**

### Rechercher des traces

1. Cliquer sur le bouton **"Run Query"** (sans filtre) pour voir les traces récentes
2. Ou filtrer par :
   - **Service** : `service-auth`, `service-users`, etc.
   - **Span name** : nom d'une opération (ex: `create_user`)
   - **Duration** : traces qui ont pris plus de X ms (utile pour détecter les lenteurs)

### Lire une trace

Chaque trace s'affiche en **waterfall** (cascade) :

```
service-auth        oauth_callback          [████████████████] 245ms
  service-users       create_user             [██] 12ms
```

- La barre montre la durée relative
- L'indentation montre la hiérarchie parent/enfant
- Les couleurs distinguent les services

### Tags utiles

Dans ce projet, chaque span a un tag `rpc` défini par `#[tracing::instrument(fields(rpc = "nom"))]` qui indique le nom de la méthode gRPC.

## Pourquoi mes traces ne sont pas liées ?

Si tu vois des traces séparées au lieu d'une seule trace liée, c'est que la propagation du contexte ne fonctionne pas. Vérifier :

1. Le service émetteur appelle bien `prop.inject_context(...)` avant l'appel sortant
2. Le service récepteur appelle bien `prop.extract(...)` et `set_parent(parent_cx)`
3. `TraceContextPropagator::new()` est bien initialisé au démarrage (`init_tracer` le fait)

## Ce que Zipkin ne fait pas

- Zipkin ne stocke les traces que **temporairement** (en mémoire par défaut, disparaît au redémarrage)
- Ce n'est pas un outil de métriques (pas de graphes de latence dans le temps) → c'est Prometheus/Grafana pour ça
- Pour la prod, préférer un backend persistant (Jaeger avec stockage, Tempo dans Grafana Cloud, etc.)
