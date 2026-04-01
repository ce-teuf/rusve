rm -rf ./clients/webapp/src/lib/proto
mkdir ./clients/webapp/src/lib/proto

rm -f ./services/service-auth/src/proto.rs
rm -f ./services/service-users/src/proto.rs
rm -f ./services/service-notes/src/proto.rs
rm -f ./services/service-utils/src/proto.rs
rm -f ./services/service-scraper/src/proto.rs

cd ./proto

# Client (TypeScript bindings)
npm i
npm run proto

# Server (Rust bindings)
cargo run
