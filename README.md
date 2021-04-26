# route-bucket-backend

## Requirements
* `docker`: 18.09 or higher
* `docker-compose`: 1.25.1 or higher

Both of these are for build kit support.

## Run the Project
```bash
./start_server.sh
```
The root of the app will be at `http://localhost:8080/`.

### (Optional) Add seed data to the DB
```bash
./insert_seed_data.sh
```
See `src/bin/seed.rs`

## Documentation
To see the documentation(SwaggerUI).
Swagger container will be up in `./start_server.sh`.
Go to http://localhost:10080/ for the documents.
