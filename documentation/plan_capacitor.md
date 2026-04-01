# Plan d'intégration Capacitor - Rusve

## État actuel

```
rusve/
├── clients/
│   ├── webapp/           # Application SvelteKit
│   ├── android/          # Répertoire vide
│   └── ios/              # Répertoire vide
├── services/
│   ├── service-auth/
│   ├── service-users/
│   ├── service-notes/
│   └── service-utils/
├── Makefile              # À CORRIGER
└── ...
```

---

## Architecture recommandée

```
rusve/
├── clients/
│   ├── webapp/           # Application SvelteKit (PWA)
│   │   ├── src/
│   │   ├── static/
│   │   ├── build/        # Output du build
│   │   └── package.json
│   ├── android/         # Projet natif Android
│   │   └── android/     # Généré par Capacitor
│   └── ios/              # Projet natif iOS
│       └── ios/          # Généré par Capacitor
├── services/
│   ├── service-auth/
│   ├── service-users/
│   ├── service-notes/
│   └── service-utils/
├── capacitor.config.ts   # Config Capacitor à la racine
├── Makefile              # À CORRIGER
└── docker-compose*.yml
```

---

## Étape 1: Corriger le Makefile

Le Makefile doit pointer vers les nouveaux chemins.

### Ligne 77-82 - cible `client-env`:
```makefile
# CORRIGÉ:
	@printf 'USERS_URI=localhost:%s\nNOTES_URI=localhost:%s\nUTILS_URI=localhost:%s\nGRPC_SSL=false\nENV=development\nCOOKIE_DOMAIN=localhost\nPUBLIC_AUTH_URL=http://localhost:8080\nJWT_SECRET=%s\nUPSEND_KEY=%s\n' \
		$(USERS_PORT) $(NOTES_PORT) $(UTILS_PORT) "$(JWT_SECRET)" "$(UPSEND_KEY)" \
		> clients/webapp/.env
```

### Ligne 85-86 - cible `client`:
```makefile
# CORRIGÉ:
	cd clients/webapp && pnpm run dev
```

### Lignes 89-96 - cible `dev`:
```makefile
# CORRIGÉ:
(cd services/service-auth  && $(ENV_AUTH)  cargo run) & \
(cd services/service-users && $(ENV_USERS) cargo run) & \
(cd services/service-notes && $(ENV_NOTES) cargo run) & \
(cd services/service-utils && $(ENV_UTILS) cargo run) & \
(cd clients/webapp        && pnpm run dev) & \
```

### Lignes 99-106 - cible `watch`: mêmes corrections

### Lignes 108-121 - cible `release`:
Corriger tous les chemins `service-*/` → `services/service-*/`

---

## Étape 2: Installer Capacitor

```bash
cd clients/webapp
npm install @capacitor/core @capacitor/cli @capacitor/android @capacitor/ios
```

---

## Étape 3: Configurer Capacitor à la racine du projet

Créer `capacitor.config.ts` à la racine `rusve/`:

```typescript
import { CapacitorConfig } from '@capacitor/cli';

const config: CapacitorConfig = {
  appId: 'com.rusve.app',
  appName: 'Rusve',
  webDir: 'clients/webapp/build',
  
  android: {
    path: 'clients/android'
  },
  
  ios: {
    path: 'clients/ios'
  },
  
  server: {
    androidScheme: 'https'
  }
};

export default config;
```

---

## Étape 4: Initialiser Capacitor

```bash
# Initialiser la config
npx cap init --config capacitor.config.ts

# OU avec les paramètres
npx cap init "Rusve" --web-dir clients/webapp/build --app-id com.rusve.app --config capacitor.config.ts
```

---

## Étape 5: Ajouter les plateformes natives

```bash
# Android
npx cap add android

# iOS (macOS uniquement)
npx cap add ios
```

Ces commandes créent:
- `clients/android/android/` - projet Android natif
- `clients/ios/ios/` - projet iOS natif

---

## Étape 6: Scripts npm dans package.json

Ajouter dans `clients/webapp/package.json`:

```json
{
  "scripts": {
    "cap:sync": "npx cap sync --config ../../capacitor.config.ts",
    "cap:android": "npx cap open android --config ../../capacitor.config.ts",
    "cap:ios": "npx cap open ios --config ../../capacitor.config.ts",
    "cap:run:android": "npx cap run android --config ../../capacitor.config.ts",
    "cap:run:ios": "npx cap run ios --config ../../capacitor.config.ts"
  }
}
```

---

## Étape 7: PWA Service Worker

Le manifest.json existe déjà dans `clients/webapp/static/`.

Pour le offline, installer Workbox:

```bash
npm install workbox-precaching workbox-routing
```

Créer `clients/webapp/src/service-worker.ts`:

```typescript
/// <reference lib="webworker" />
import { build, files, version } from '$service-worker';

declare const self: ServiceWorkerGlobalScope;

const CACHE = `cache-${version}`;
const ASSETS = [...build, ...files];

self.addEventListener('install', (event) => {
  async function addFilesToCache() {
    const cache = await caches.open(CACHE);
    await cache.addAll(ASSETS);
  }
  event.waitUntil(addFilesToCache());
});

self.addEventListener('activate', (event) => {
  async function deleteOldCaches() {
    for (const key of await caches.keys()) {
      if (key !== CACHE) await caches.delete(key);
    }
  }
  event.waitUntil(deleteOldCaches());
});

self.addEventListener('fetch', (event) => {
  if (event.request.method !== 'GET') return;

  async function respond() {
    const url = new URL(event.request.url);
    const cache = await caches.open(CACHE);

    if (ASSETS.includes(url.pathname)) {
      const response = await cache.match(url.pathname);
      if (response) return response;
    }

    try {
      const response = await fetch(event.request);
      if (response.status === 200) {
        cache.put(event.request, response.clone());
      }
      return response;
    } catch {
      return cache.match(event.request);
    }
  }

  event.respondWith(respond());
});
```

