# Migration Capacitor — Porter l'app sur mobile

## Pourquoi supprimer le SSR

C'est la question centrale. Pour la comprendre, il faut voir comment fonctionne chacun des deux modèles.

### Comment fonctionne le SSR actuel

Avec `adapter-node`, chaque requête passe par un serveur Node.js :

```
Navigateur → Caddy → Node.js (SvelteKit)
                          ↓
                      hooks.server.ts   (valide le JWT via gRPC Auth)
                      +page.server.ts   (charge les données via gRPC)
                          ↓ gRPC
                      service-users / service-notes / service-utils
                          ↓
                      renvoie HTML complet + JSON sérialisé → navigateur
```

Le serveur Node.js est au cœur du système. Il fait tourner :
- `hooks.server.ts` — validé le JWT à chaque requête en appelant `usersService.Auth()`
- `+page.server.ts` — charge les données en appelant les services gRPC directement
- `@grpc/grpc-js` — bibliothèque Node.js qui parle le protocole gRPC (bindings C++ natifs)
- `$env/static/private` — variables secrètes (JWT_SECRET, USERS_URI...) disponibles uniquement côté serveur

### Comment fonctionne Capacitor en mode bundle

Capacitor copie les fichiers web **dans** l'APK/IPA lors de la compilation :

```
APK contient :
  ├── index.html
  ├── assets/
  │   ├── bundle.js      ← tout le code JS compilé
  │   └── bundle.css
  └── (pas de Node.js, pas de serveur)
```

Quand l'app s'ouvre, la WebView du téléphone charge ces fichiers. La WebView est un **navigateur embarqué** — identique à Chrome ou Safari. Elle peut exécuter du JavaScript standard, faire des requêtes HTTP, afficher du HTML/CSS.

Ce qu'elle **ne peut pas faire** :

| Ce que le SSR utilise | Pourquoi ça ne marche pas dans une WebView |
|-----------------------|-------------------------------------------|
| `hooks.server.ts` | Requiert Node.js — ne tourne pas dans un navigateur |
| `+page.server.ts` | Même raison — fichier inexistant à l'exécution |
| `@grpc/grpc-js` | Module Node.js avec bindings C++ — impossible à compiler pour navigateur |
| `$env/static/private` | Ces secrets seraient compilés dans le bundle JS → **fuite de sécurité** |
| Cookies HttpOnly | Créés par le serveur via `Set-Cookie` — pas de serveur = pas de cookie |
| `redirect()` SvelteKit | Réponse HTTP 302 — nécessite un serveur pour l'envoyer |

**En résumé** : SSR = le serveur EST l'application. Capacitor bundle = le fichier JS EST l'application. Ces deux modèles sont incompatibles par nature.

---

## Architecture actuelle vs architecture cible

### Actuelle (web SSR)

```
Navigateur
    ↓ HTTP
Caddy (reverse proxy)
    ├── /oauth-login/*  → service-auth (Axum :8090)
    ├── /oauth-callback/* → service-auth (Axum :8090)
    └── /*             → SvelteKit Node.js (:3000)
                              ↓ gRPC
                        service-users (:50051)
                        service-notes (:50052)
                        service-utils (:50053)
```

### Phase 1 — Web Wrapper (zéro code)

```
WebView Capacitor
    ↓ charge https://monapp.com  (server.url dans capacitor.config.ts)
Caddy → SvelteKit SSR → gRPC
         (identique au web, aucune modification)
```

### Phase 2 — Full Native

```
WebView Capacitor
    ↓ bundle statique embarqué dans APK (index.html + JS)
    ↓ fetch('https://monapp.com/api/notes', { Authorization: Bearer <token> })
Caddy
    ├── /oauth-login/*    → service-auth
    ├── /oauth-callback/* → service-auth
    ├── /api/*            → SvelteKit Node.js (BFF, API REST uniquement)
    └── /*                → SvelteKit Node.js (SSR pour web)
                                ↓ gRPC (inchangé)
                          services Rust
```

