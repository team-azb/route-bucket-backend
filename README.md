# route-bucket-backend

[![CI](https://github.com/team-azb/route-bucket-backend/actions/workflows/cargo.yml/badge.svg)](https://github.com/team-azb/route-bucket-backend/actions/workflows/cargo.yml)

## Requirements
* `docker`: 18.09 or higher
* `docker-compose`: 1.25.1 or higher

Both of these are for build kit support.

## Run the Project
```bash
make start
```
This will start the following
4 containers. (See `docker-compose.yml`.)

* `api`: Rust backend server.
* `db`: MySQL server.
* `osrm`: [OSRM](https://github.com/Project-OSRM/osrm-backend) 
  server for route generation.
* `swagger`: [Swagger UI](https://github.com/swagger-api/swagger-ui) for the backend api.  

The root of the app will be at `http://localhost:8080/`.

## For Developpers
### Testing
Run
```bash
make test
```
to run [cargo-test](https://doc.rust-lang.org/cargo/commands/cargo-test.html) on the docker container.

### Add seed data to the DB
```bash
make seed
```
See `api/src/bin/seed.rs`.

### Change DB schema
After modifying `db/schema.sql`, run
```bash
make migrate-dry-run
```
to check the diff, and run
```bash
make migrate
```
to migrate the database running on the `db` container.

## Documentation
### API Documentation
`swagger` container will be up with `make start`.
To see the documentation(SwaggerUI),
go to http://localhost:4000/

### Design Principles
You can see the documentation on our design principles 
[here](./docs/architecture.md) ([japanese version](./docs/architecture-ja.md) also available).
