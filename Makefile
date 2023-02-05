# export RUST_BACKTRACE=full
export RUST_LOG=trace
export DEBUG=true
export REDIS_PASSWORD=oyVgWXEZVn
export REDIS_SENTINEL_MASTER=mymaster
#export REDIS_ADDRESS=localhost:6379
export REDIS_SENTINEL_ADDRESSES=localhost:26379

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
	ID=1 \
	NAME=iss-position \
	NAMESPACE=iss \
		cargo +nightly run --bin iss_position

.PHONY: build
build:
	cargo build

.PHONY: prod
prod:
	cargo +nightly build --release