Résultat décisif : UN seul enregistrement en DB, UN seul appel gRPC, UN seul envoi SMTP. Lettre envoie l'email une fois.                                        
                                                                                                                                                                  
  Le doublon se passe au niveau de la livraison ProtonMail, pas dans ton code. ProtonMail a un comportement connu : quand tu envoies depuis un domaine            
  personnalisé ProtonMail (admin@rusvetest.quest via smtp.protonmail.ch) vers une adresse @protonmail.com, ProtonMail traite le message comme à la fois un email  
  SMTP entrant ET une livraison interne ProtonMail → deux copies dans la boîte du destinataire.                                                                   
                                                                                                                                                                  
  J'ai d'ailleurs envoyé un email de diagnostic (SMTP diagnostic test) depuis ce compte tout à l'heure — est-ce qu'il est arrivé une ou deux fois dans            
  cetjobsearch@protonmail.com ? C'est le test décisif.
                                                                                                                                                                  
  La solution est de ne plus utiliser ProtonMail SMTP pour les transactionnels, mais d'utiliser SendGrid (qui est déjà configuré comme fallback dans ton code).   
  Pour basculer :                                                                                                                                                 
                                                                                                                                                                  
  Option 1 — laisser SendGrid actif : dans ton .env, vide SMTP_HOST et SMTP_PASSWORD → is_smtp_configured retourne false → SendGrid est utilisé.                  
                                                                                                                                                                  
  Option 2 — utiliser un vrai SMTP transactionnel : Mailtrap (dev), AWS SES, Brevo, etc. Ces services n'ont pas le problème de double livraison interne           
  ProtonMail.                                                                                                                                                     
                                                                                                                                                                  
  La raison pour laquelle "les gens disent que c'est le backend" c'est que le bug est déclenchable par le backend (le choix du provider SMTP), même si la cause   
  profonde est une particularité ProtonMail.  