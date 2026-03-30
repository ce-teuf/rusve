rm -rf ./clients/webapp/src/lib/proto
mkdir ./clients/webapp/src/lib/proto
rm -rf ./services/service-auth/src/proto
rm -rf ./services/service-users/src/proto
rm -rf ./services/service-notes/src/proto
rm -rf ./services/service-utils/src/proto

cd ./proto

# Client
npm i
npm run proto

# Server
cargo run
