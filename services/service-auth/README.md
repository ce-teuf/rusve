service-auth n'utilise pas public.key car il génère les JWT (avec private.key côté client) — il ne les vérifie pas. Les services qui vérifient les tokens
  (service-users, service-notes, service-utils) ont besoin de public.key.

  La chaîne actuelle :                                                                                                                                        
  - client (SvelteKit) → signe les JWT avec private.key
  - service-users/notes/utils → vérifient avec public.key                                                                                                     
  - service-auth → fait le flow OAuth et appelle service-users, mais ne vérifie pas de JWT lui-même                                                         
                                                                                                                                                              
  Donc non, service-auth n'en a pas besoin. Le scripts/keys.sh est correct tel quel.          