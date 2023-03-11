export RUST_BACKTRACE=full
export RUST_LOG=debug
export DEBUG=true
export REDIS_PASSWORD=7tgbBSO2Yu
#export REDIS_SENTINEL_MASTER=mymaster
export REDIS_ADDRESS=localhost:6379
export REDIS_SENTINEL_ADDRESSES=localhost:26379
export PRIVATE_PORT=3003
export PUBLIC_PORT=3004
export NAMESPACE=rhiaqey

define CHANNELS
[
	{
		"Name": "sdf",
		"Size": 10
	},
	{
		"Name": "cokoland",
		"Size": 15
	}
]
endef

export CHANNELS

.PHONY: iss
iss:
	ID=pub1 \
	NAME=iss-position-1 \
		cargo +nightly run --bin iss-position

.PHONY: iss-prod
iss-prod:
	ID=pub1 \
	NAME=iss-position-1 \
		cargo +nightly run --release --bin iss-position

.PHONY: ticker
ticker:
	ID=ticker1 \
	NAME=ticker-1 \
	PRIVATE_PORT=3005 \
    PUBLIC_PORT=3006 \
		cargo +nightly run --bin ticker

.PHONY: build
build:
	cargo +nightly build

.PHONY: prod
prod:
	cargo +nightly build --release

.PHONY: redis
redis:
	docker run -it --rm --name redis -p 6379:6379 \
		-e ALLOW_EMPTY_PASSWORD=yes \
		bitnami/redis:7.0.9