---

## Phase 1 — Web Wrapper

La façon la plus rapide d'avoir l'app sur mobile. Capacitor propose `server.url` : la WebView charge l'app hébergée sur HTTPS au lieu du bundle local. L'app reste 100% SSR, aucun code à changer.

### Prérequis

- Un domaine public avec HTTPS (ex: `monapp.com`)
- Caddy gère TLS automatiquement via Let's Encrypt (gratuit, sans configuration SSL)

### 1. Configurer Caddy avec HTTPS

```
# Caddyfile (production)
{
    email votre@email.com
}

monapp.com {
    reverse_proxy /oauth-login/*    localhost:8090
    reverse_proxy /oauth-callback/* localhost:8090
    reverse_proxy                   localhost:3000
}
```

Caddy obtient automatiquement un certificat Let's Encrypt pour `monapp.com`. Aucune configuration SSL supplémentaire.

### 2. Mettre à jour capacitor.config.ts

```typescript
import { CapacitorConfig } from '@capacitor/cli';

const config: CapacitorConfig = {
  appId: 'com.rusve.app',
  appName: 'Rusve',
  webDir: 'clients/webapp/build',   // utilisé si server.url est absent (dev offline)
  server: {
    url: 'https://monapp.com',       // WebView charge cette URL
    cleartext: false,
    androidScheme: 'https',
  },
};

export default config;
```

### 3. Build et déployer

```bash
# Installer les dépendances Capacitor
./scripts/capacitor.sh install

# Initialiser Android
./scripts/capacitor.sh add:android

# Sync
./scripts/capacitor.sh sync

# Ouvrir Android Studio pour signer et déployer
./scripts/capacitor.sh open:android
```

### Avantages et limites

**Avantages :**
- Déployable en quelques heures
- Toutes les features web disponibles immédiatement
- Mises à jour instantanées (pas de resoumission App Store pour chaque changement)
- OAuth fonctionne tel quel

