# Phase 2 Capacitor — Instructions d'exécution

## Prérequis

### Outils requis
```bash
# Android Studio (avec Android SDK, émulateur)
# Télécharger : https://developer.android.com/studio

# Java 17+
java -version

# Node.js (via nvm) + pnpm
node -v   # v18+ requis
pnpm -v

# Rust + cargo (pour les services)
cargo -V
```

### Variables d'environnement
```bash
# À la racine du projet :
cp .env.example .env
# Remplir : GOOGLE_CLIENT_ID, GOOGLE_CLIENT_SECRET, GITHUB_CLIENT_ID, GITHUB_CLIENT_SECRET, JWT_SECRET
```

---

## 1. Démarrer le backend

```bash
# Depuis la racine rusve/
make db       # démarre PostgreSQL (Docker)
make dev      # démarre tous les services Rust + webapp SSR
```

Le backend tourne sur :
- `http://localhost:8090` — service-auth (OAuth)
- `localhost:50051/52/53` — service-users/notes/utils (gRPC)
- `http://localhost:8080` — webapp SvelteKit (SSR web)

---

## 2. Installer les dépendances

```bash
cd clients/webapp
pnpm install
cd ../..
```

Vérifier que le CLI Capacitor est bien installé :
```bash
ls clients/webapp/node_modules/.bin/cap   # doit exister
```

---

## 3. Ajouter la plateforme Android (première fois seulement)

```bash
# Depuis la racine rusve/
./scripts/capacitor.sh add:android
```

Ce que fait cette commande :
1. Lance `pnpm run build` (build SSR web — nécessaire pour que cap initialise correctement)
2. Exécute `cap add android` depuis `clients/webapp/`
3. Génère le projet natif Android dans `clients/webapp/android/`

---

## 4. Builder le bundle mobile (SPA statique)

```bash
# Depuis clients/webapp/
pnpm run build:mobile
```

Ce que fait `build:mobile` : `BUILD_TARGET=mobile vite build`
- Active `adapter-static` (au lieu de `adapter-node`) via `svelte.config.ts`
- Génère `clients/webapp/build/` : `index.html` + assets JS/CSS statiques
- Ce dossier est ce qui sera embarqué dans l'APK

Résultat attendu :
```
> Using @sveltejs/adapter-static
  Wrote site to "build"
  ✔ done
```

---

## 5. Synchroniser le bundle dans Android

```bash
# Depuis la racine rusve/
./scripts/capacitor.sh sync
```

Ce que fait `sync` :
1. Lance `pnpm run build:mobile` (build statique)
2. Exécute `cap sync` depuis `clients/webapp/`
3. Copie `build/` dans `clients/webapp/android/app/src/main/assets/public/`
4. Met à jour les plugins natifs (CapacitorHttp, App, Preferences)

---

## 6. Ouvrir Android Studio

```bash
./scripts/capacitor.sh open:android
```

Dans Android Studio :
1. Attendre la fin de l'indexation Gradle (~2 min première fois)
2. **Build > Make Project** pour vérifier
3. Démarrer l'émulateur ou connecter un appareil USB (USB Debugging activé)
4. Cliquer **Run ▶**

---

## 7. Tester l'authentification OAuth mobile

Le flow OAuth sur mobile utilise les **deep links** (`com.rusve.app://callback`).

### Configuration requise

`AUTH_URL` (dans `.env` ou `Makefile`) doit être accessible depuis l'émulateur Android.
L'hôte machine depuis un émulateur Android = `10.0.2.2` :

```bash
# .env
AUTH_URL=http://10.0.2.2:8090
CLIENT_URL=http://10.0.2.2:8080
```

Et dans `clients/webapp/.env` :
```bash
PUBLIC_AUTH_URL=http://10.0.2.2:8090
```

### Flow complet

```
1. App Android ouvre /auth
2. Tap "Continue with Google"
3. → fetch(PUBLIC_AUTH_URL) pour vérifier que le serveur répond
4. → window.location.href = http://10.0.2.2:8090/oauth-login/google?mobile=true
5. → service-auth stocke state "mobile:{csrf_token}" en base
6. → redirect vers Google OAuth
7. → Google redirect vers /oauth-callback/google?code=...&state=mobile:{csrf}
8. → service-auth détecte préfixe "mobile:", crée/met à jour l'user, récupère le JWT
9. → redirect vers com.rusve.app://callback?token={JWT}
10. → Android intercepte le deep link → appUrlOpen event déclenché
11. → auth/+page.svelte extrait le token, le stocke via @capacitor/preferences
12. → goto('/dashboard')
```

### Tester le deep link manuellement (sans OAuth)

```bash
# Générer un JWT valide depuis le login web, puis :
adb shell am start -W -a android.intent.action.VIEW \
  -d "com.rusve.app://callback?token=TON_JWT" \
  com.rusve.app
```

---

## 8. Tester les endpoints REST API

Backend démarré (`make dev`), tester depuis le terminal avec un token JWT valide
(récupéré depuis le cookie `token` après un login web dans le navigateur) :

