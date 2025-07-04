export REDIS_MODE=standalone
export REDIS_PASSWORD=welcome
export REDIS_ADDRESS=0.0.0.0:6379
export REDIS_SENTINEL_MASTER=mymaster
export REDIS_SENTINEL_ADDRESSES=localhost:26379,localhost:26380,localhost:26381
export REDIS_VERSION=7.4.0
export REDIS_INSIGHT_VERSION=2.54.0

export RUST_BACKTRACE=full
export RUST_LOG=rhiaqey=trace
export DEBUG=true
export PRIVATE_PORT=3003
export PUBLIC_PORT=3004
export NAMESPACE=rhiaqey

define PUBLIC_KEY
-----BEGIN RSA PUBLIC KEY-----
MIIBCgKCAQEAwWOo7UYK8upVY3qf1zvpwdyVL+4KWwKx4lKQXd5ljiEjNBdhQlRP
869LFR+k4CMIYqKYSGzbYpvfXOwXNHEjfwXiEnm8gro8cGTRdb7n9jKpN7UXMIez
DRflWd8K8Cma4DQPethmNiCtpMoHlINYgNMFTtbK9QOaFKO1JZVUyrHN+qsmtkPO
dMJ68zHiQMtWs00eABdPtS3cSmvfkk7Dz30pcNXdHuYtEQx3KAfqRIJ49F1vnu17
D6Sw5fRD+IxhICkuKjATBqAkpYk5sP4Xcd2QYqU8qFTmu3WrwgHEHAZ85L3kNxKb
Hh3mF9PnLxlo4WQANH65Ej51HVGOKw4VHQIDAQAB
-----END RSA PUBLIC KEY-----
endef

export PUBLIC_KEY

define PRIVATE_KEY
-----BEGIN RSA PRIVATE KEY-----
MIIEpQIBAAKCAQEAwWOo7UYK8upVY3qf1zvpwdyVL+4KWwKx4lKQXd5ljiEjNBdh
QlRP869LFR+k4CMIYqKYSGzbYpvfXOwXNHEjfwXiEnm8gro8cGTRdb7n9jKpN7UX
MIezDRflWd8K8Cma4DQPethmNiCtpMoHlINYgNMFTtbK9QOaFKO1JZVUyrHN+qsm
tkPOdMJ68zHiQMtWs00eABdPtS3cSmvfkk7Dz30pcNXdHuYtEQx3KAfqRIJ49F1v
nu17D6Sw5fRD+IxhICkuKjATBqAkpYk5sP4Xcd2QYqU8qFTmu3WrwgHEHAZ85L3k
NxKbHh3mF9PnLxlo4WQANH65Ej51HVGOKw4VHQIDAQABAoIBAFUt0kAAM95evJF+
d1zT9NgAkm10CXegrj0jZJcT1+NMUTcmfR48CKMquIVrVLGsfIsFVtG/sLm0MiO5
kVb15k6Shsrgd9mUsf5HScL0/TKBiesRhk9H1eOUfN6i0SyLBr5t78uJ+SsqJZGJ
suEFITxMte/Nx8M3fOxOVwFgzuIfvrn/b5ylsWHOVaKv3nH5wIXRenBUHHSoW/93
FUDyr4ylk3K0ipYkCKCK2ZU/C/GfJSWPiWeJCA+8u9avZDc4Jo5VgNg1bAGeISjI
DW5K6MfKZQRUMf5Yxp7o8Vn0KCVLCZ135OUATt87LrpAcuVhohk5f95zwwg+RDHV
a2BJPHkCgYEA3hz4DFRWT9SL6ISqwmgKLG4TPt7BCIGudS5rUI4usFpPxclJ92d/
anbrbaysyQooFHZbSjLPrx+mAyoyl8X/o4Vg7dYFRqqYrR0qIDNlsSFCLmFoPJsf
iq9aMdvbCZ4fgiU+hhFqf9ggwi4cfBLJ0dLRkNSvvT1WkjAqyh2K5sMCgYEA3uTU
GPsF5pHiU1l15EBWb7cQHbnst67nx6wAF1NgFbvnKsloILjHhvJt2v3y6eHVzPmz
eTP5eX0NGkoxSM+XUGAEwoojj0N60oQfHq89D8l1FMnnycVrYNbPpcpcj06InCI6
DD8v/eegdtJB72nRRVWYVPJEbQMijpXe1L78Fp8CgYEAux+yxkhjMvxBJpJmfkRC
le3inuvxuqXugWCrHoG6ye48GMidXSa/zMUFUS/RncnFvH9+J4OpPsOuDCqH1yAD
YBMldxoA9ekRmX7hl6FVgiYf6I090RlrOF7E4Q51eaPSrcWM6ExR6gT+jDlm3AzE
JNa0oYzdxdOgVKbp1b+P3xsCgYEAxbrFlOuKzoH3/uzLspKIm04Qk+5N2pzkrLMe
2ZJzGJS6e7B0GSMSjdoeLjk99tEKs22IEytSUr3mk73hfq/5kam8Tz/wT7UTDhF6
8eOPDaQvoyOB5fKmUR/+0Rp1hgOrGKccS6T6VAnYxc+8AkEjDpjiK+lHXlV7oHW/
WYebZg0CgYEA05xqLd7BviuWt0alFWxn3d73fHfDxMljk81w3/7XraRotiLsvoFG
65/r/v/BefLY8aJmIRuZC26HYyDgNXoe+h9Izp4mg+5PXpTWJx5Lx43/yErIg+HG
Xzq0hM31N+8073JWS+QRP1YJJWHqDhJbTJOzK2VepVubjEmPTY1DeTw=
-----END RSA PRIVATE KEY-----
endef

