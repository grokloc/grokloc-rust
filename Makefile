IMG_DEV      = grokloc/grokloc-rs:dev
DOCKER       = docker
DOCKER_RUN   = $(DOCKER) run --rm -it
CWD          = $(shell pwd)
BASE         = /grokloc
PORTS        = -p 3000:3000
VOLUMES      = -v $(CWD):$(BASE)
RUN          = $(DOCKER_RUN) $(VOLUMES) -w $(BASE) $(PORTS) $(IMG_DEV)

.PHONY: build
build:
	cargo build --verbose

.PHONY: test
test:
	cargo test --verbose

.PHONY: clippy
clippy:
	cargo clippy --all-targets --all-features -- -D warnings

.PHONY: all
all: test clippy

.PHONY: docker
docker:
	$(DOCKER) build . -f Dockerfile -t $(IMG_DEV)

.PHONY: docker-push
docker-push:
	$(DOCKER) push $(IMG_DEV)

.PHONY: docker-pull
docker-pull:
	$(DOCKER) pull $(IMG_DEV)

.PHONY: shell
shell:
	$(RUN) /bin/bash

.PHONY: container-build
container-build:
	$(RUN) make build

.PHONY: container-test
container-test:
	$(RUN) make test
