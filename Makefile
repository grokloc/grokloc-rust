IMG_DEV      = grokloc/grokloc-rs:dev
DOCKER       = docker
DOCKER_RUN   = $(DOCKER) run --rm -it
CWD          = $(shell pwd)
BASE         = /grokloc
PORTS        = -p 3000:3000
VOLUMES      = -v $(CWD):$(BASE)
RUN          = $(DOCKER_RUN) $(VOLUMES) -w $(BASE) $(PORTS) $(IMG_DEV)

# Default rule - local test
.PHONY: local-test
local-test:
	cargo test

.PHONY: clippy
clippy:
	cargo clippy --all-targets --all-features -- -D warnings

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

.PHONY: test
test:
	$(RUN) make local-test
