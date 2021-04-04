# route-bucket-backend

## supported environment
`rustc` & `cargo` must be `>=1.51.0` since this project uses [const generics](https://blog.rust-lang.org/2021/03/25/Rust-1.51.0.html)

## Run the Project
### Start MySQL
```bash
docker-compose up db
```

### DB Setup

1. install [diesel_cli](https://crates.io/crates/diesel_cli)
   ```bash
   cargo install diesel_cli
   ```
1. run ↓ at the root of this project
   ```bash
   source .env
   diesel setup
   ```
1. (optional) to add seed data to the db, run ↓
   ```bash
   cargo run --bin seed
   ```

### Start the Backend Server
```bash
cargo run
```
The root of the app will be at http://localhost:8080/
