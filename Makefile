empty_target_dir = ./docker/container_empty_targets

.PHONY = start seed migrate migrate-dry-run

start:
	DOCKER_BUILDKIT=1 \
	COMPOSE_DOCKER_CLI_BUILD=1 \
	BUILDKIT_PROGRESS=plain \
	docker-compose up --build

seed:
	docker-compose run api /app/target/release/seed

migrate: $(empty_target_dir)/db_manager
	docker-compose run db_manager \
		mysqldef --host=db --password=password \
		--file=mysql/schema.sql route_bucket_db

migrate-dry-run: $(empty_target_dir)/db_manager
	docker-compose run db_manager \
		mysqldef --host=db --password=password \
		--file=mysql/schema.sql --dry-run route_bucket_db

$(empty_target_dir)/db_manager: ./mysql/schema.sql
	docker-compose build db_manager
	touch $@
