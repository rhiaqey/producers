# export RUST_BACKTRACE=full
export RUST_LOG=trace
export DEBUG=true
export REDIS_PASSWORD=7tgbBSO2Yu
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

define SETTINGS
{
	"Url": "http://localhost:8080/feed.xml",
	"Api": "123",
	"Interval": 15000,
	"Timeout": 30000,
	"WhitelistedIPs": [ "127.0.0.1", "FE80::903A:1C1A:E802:11E4" ],
	"WebhookEndpoint": "https://hook.eu1.make.com/vgg0c4ggjkix5x32wkalis47o15zulsx"
}
endef

export SETTINGS

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