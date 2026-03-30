openssl genpkey -algorithm RSA -out private.key -pkeyopt rsa_keygen_bits:2048
openssl rsa -pubout -in private.key -out public.key

cp private.key ../clients/webapp/src/lib/server/private.key
cp public.key ../services/service-users/public.key
cp public.key ../services/service-notes/public.key
cp public.key ../services/service-utils/public.key
