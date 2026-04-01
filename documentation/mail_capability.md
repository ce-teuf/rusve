Implémentation terminée
Ce qui a été fait :
1. .env : Ajouté vars SMTP (placeholder) + garder SENDGRID_API_KEY
      SMTP_HOST=smtp.protonmail.ch
   SMTP_PORT=587
   SMTP_USERNAME=xxx
   SMTP_PASSWORD=xxx
   SMTP_FROM_EMAIL=xxx
   SMTP_FROM_NAME=rusve-test-app
   
2. Makefile : Ajouté SMTP_* à ENV_UTILS
3. docker-compose.app.yml : Ajouté SMTP_* à service-utils
4. Cargo.toml : Ajouté lettre (avec feature tokio1-native-tls)
5. lib.rs : Ajouté champs SMTP dans Env (optionnels avec default)
6. email_service.rs : 
   - Implémenté envoi via SMTP (lettre)
   - Fallback automatique vers SendGrid si SMTP pas configuré
   - Log qui indique quel provider est utilisé
Configuration pour activer Proton :
Dans ton .env, remplacer les placeholders :
SMTP_HOST=smtp.protonmail.ch
SMTP_PORT=587
SMTP_USERNAME=validation@rusvetest.quest
SMTP_PASSWORD=ton_token_proton
SMTP_FROM_EMAIL=validation@rusvetest.quest
SMTP_FROM_NAME=rusve-test-app
Le service détectera automatiquement que SMTP est configuré et l'utilisera à la place de SendGrid.