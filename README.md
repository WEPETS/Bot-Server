
## Starting the DB, Cloudflared, Sui

```sh
# Start postgresql server docker image:
docker run --rm --name pg -p 5432:5432 \
   -e POSTGRES_PASSWORD=welcome \
   postgres:15

# Share local 
cloudflared tunnel --url http://localhost:8080

# Start sui
sui start
```

## Dev (watch)

> NOTE: Install cargo watch with `cargo install cargo-watch`.

```sh
# Terminal 1 - To run the server.
cargo watch -q -c -w src/ -x "run"

# Terminal 2 - To run the quick_dev.
cargo watch -q -c -w examples/ -w .env -x "run --example quick_dev"
```

## Unit Test (watch)

```sh
cargo watch -q -c -x "test -- --nocapture"

```

## Dev

```sh
# Terminal 1 - To run the server.
cargo run

# Terminal 2 - To run the tests.
cargo run --example quick_dev
```

## Unit Test

```sh
cargo test -- --nocapture

```

<br />

---