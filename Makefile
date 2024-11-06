# Define variables
DOCKER_IMAGE = jocas/breitbandmessung
DOCKER_COMPOSE = docker-compose.yml
CHECK_INTERVAL ?= 60

# Default target
.PHONY: all
all: build

# Build the Docker image
.PHONY: build
build:
	docker build -t $(DOCKER_IMAGE) .

# Run the Docker Compose setup
.PHONY: up
up:
	@CHECK_INTERVAL=$(CHECK_INTERVAL) docker-compose -f $(DOCKER_COMPOSE) up -d

# Stop the Docker Compose setup
.PHONY: down
down:
	docker-compose -f $(DOCKER_COMPOSE) down

# Show logs from the running service
.PHONY: logs
logs:
	docker-compose -f $(DOCKER_COMPOSE) logs -f

# Run tests
.PHONY: test
test:
	cargo test -- --nocapture

# Clean up Docker images and containers
.PHONY: clean
clean:
	docker-compose -f $(DOCKER_COMPOSE) down -v
	docker rmi $(DOCKER_IMAGE) || true
	cargo clean

# Lint the Rust code
.PHONY: lint
lint:
	cargo fmt -- --check
	cargo clippy -- -D warnings

# Shell into the running container
.PHONY: shell
shell:
	docker-compose -f $(DOCKER_COMPOSE) exec speed_test /bin/sh

# Rebuild and run
.PHONY: rebuild
rebuild: clean build up

