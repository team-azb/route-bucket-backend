db_manager = db/manager/empty_make_target

.PHONY = start start-without-api seed migrate migrate-dry-run

start:
	DOCKER_BUILDKIT=1 \
	COMPOSE_DOCKER_CLI_BUILD=1 \
	BUILDKIT_PROGRESS=plain \
	docker-compose up --build

start-without-api:
	DOCKER_BUILDKIT=1 \
	COMPOSE_DOCKER_CLI_BUILD=1 \
	BUILDKIT_PROGRESS=plain \
	docker-compose up --build db osrm swagger

seed:
	docker-compose run api /app/target/release/seed

test:
	docker-compose run api cargo test --all

migrate: $(db_manager)
	docker-compose run db_manager \
		mysqldef --host=db --password=password \
		--file=mysql/schema.sql route_bucket_db

migrate-dry-run: $(db_manager)
	docker-compose run db_manager \
		mysqldef --host=db --password=password \
		--file=mysql/schema.sql --dry-run route_bucket_db

$(db_manager): db/schema.sql
	docker-compose build db_manager
	touch $@
