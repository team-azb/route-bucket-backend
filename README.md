# route-bucket-backend

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

### (Optional) Add seed data to the DB
```bash
make seed
```
See `src/bin/seed.rs`.

### (Optional) Change DB schema
After modifying `mysql/schema.sql`, run
```bash
make migrate-dry-run
```
to check the diff, and run
```bash
make migrate
```
to migrate the database running on the `db` container.

## Documentation
`swagger` container will be up with `make start`.
To see the documentation(SwaggerUI),
go to http://localhost:4000/