export PRIVATE_KEY

.PHONY: yahoo
yahoo:
	ID=yahoo-1 \
	NAME=yahoo-1 \
		cargo run --bin yahoo --features yahoo

.PHONY: yahoo-prod
yahoo-prod:
	ID=yahoo-1 \
	NAME=yahoo-1 \
		cargo run --release --bin yahoo --features yahoo

.PHONY: iss
iss:
	ID=pub1 \
	NAME=iss-position-1 \
		cargo run --bin iss-position --features iss-position

.PHONY: iss-prod
iss-prod:
	ID=pub1 \
	NAME=iss-position-1 \
		cargo run --release --bin iss-position --features iss-position

.PHONY: iss-astros
iss-astros:
	ID=pub13 \
	PRIVATE_PORT=3013 \
	PUBLIC_PORT=3014 \
	NAME=iss-astros-1 \
		cargo run --bin iss-astros --features iss-astros

.PHONY: iss-astros-prod
iss-astros-prod:
	ID=pub13 \
	NAME=iss-astros-1 \
		cargo run --release --bin iss-astros --features iss-astros

.PHONY: ticks
ticks: ticker

.PHONY: ticker
ticker: ticker1

.PHONY: ticker1
ticker1:
	ID=ticker1 \
	NAME=ticker-1 \
	PRIVATE_PORT=3005 \
    PUBLIC_PORT=3006 \
		cargo run --bin ticker --features ticker

.PHONY: ticker2
ticker2:
	ID=ticker2 \
	NAME=ticker-1 \
	PRIVATE_PORT=3115 \
    PUBLIC_PORT=3116 \
		cargo run --bin ticker --features ticker

.PHONY: ticker-prod
ticker-prod:
	ID=ticker1 \
	NAME=ticker-1 \
	PRIVATE_PORT=3005 \
    PUBLIC_PORT=3006 \
		cargo run --release --bin ticker --features ticker

.PHONY: pinger
pinger:
	ID=pinger1 \
	NAME=pinger-1 \
	PRIVATE_PORT=3007 \
    PUBLIC_PORT=3008 \
		cargo run --bin pinger --features pinger

.PHONY: pinger-prod
pinger-prod:
	ID=pinger1 \
	NAME=pinger-1 \
	PRIVATE_PORT=3007 \
    PUBLIC_PORT=3008 \
		cargo run --release --bin pinger --features pinger

.PHONY: rss
rss:
	ID=rss1 \
	NAME=rss-1 \
	PRIVATE_PORT=3009 \
    PUBLIC_PORT=3010 \
		cargo run --bin rss --features rss

.PHONY: rss-prod
rss-prod:
	ID=rss1 \
	NAME=rss-1 \
	PRIVATE_PORT=3009 \
    PUBLIC_PORT=3010 \
		cargo run --release --bin rss --features rss

.PHONY: ecb
ecb:
	ID=ecb1 \
	NAME=ecb-1 \
	PRIVATE_PORT=3011 \
    PUBLIC_PORT=3012 \
		cargo run --bin ecb-daily --features ecb-daily

.PHONY: ecb-prod
ecb-prod:
	ID=ecb1 \
	NAME=ecb-1 \
	PRIVATE_PORT=3011 \
    PUBLIC_PORT=3012 \
		cargo run --release --bin ecb-daily --features ecb-daily

.PHONY: ctrader
ctrader:
	ID=ctrader1 \
	NAME=ctrader-1 \
	PRIVATE_PORT=3013 \
    PUBLIC_PORT=3014 \
		cargo run --bin ctrader-fix --features ctrader-fix

.PHONY: build
build: prod

.PHONY: prod
prod:
	cargo build --release --all-features -j 64
	ls -lah target/release

.PHONY: dev
dev:
	cargo build --all-features -j 64
	ls -lah target/debug

.PHONY: docker-multi
docker-multi:
	docker buildx build \
		--platform linux/amd64 \
		--build-arg BINARY=ticker \
		--build-arg FEATURES=ticker \
		-t rhiaqey/ticker:dev \
		--progress=plain \
		--no-cache \
		.

.PHONY: hub
hub:
	docker run -it --rm --name hub -p 3000:3000 -p 3001:3001 \
		-e REDIS_PASSWORD=${REDIS_PASSWORD} \
		-e PUBLIC_KEY=/certs/pub.pem \
		-e PRIVATE_KEY=/certs/priv.pem \
		-e PRIVATE_PORT=3000 \
		-e PUBLIC_PORT=3001 \
		-v ./certs:/certs \
		--entrypoint 'rhiaqey-hub' \
		--network host \
		rhiaqey/hub:latest 'run'

.PHONY: redis
redis:
	docker run -it --rm --name redis -p 6379:6379 \
		-e ALLOW_EMPTY_PASSWORD=no \
		-e REDIS_PASSWORD=${REDIS_PASSWORD} \
		--network host \
		rhiaqey/redis:${REDIS_VERSION}

.PHONY: redisinsight
redisinsight:
	docker run -it --rm --name redisinsight -p 5540:5540 \
		--network host \
		redis/redisinsight:${REDIS_INSIGHT_VERSION}

.PHONY: all
all:
	cargo b --release --all-features