---

## Étape 8: Configuration de l'API par plateforme

Créer `clients/webapp/src/lib/api.ts`:

```typescript
import { Capacitor } from '@capacitor/core';
import { isPlatform } from '@capacitor/core';

export const getApiUrl = () => {
  if (Capacitor.isNativePlatform()) {
    // Production: URL du serveur
    if (import.meta.env.PROD) {
      return 'https://api.rusve.com';
    }
    // Développement: IP locale (à ajuster)
    return 'http://192.168.1.X:8090';
  }
  // Web: localhost
  return 'http://localhost:8090';
};

export const isNative = () => Capacitor.isNativePlatform();
export const isAndroid = () => isPlatform('android');
export const isIOS = () => isPlatform('ios');
```

---

## Développement

### Mode web standard
```bash
make dev
# ou
cd clients/webapp && pnpm run dev
```

### Android avec live reload
```bash
# Terminal 1: backend + webapp
make dev

# Terminal 2: lancer l'app Android
cd clients/webapp
npx cap run android --config ../../capacitor.config.ts
```

### iOS avec live reload (macOS uniquement)
```bash
cd clients/webapp
npx cap run ios --config ../../capacitor.config.ts
```

---

## Commandes Capacitor utiles

```bash
# Sync le web vers les natifs
npx cap sync --config capacitor.config.ts

# Copier sans reinstall deps natifs (plus rapide)
npx cap copy --config capacitor.config.ts

# Ouvrir Android Studio
npx cap open android --config capacitor.config.ts

# Ouvrir Xcode
npx cap open ios --config capacitor.config.ts

# Build release Android
npx cap build android --config capacitor.config.ts
```

---

## Production

```bash
# 1. Build web
cd clients/webapp && pnpm run build

# 2. Sync vers natifs
npx cap sync --config capacitor.config.ts

# 3. Ouvrir IDE pour signer
npx cap open android --config capacitor.config.ts
# ou
npx cap open ios --config capacitor.config.ts
```

---

## Prérequis système

### Android (Linux)
```bash
# JDK 17+
sudo apt install openjdk-17-jdk

# Android SDK
mkdir -p ~/android-sdk/cmdline-tools
cd ~/android-sdk/cmdline-tools
wget https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip
unzip commandlinetools-linux-11076708_latest.zip
mv cmdline-tools latest

# ~/.bashrc ou ~/.zshrc
export ANDROID_HOME=~/android-sdk
export PATH=$PATH:$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools

# SDK components
sdkmanager "platform-tools" "platforms;android-34" "build-tools;34.0.0"
```

### iOS sur Debian/Linux - Options disponibles

**Problème**: Apple requiert Xcode pour compiler iOS, et Xcode n'existe que sur macOS.

**Solutions**:

#### Option A: Capgo (Recommandée pour Capacitor)

[Capgo](https://capgo.app/) permet de compiler iOS dans le cloud sans Mac.

**Fonctionnement**:
1. Générer le projet iOS localement (`cap add ios`)
2. Build web localement (`npm run build`)
3. Sync vers iOS (`cap sync ios`)
4. Trigger cloud build via CLI Capgo

**Installation**:
```bash
npm install -g @capgo/cli
capgo login
```

**Commandes**:
```bash
# Configurer les credentials Apple
capgo build credentials save \
  --platform ios \
  --certificate ./cert.p12 \
  --p12-password "password" \
  --provisioning-profile ./profile.mobileprovision \
  --apple-key ./AuthKey.p8 \
  --apple-key-id "KEY123" \
  --apple-issuer-id "issuer-uuid" \
  --apple-team-id "team-id"

# Trigger build
capgo build com.rusve.app --platform ios --build-mode release
```

**Prérequis**:
- Compte Apple Developer (99$/an)
- Certificate de distribution (.p12)
- Provisioning profile (.mobileprovision)
- Clé API App Store Connect (.p8)

**Tarif**:
- Gratuit pour les builds manuels
- Paiant pour CI/CD automatique

#### Option B: Ionic Appflow

Service officiel Ionic pour cloud builds. Similar à Capgo.

#### Option C: Location Mac

Utiliser un Mac temporairement pour:
- Générer les credentials initiaux
- Build initial
- Soumissions App Store/TestFlight

#### Option D: Virtualisation (non recommandé)

- macOS sur Linux via esxi/charmm - **ILLÉGAL** (violation licence Apple)
- Pas de solution viable légalement

---

## Notes importantes

1. **webDir**: Avec SvelteKit + adapter-node, le build sort dans `build/`.

2. **Live reload**: Le serveur de dev doit tourner pour `cap run`.

3. **HTTPS**: Les appareils mobiles nécessitent HTTPS sauf localhost. Pour tester sur appareil réel, utiliser ngrok.

4. **Config centralisée**: Un seul `capacitor.config.ts` à la racine permet de gérer tout depuis un seul endroit.