**Limites :**
- Nécessite une connexion internet (pas de mode offline)
- Accès limité aux APIs natives (caméra, GPS, notifications push nécessitent des plugins supplémentaires)
- Performance identique au web (pas d'optimisation native)

---

## Phase 2 — Full Native

Pour un accès complet aux APIs natives (caméra, push notifications, GPS, stockage sécurisé) et la possibilité d'un mode offline, il faut un bundle statique dans l'APK.

Le SvelteKit server continue de tourner, mais joue le rôle de **BFF (Backend for Frontend)** : il expose une API REST que le bundle statique appelle via `fetch()`. Le gRPC reste côté serveur, caché derrière l'API.

### Étape 1 — Ajouter les routes REST dans SvelteKit

Créer `clients/webapp/src/routes/api/` avec les endpoints suivants. La logique est identique aux `+page.server.ts` existants — on la déplace juste dans des handlers REST qui acceptent `Authorization: Bearer <token>` au lieu de lire les cookies.

| Fichier | Méthodes | Source |
|---------|----------|--------|
| `api/notes/+server.ts` | GET (liste), POST (créer) | notes/+page.server.ts |
| `api/notes/[id]/+server.ts` | GET (détail), PUT (MAJ), DELETE | notes/[noteId]/+page.server.ts |
| `api/files/+server.ts` | GET (liste), POST (upload) | files/+page.server.ts |
| `api/files/[id]/+server.ts` | GET (download), DELETE | files/+page.server.ts |
| `api/emails/+server.ts` | GET (liste), POST (envoyer) | emails/+page.server.ts |
| `api/profile/+server.ts` | GET, POST | profile/+page.server.ts |
| `api/subscription/+server.ts` | POST (checkout, portal) | subscription/+page.server.ts |

**Pattern type** (`api/notes/+server.ts`) :

```typescript
import { json, error } from '@sveltejs/kit';
import { grpcSafe, safe } from '$lib/safe';
import { notesService } from '$lib/server/grpc';
import { createMetadata } from '$lib/server/metadata';
import type { RequestHandler } from './$types';

// Extrait le token du header Authorization
function extractToken(request: Request): string {
    return request.headers.get('Authorization')?.replace('Bearer ', '') ?? '';
}

export const GET: RequestHandler = async ({ request, url }) => {
    const token = extractToken(request);
    if (!token) throw error(401, 'Unauthorized');

    const metadata = await createMetadata(token);
    const offset = Number(url.searchParams.get('p') ?? 1) - 1;
    const limit = 10;

    const [count, notesResult] = await Promise.all([
        new Promise<...>((r) => { notesService.CountNotesByUserId({}, metadata, grpcSafe(r)); }),
        safe(new Promise<...>((res, rej) => {
            const stream = notesService.GetNotesByUserId({ offset: offset * limit, limit }, metadata);
            const notes = [];
            stream.on('data', (note) => notes.push(note));
            stream.on('error', rej);
            stream.on('end', () => res(notes));
        }))
    ]);

    if (count.error || notesResult.error) throw error(500, 'Failed to fetch notes');

    return json({ notes: notesResult.data, total: Number(count.data.count), pageSize: limit });
};

export const POST: RequestHandler = async ({ request }) => {
    const token = extractToken(request);
    if (!token) throw error(401, 'Unauthorized');

    const metadata = await createMetadata(token);
    const body = await request.json();

    const result = await new Promise<...>((r) => {
        notesService.CreateNote(body, metadata, grpcSafe(r));
    });

    if (result.error) throw error(400, result.msg);
    return json(result.data);
};
```

### Étape 2 — Gérer l'auth côté mobile

Le flow OAuth actuel redirige vers `https://monapp.com/?token=JWT`, puis `hooks.server.ts` place le token dans un cookie HttpOnly. Ce mécanisme ne fonctionne pas dans un bundle statique (pas de `hooks.server.ts`).

**Solution : Deep links + @capacitor/preferences**

#### a) Installer les plugins

```bash
cd clients/webapp
pnpm add @capacitor/preferences @capacitor/app
```

#### b) Configurer le deep link Android/iOS

Dans `capacitor.config.ts` :
```typescript
plugins: {
    App: {
        appUrlOpen: 'com.rusve.app://callback'
    }
}
```

Dans `android/app/src/main/AndroidManifest.xml` (généré par Capacitor, à ajouter) :
```xml
<intent-filter android:autoVerify="true">
    <action android:name="android.intent.action.VIEW" />
    <category android:name="android.intent.category.DEFAULT" />
    <category android:name="android.intent.category.BROWSABLE" />
    <data android:scheme="com.rusve.app" android:host="callback" />
</intent-filter>
```

#### c) Modifier service-auth pour le deep link

Dans `services/service-auth/src/auth_service.rs`, modifier `oauth_callback` pour détecter si la requête vient de l'app mobile :

```rust
// Après avoir obtenu le token JWT :
let redirect_uri = if state.env.is_mobile_request(&query) {
    // Deep link pour l'app mobile
    format!("com.rusve.app://callback?token={}", token.id)
} else {
    // Web classique
    format!("{}/?token={}", state.env.client_url, token.id)
};
Ok(Redirect::to(&redirect_uri))
```

Le paramètre `?mobile=true` est ajouté lors de l'initiation du flow OAuth depuis l'app mobile et propagé via le `state` CSRF (qui survit au redirect OAuth).

#### d) Écouter le deep link dans la page auth

Modifier `clients/webapp/src/routes/auth/+page.svelte` :

```typescript
import { App } from '@capacitor/app';
import { Preferences } from '@capacitor/preferences';
import { goto } from '$app/navigation';
import { Capacitor } from '@capacitor/core';

// Écouter le deep link quand Capacitor est disponible
if (Capacitor.isNativePlatform()) {
    App.addListener('appUrlOpen', async ({ url }) => {
        const params = new URL(url).searchParams;
        const token = params.get('token');
        if (token) {
            await Preferences.set({ key: 'token', value: token });
            goto('/dashboard');
        }
    });
}

// Modifier onLogin pour ajouter ?mobile=true
async function onLogin(provider: string): Promise<void> {
    const suffix = Capacitor.isNativePlatform() ? '?mobile=true' : '';
    window.location.href = `${PUBLIC_AUTH_URL}/oauth-login/${provider}${suffix}`;
}
```

#### e) Utilitaire pour les requêtes authentifiées

Créer `clients/webapp/src/lib/mobile/api.ts` :

```typescript
import { Capacitor } from '@capacitor/core';
import { Preferences } from '@capacitor/preferences';

// Récupère le token selon la plateforme
export async function getToken(): Promise<string> {
    if (Capacitor.isNativePlatform()) {
        const { value } = await Preferences.get({ key: 'token' });
        return value ?? '';
    }
    // Sur web, pas besoin — le cookie est envoyé automatiquement
    return '';
}

// fetch() avec Authorization header pour mobile
export async function apiFetch(path: string, options: RequestInit = {}): Promise<Response> {
    const token = await getToken();
    return fetch(path, {
        ...options,
        headers: {
            ...options.headers,
            ...(token ? { 'Authorization': `Bearer ${token}` } : {}),
            'Content-Type': 'application/json',
        }
    });
}
```

### Étape 3 — Pages client-side pour mobile

SvelteKit permet d'avoir à la fois `+page.server.ts` (SSR) et `+page.ts` (universel). Pour le build mobile, on utilise `+page.ts` qui appelle l'API REST.

**Exemple pour notes :**

```typescript
// clients/webapp/src/routes/(app)/notes/+page.ts
import { apiFetch } from '$lib/mobile/api';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ url, fetch: kitFetch }) => {
    const page = url.searchParams.get('p') ?? '1';
    const res = await apiFetch(`/api/notes?p=${page}`);
    if (!res.ok) return { notes: [], total: 0, pageSize: 10 };
    return await res.json();
};
```

**Sur web**, `+page.server.ts` est utilisé (SSR, gRPC direct).
**Sur mobile** (adapter-static), `+page.server.ts` n'existe pas → `+page.ts` est utilisé.

### Étape 4 — Config de build mobile

Créer `clients/webapp/svelte.config.mobile.js` :

```javascript
import adapterStatic from '@sveltejs/adapter-static';

export default {
    kit: {
        adapter: adapterStatic({
            fallback: 'index.html',  // SPA fallback pour le routing côté client
        }),
        prerender: {
            handleMissingId: 'ignore',
            entries: [],             // pas de prerendering (auth requise)
        }
    }
};
```

Ajouter dans `clients/webapp/package.json` :

```json
{
    "scripts": {
        "build:mobile": "vite build --config svelte.config.mobile.js",
        "cap:sync": "npx cap sync --config ../../capacitor.config.ts",
        "cap:android": "npx cap open android --config ../../capacitor.config.ts",
        "cap:run:android": "npx cap run android --config ../../capacitor.config.ts"
    }
}
```

### Étape 5 — CapacitorHttp (CORS)

La WebView sur Android/iOS bloque les requêtes cross-origin par défaut. Le plugin `CapacitorHttp` remplace `fetch()` par une implémentation native qui ne subit pas ces restrictions.

Dans `capacitor.config.ts` :

```typescript
plugins: {
    CapacitorHttp: {
        enabled: true
    }
}
```

Avec `enabled: true`, tous les `fetch()` de l'app passent par la couche native sur Android/iOS — CORS contourné automatiquement, sans modifier le code.

### Étape 6 — Caddy inchangé

La bonne nouvelle : **Caddy n'a pas besoin d'être modifié**. Un seul domaine, tout passe par Caddy :

```
monapp.com {
    # OAuth
    reverse_proxy /oauth-login/*    localhost:8090
    reverse_proxy /oauth-callback/* localhost:8090

    # API REST (mobile) + SSR (web) — même serveur SvelteKit
    reverse_proxy                   localhost:3000
}
```

Le routing `/api/*` est géré par SvelteKit, pas par Caddy.

---

## Récapitulatif des fichiers à créer/modifier

### Phase 1

| Fichier | Action |
|---------|--------|
| `capacitor.config.ts` | Ajouter `server.url: 'https://monapp.com'` |
| `Caddyfile` | Remplacer `yourdomain.com` par le vrai domaine |

### Phase 2

| Fichier | Action |
|---------|--------|
| `clients/webapp/src/routes/api/notes/+server.ts` | Créer — GET/POST |
| `clients/webapp/src/routes/api/notes/[id]/+server.ts` | Créer — GET/PUT/DELETE |
| `clients/webapp/src/routes/api/files/+server.ts` | Créer — GET/POST |
| `clients/webapp/src/routes/api/files/[id]/+server.ts` | Créer — GET/DELETE |
| `clients/webapp/src/routes/api/emails/+server.ts` | Créer — GET/POST |
| `clients/webapp/src/routes/api/profile/+server.ts` | Créer — GET/POST |
| `clients/webapp/src/routes/api/subscription/+server.ts` | Créer — POST |
| `clients/webapp/src/lib/mobile/api.ts` | Créer — utilitaire apiFetch |
| `clients/webapp/src/routes/(app)/notes/+page.ts` | Créer — load client-side |
| `clients/webapp/src/routes/(app)/notes/[noteId]/+page.ts` | Créer |
| `clients/webapp/src/routes/(app)/files/+page.ts` | Créer |
| `clients/webapp/src/routes/(app)/emails/+page.ts` | Créer |
| `clients/webapp/src/routes/(app)/profile/+page.ts` | Créer |
| `clients/webapp/src/routes/(app)/subscription/+page.ts` | Créer |
| `clients/webapp/src/routes/auth/+page.svelte` | Modifier — deep link handler |
| `clients/webapp/svelte.config.mobile.js` | Créer — adapter-static |
| `clients/webapp/package.json` | Ajouter scripts build:mobile, deps Capacitor |
| `capacitor.config.ts` | Ajouter plugins CapacitorHttp + App deep link |
| `services/service-auth/src/auth_service.rs` | Modifier — redirect deep link si ?mobile=true |

---

## Commandes de build

```bash
# Build web (adapter-node, SSR)
cd clients/webapp && pnpm run build

# Build mobile (adapter-static)
cd clients/webapp && pnpm run build:mobile

# Sync vers Android
./scripts/capacitor.sh sync

# Ouvrir Android Studio
./scripts/capacitor.sh open:android

# Lancer sur émulateur
./scripts/capacitor.sh run:android
```

---

## Vérification — Phase 1

1. Déployer l'app web sur un serveur avec un domaine
2. Vérifier HTTPS : `curl -I https://monapp.com` → `200 OK`
3. Vérifier OAuth : se connecter via le navigateur → `/dashboard`
4. `npx cap sync` avec `server.url` configuré
5. Ouvrir sur Android → l'app charge `https://monapp.com`
6. Se connecter → token dans cookie → /dashboard accessible

## Vérification — Phase 2

1. Tester les endpoints REST directement :
   ```bash
   TOKEN="votre_jwt_token"
   curl -H "Authorization: Bearer $TOKEN" https://monapp.com/api/notes
   # Doit retourner du JSON avec les notes
   ```

2. Build mobile et vérifier le bundle :
   ```bash
   cd clients/webapp && pnpm run build:mobile
   ls build/     # index.html + assets/ présents
   ```

3. Tester le deep link OAuth sur émulateur Android :
   - Lancer l'app
   - Cliquer "Continue with Google"
   - Après OAuth → app reçoit `com.rusve.app://callback?token=...`
   - Vérifier que le token est stocké : Preferences.get('token')
   - `/dashboard` accessible

4. Tester offline : couper le WiFi → les pages déjà visitées doivent rester accessibles (cache Capacitor)
