# Implémentation Capacitor - Rusve

## Fichiers créés

### 1. `capacitor.config.ts` (à la racine)

Configuration centralisée de Capacitor avec chemins personnalisés:

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

### 2. `scripts/capacitor.sh`

Script principal pour toutes les opérations Capacitor.

**Usage:**
```bash
./scripts/capacitor.sh <command>
```

**Commandes disponibles:**

| Commande | Description |
|----------|-------------|
| `./scripts/capacitor.sh install` | Installe les dépendances npm Capacitor |
| `./scripts/capacitor.sh init` | Crée `capacitor.config.ts` |
| `./scripts/capacitor.sh add:android` | Ajoute la plateforme Android |
| `./scripts/capacitor.sh add:ios` | Ajoute la plateforme iOS |
| `./scripts/capacitor.sh sync` | Build web + sync vers natif |
| `./scripts/capacitor.sh run:android` | Lance sur Android (émulateur/appareil) |
| `./scripts/capacitor.sh run:ios` | Lance sur iOS (macOS uniquement) |
| `./scripts/capacitor.sh open:android` | Ouvre Android Studio |
| `./scripts/capacitor.sh open:ios` | Ouvre Xcode (macOS uniquement) |
| `./scripts/capacitor.sh setup` | Setup complet (install + init + add:android) |

### 3. `scripts/capacitor-setup.sh`

Script utilitaire pour ajouter les dépendances et scripts au package.json (utilise jq).

---

## Structure générée après setup

```
rusve/
├── clients/
│   ├── webapp/
│   │   ├── src/
│   │   ├── static/
│   │   ├── build/           # Output du build SvelteKit
│   │   ├── android/         # Projet natif Android
│   │   │   └── android/
│   │   └── ios/             # Projet natif iOS
│   │       └── ios/
│   ├── android/             # Répertoire vide (redondant, peut supprimer)
│   └── ios/                 # Répertoire vide (redondant, peut supprimer)
├── capacitor.config.ts      # Config Capacitor
├── scripts/
│   ├── capacitor.sh         # Script principal
│   └── capacitor-setup.sh   # Script d'installation
└── ...
```

---

## Utilisation

### Setup initial

```bash
# Option 1: Setup complet
./scripts/capacitor.sh setup

# Option 2: Manuel étape par étape
./scripts/capacitor.sh install
./scripts/capacitor.sh init
./scripts/capacitor.sh add:android
```

### Développement Android

```bash
# Terminal 1: Backend + Webapp
make dev

# Terminal 2: Lancer l'app Android
./scripts/capacitor.sh sync        # Build web + sync
./scripts/capacitor.sh run:android  # Lance sur émulateur/appareil
```

### Développement iOS (macOS uniquement)

```bash
./scripts/capacitor.sh sync
./scripts/capacitor.sh run:ios
```

### iOS sans Mac (Capgo)

```bash
# Build web
cd clients/webapp && npm run build

# Sync vers iOS
npx cap sync ios --config ../../capacitor.config.ts

# Cloud build via Capgo
capgo build com.rusve.app --platform ios --build-mode release
```

---

## Commandes Capacitor directes

```bash
# Sync (build + copy)
npx cap sync --config capacitor.config.ts

# Copy uniquement (plus rapide)
npx cap copy --config capacitor.config.ts

# Build release
npx cap build android --config capacitor.config.ts

# Ouvrir IDE
npx cap open android --config capacitor.config.ts
npx cap open ios --config capacitor.config.ts
```

---

## Notes

- Les scripts utilisent `--config ../../capacitor.config.ts` car ils sont exécutés depuis `clients/webapp/`
- Le build web doit être dans `clients/webapp/build/` (output SvelteKit avec adapter-node)
- Les dossiers `clients/android/` et `clients/ios/` vides peuvent être supprimés - Capacitor génère les vrais projets dans `clients/webapp/android/` et `clients/webapp/ios/`
