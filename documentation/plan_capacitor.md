Voici mon plan détaillé pour réorganiser le projet avec Capacitor:
Structure proposée
rusve/
├── clients/
│   ├── webapp/           # Projet SvelteKit actuel (PWA)
│   │   ├── src/
│   │   ├── static/
│   │   ├── package.json
│   │   ├── vite.config.ts
│   │   └── ...
│   ├── android/         # Projet Capacitor Android
│   │   ├── android/
│   │   ├── capacitor.config.ts
│   │   └── package.json
│   └── ios/             # Projet Capacitor iOS
│       ├── ios/
│       ├── capacitor.config.ts
│       └── package.json
├── services/            # (services Rust actuels)
├── docker-compose*.yml
└── ...
Étapes détaillées
1. Créer la nouvelle arborescence
# Renommer client -> clients/webapp
mv client clients/webapp
# Créer les dossiers android et ios
mkdir clients/android clients/ios
2. Configurer Capacitor dans webapp
Ajouter @capacitor/core et @capacitor/cli dans clients/webapp/package.json:
cd clients/webapp
npm install @capacitor/core @capacitor/cli @capacitor/ios @capacitor/android
npx cap init "Rusve" --web-dir build --app-id com.rusve.app
Modifier vite.config.ts pour que le build sorte dans build (actuellement c'est peut-être .svelte-kit/output ou build - vérifier).
3. Configurer les natifs (android/ios)
Pour Android:
cd clients/android
npx create . --directory . --template app --project-name rusve
# OU simplement copier depuis webapp et faire:
npx cap add android
Pour iOS:
cd clients/ios
npx cap add ios
Alternative recommandée: Les projets android/ios peuvent être quasi vides - ils contiennent juste:
- capacitor.config.ts pointant vers le dossier webapp
- Le dossier android/ ou ios/ généré par Capacitor
- Un package.json minimal
4. Fichiers de config Capacitor
clients/webapp/capacitor.config.ts:
import { CapacitorConfig } from '@capacitor/cli';
const config: CapacitorConfig = {
  appId: 'com.rusve.app',
  appName: 'Rusve',
  webDir: 'build',
  server: {
    androidScheme: 'https'
  }
};
export default config;
5. Manifest PWA existant
Ton static/manifest.json actuel est correct pour la PWA. Pour les apps natives, Capacitor génère automatiquement les icônes et configs natives.
6. Service Worker (optionnel mais recommandé)
Installer Workbox pour le offline:
npm install workbox-precaching workbox-routing
Créer src/service-worker.ts dans webapp.
7. Scripts package.json
Ajouter dans clients/webapp/package.json:
{
  "scripts": {
    "cap:sync": "npx cap sync",
    "cap:android": "npx cap add android && npx cap open android",
    "cap:ios": "npx cap add ios && npx cap open ios"
  }
}
8. Docker-compose à mettre à jour
Modifier docker-compose.app.yml pour pointer vers clients/webapp au lieu de client.
Points importants
1. Code partagé: Le code SvelteKit reste dans webapp. Android/iOS utilisent juste le build via Capacitor.
2. Serveur de dev: Pour tester les apps natives avec le backend local, utiliser npx cap run android qui forward le localhost.
3. API URLs: Les apps natives doivent pointer vers ton serveur. En dev: ton IP locale avec le port. En prod: le domaine.
4. Fichiers à partager (via workspace ou monorepo):
   - Le code SvelteKit reste en un seul endroit
   - android/ et ios/ sont regenerés via cap sync
Tu veux que je passe à l'exécution ou tu as des questions sur des points spécifiques?
