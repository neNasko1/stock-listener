cargo build &&
APCA_API_KEY_ID=$"{cat config.json | jq '.\"APCA_API_KEY_ID\"'}" APCA_API_SECRET_KEY=$"{cat config.json | jq '.\"APCA_API_SECRET_KEY\"'}" ./target/debug/moneys
