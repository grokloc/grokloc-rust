IMG_BASE     = rust:slim-bullseye
IMG_DEV      = grokloc/grokloc-rs:dev
DOCKER       = docker
DOCKER_RUN   = $(DOCKER) run --rm -it
CWD          = $(shell pwd)
BASE         = /grokloc
PORTS        = -p 3000:3000
VOLUMES      = -v $(CWD):$(BASE)
RUN          = $(DOCKER_RUN) $(VOLUMES) -w $(BASE) $(PORTS) $(IMG_DEV)

.PHONY: update
update:
	cargo update

.PHONY: upgrade
upgrade:
	cargo upgrade

.PHONY: build
build:
	cargo build --verbose

.PHONY: test
test:
	cargo test --verbose

.PHONY: clippy
clippy:
	cargo clippy --all-targets --all-features -- -D warnings

.PHONY: fmt
fmt:
	for i in `find src -name \*.rs`; do rustfmt --edition 2021 $$i; done

.PHONY: all
all: test clippy fmt

.PHONY: docker
docker:
	$(DOCKER) pull $(IMG_BASE)
	$(DOCKER) build . -f Dockerfile -t $(IMG_DEV)
	$(DOCKER) system prune -f
	$(DOCKER) system prune -f
	$(DOCKER) system prune -f

.PHONY: docker-push
docker-push:
	$(DOCKER) push $(IMG_DEV)

.PHONY: docker-pull
docker-pull:
	$(DOCKER) pull $(IMG_BASE)
	$(DOCKER) pull $(IMG_DEV)
	$(DOCKER) system prune -f
	$(DOCKER) system prune -f
	$(DOCKER) system prune -f

.PHONY: shell
shell:
	$(RUN) /bin/bash

.PHONY: container-build
container-build:
	$(RUN) make build

.PHONY: container-test
container-test:
	$(RUN) make test
