DOCKER_BUILDKIT=1 \
COMPOSE_DOCKER_CLI_BUILD=1 \
BUILDKIT_PROGRESS=plain \
docker-compose run api /app/bin/release/seed