CARGO ?= cargo
SERVER_BIN ?= server
SERVER_PORT ?= 3000

.PHONY: run fmt check test docker-build

run:
	SERVER_PORT=$(SERVER_PORT) $(CARGO) run -p $(SERVER_BIN)

fmt:
	$(CARGO) fmt --all

check:
	$(CARGO) check -p $(SERVER_BIN)

test:
	$(CARGO) test -p $(SERVER_BIN)

docker-build:
	docker build -t $(SERVER_BIN):latest -f server/Dockerfile .