```bash
TOKEN="colle_ton_jwt_ici"

# Notes — liste paginée
curl -H "Authorization: Bearer $TOKEN" http://localhost:8080/api/notes
curl -H "Authorization: Bearer $TOKEN" "http://localhost:8080/api/notes?p=2"

# Notes — créer
curl -X POST -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"title":"Test mobile","content":"Hello depuis curl"}' \
  http://localhost:8080/api/notes

# Notes — modifier
curl -X PUT -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"title":"Modifié","content":"Contenu mis à jour"}' \
  http://localhost:8080/api/notes/NOTE_ID

# Notes — supprimer
curl -X DELETE -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/notes/NOTE_ID

# Files — liste
curl -H "Authorization: Bearer $TOKEN" http://localhost:8080/api/files

# Files — upload
curl -X POST -H "Authorization: Bearer $TOKEN" \
  -F "file=@/chemin/vers/fichier.pdf" \
  http://localhost:8080/api/files

# Emails — liste
curl -H "Authorization: Bearer $TOKEN" http://localhost:8080/api/emails

# Profile
curl -H "Authorization: Bearer $TOKEN" http://localhost:8080/api/profile

# Subscription — URL Stripe checkout
curl -X POST -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"action":"checkout"}' \
  http://localhost:8080/api/subscription

# Subscription — URL Stripe portail
curl -X POST -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"action":"portal"}' \
  http://localhost:8080/api/subscription
```

---

## 9. Workflow de développement quotidien

```bash
# Terminal 1 : backend (services Rust + webapp SSR)
make dev

# Terminal 2 : après chaque modification du code SvelteKit
cd clients/webapp
pnpm run build:mobile        # rebuild le bundle statique
cd ../..
./scripts/capacitor.sh sync  # sync dans Android (= build:mobile + cap sync)

# Android Studio : Run ▶ pour relancer l'app
```

### Live Reload (optionnel — charge l'app depuis le serveur local)

Dans `clients/webapp/capacitor.config.cjs`, décommenter :
```js
server: {
  url: 'http://10.0.2.2:8080',  // pointe vers la webapp SSR locale
  cleartext: true,
}
```
Puis `./scripts/capacitor.sh sync`. L'app chargera la webapp en direct sans rebuild.
Pratique pour le dev UI, mais la webapp doit être accessible sur le réseau.

---

## 10. Build release APK

```bash
# 1. Builder le bundle statique
cd clients/webapp && pnpm run build:mobile && cd ../..

# 2. Synchroniser dans Android
./scripts/capacitor.sh sync

# 3. Ouvrir Android Studio pour signer et exporter l'APK
./scripts/capacitor.sh open:android
# Dans Android Studio : Build > Generate Signed Bundle / APK
```

APK de debug (sans signature) :
```
clients/webapp/android/app/build/outputs/apk/debug/app-debug.apk
```

APK release signé :
```
clients/webapp/android/app/build/outputs/apk/release/app-release.apk
```

---

## Résumé des fichiers créés / modifiés

| Fichier | Rôle |
|---|---|
| `clients/webapp/src/routes/api/notes/+server.ts` | REST GET /api/notes, POST |
| `clients/webapp/src/routes/api/notes/[id]/+server.ts` | REST GET/PUT/DELETE par id |
| `clients/webapp/src/routes/api/files/+server.ts` | REST GET /api/files, POST upload |
| `clients/webapp/src/routes/api/files/[id]/+server.ts` | REST GET download, DELETE |
| `clients/webapp/src/routes/api/emails/+server.ts` | REST GET /api/emails, POST send |
| `clients/webapp/src/routes/api/profile/+server.ts` | REST GET/POST /api/profile |
| `clients/webapp/src/routes/api/subscription/+server.ts` | REST POST → URL Stripe |
| `clients/webapp/src/lib/server/api-auth.ts` | Validation Bearer token → user |
| `clients/webapp/src/lib/mobile/auth.ts` | get/set/clearToken (Preferences natif) |
| `clients/webapp/src/lib/mobile/api.ts` | apiFetch\<T\> avec Bearer token auto |
| `clients/webapp/src/routes/(app)/*/+page.ts` | Universal loads (SSR web ou API mobile) |
| `clients/webapp/src/routes/(app)/+layout.ts` | Universal layout load |
| `clients/webapp/src/routes/auth/+page.svelte` | Deep link listener + ?mobile=true |
| `clients/webapp/src/hooks.server.ts` | Skip auth pendant `building` (build statique) |
| `clients/webapp/svelte.config.ts` | adapter-static si BUILD_TARGET=mobile |
| `clients/webapp/package.json` | script build:mobile + deps Capacitor |
| `clients/webapp/capacitor.config.cjs` | Config Capacitor (webDir, plugins) |
| `scripts/capacitor.sh` | Utilise bin local + cap run depuis clients/webapp |
| `services/service-auth/src/auth_service.rs` | Deep link redirect si ?mobile=true |
