export RUST_BACKTRACE=full
export RUST_LOG=rhiaqey=trace
export DEBUG=true
export REDIS_PASSWORD=7tgbBSO2Yu
#export REDIS_SENTINEL_MASTER=mymaster
export REDIS_ADDRESS=localhost:6379
export REDIS_SENTINEL_ADDRESSES=localhost:26379
export PRIVATE_PORT=3003
export PUBLIC_PORT=3004
export NAMESPACE=rhiaqey

export SECRET=92c54ddb6370d95e04f679dc773af83da4f359265909289e330c51651e840250

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

define PUBLIC_KEY
-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAw8mFnJJrfA+r0eVz4kAT
YYEXaG1mTvgR5COuijDOWOTwm0Hd5ppkzSduCo2ci5h470FmNyOUNB8CkzDxrhCi
9HvZhjykewDhbgR0EKjfk6HbnZDfOCupeGx1YdgYXriXkVqGtcD0+1DV94P8PdBO
cbNn4cfgLLos8zOlMEKcPsO7wwdlRiW9J+60IybFloQeJlWOaD/oT+j/EB0oiKBg
LcoEkKiMTqkRfiW/tFKzLjIqEmID619jJbPUUEFHY1pp/3otilhTqgxDG1r59efP
24eIpYwxZO1GE/Mjbe6ANHpAPYMH1quXDjb0Y5SERHfWcjq30jAfolaGIlrUSw6d
3wIDAQAB
-----END PUBLIC KEY-----
endef

export PUBLIC_KEY

define PRIVATE_KEY
-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQDDyYWckmt8D6vR
5XPiQBNhgRdobWZO+BHkI66KMM5Y5PCbQd3mmmTNJ24KjZyLmHjvQWY3I5Q0HwKT
MPGuEKL0e9mGPKR7AOFuBHQQqN+TodudkN84K6l4bHVh2BheuJeRWoa1wPT7UNX3
g/w90E5xs2fhx+AsuizzM6UwQpw+w7vDB2VGJb0n7rQjJsWWhB4mVY5oP+hP6P8Q
HSiIoGAtygSQqIxOqRF+Jb+0UrMuMioSYgPrX2Mls9RQQUdjWmn/ei2KWFOqDEMb
Wvn158/bh4iljDFk7UYT8yNt7oA0ekA9gwfWq5cONvRjlIREd9ZyOrfSMB+iVoYi
WtRLDp3fAgMBAAECggEAfPnkih+E8Ppn6WIYaPIR7QmkUYqT5hDACusj/R5Oebwa
QmD3Lr6bXcGvopjmts0rVT5f6w6RCfxJfn+dpkkEXB+6qM+JBuN3Au1g0Uma/fgx
4hCaDJcCZNaGz2BLnhsi1Sv+FYMIXmwpSQg9OZAAot+sjhkyZhqpmsz6wyWh6wWT
zefxOWOyw9LHHCO11urOYKs4qGMffMkJTZ9ZBAfXKDqxFyV6bbnO+Pu/QYEMlgAF
kyTJMVCrbrf5uJUaXsW32C0vfM98yzOzU5YDgEy71rb2Uf7lnCVj1Ofk3ZE4izvm
BHMIXeozMssZMgwfnK2M846JVVvVep/d/j8nZoEBkQKBgQDOVj2J2kcCyvXJyjK9
f8B+gw66xq+cW9Zlx1ZRmtUpjEGoSkOmL7naGrAMFtITiDW5ZBHIiTcnX7XQJMrE
U0TS9ks04QV6Tl0vW+Yr3V3IeZtOpsb8Nl+9boIAdPpx2yBuGV706tcQSGuqghVd
7P8u9eTYua8CW4zig4YGJ8uiFQKBgQDy6T81YYjoga3vPMY/BnhlTGVBr2tcUTtH
uHbnW3LamCu9snvyf8qOhja08Q4g7uOidh9lm1mjHBie8FghGpWws43fmKX6TQft
jgcQnLFdGDQJq5y2OM2LB/EeoYfdK6kYsi22j6GnrUQAHDwuTeS2mysRF5Eqppjs
vBAXpXfhIwKBgQCE0YqnU/Rl3dO9YwSqarO0PBSdMgwUsCEgPuJXgT05k2koNTW6
ofoWZRtxjLcJj6JVhg7UcU8pbziPlT9YhOlGivf6P+bQxeTB+Xv+PG6D/5NzW3O3
IiEaxSm1tZcI9y628GnpacmqV5PGnBm47jeNOQdoYo4/DENyA4ugJrmzyQKBgQDD
W2kVYlq8O0cKh8McfvSm61joCc97UG0vkiA2kyp8uTM8feYHMlVSaIho3xEw1U9H
ol4/1j+x2W/Hq54FCZ9nnBA2ykp6UidVGwt9hbdzGnsHZ/hB6M8NyJZXvytIacu1
696t2zf0ZXmx6QNRbh3J6mMpfN2oApIsmlcK3W3bJwKBgECmpSb07IR1xD339Wdx
RUmyeXHNznAstW5qDIpwh4cjOjhjvkEsBPDpnpdu5P3lXQD7rae4c+vV/2rVCj1/
e0HojIfYZKE3WWfx9hMB1V7EF5lWLEfmSzY6ARit6MP27PsO7FSDbGXZ/kVBhWWn
C3mxja/ej85UOxy8VhmWr7WK
-----END PRIVATE KEY-----
endef

export PRIVATE_KEY

.PHONY: iss
iss:
	ID=pub1 \
	NAME=iss-position-1 \
		cargo +nightly run --bin iss-position --features=iss

.PHONY: iss-prod
iss-prod:
	ID=pub1 \
	NAME=iss-position-1 \
		cargo +nightly run --release --bin iss-position --features=iss

.PHONY: ticker
ticker:
	ID=ticker1 \
	NAME=ticker-1 \
	PRIVATE_PORT=3005 \
    PUBLIC_PORT=3006 \
		cargo +nightly run --bin ticker --features=ticker

.PHONY: ticker-prod
ticker-prod:
	ID=ticker1 \
	NAME=ticker-1 \
	PRIVATE_PORT=3005 \
    PUBLIC_PORT=3006 \
		cargo +nightly run --release --bin ticker --features=ticker

.PHONY: pinger
pinger:
	ID=pinger1 \
	NAME=pinger-1 \
	PRIVATE_PORT=3007 \
    PUBLIC_PORT=3008 \
		cargo +nightly run --bin pinger --features=pinger

.PHONY: pinger-prod
pinger-prod:
	ID=pinger1 \
	NAME=pinger-1 \
	PRIVATE_PORT=3007 \
    PUBLIC_PORT=3008 \
		cargo +nightly run --release --bin pinger --features=pinger

.PHONY: rss
rss:
	ID=rss1 \
	NAME=rss-1 \
	PRIVATE_PORT=3009 \
    PUBLIC_PORT=3010 \
		cargo +nightly run --bin rss --features=rss

.PHONY: rss-prod
rss-prod:
	ID=rss1 \
	NAME=rss-1 \
	PRIVATE_PORT=3009 \
    PUBLIC_PORT=3010 \
		cargo +nightly run --release --bin rss --features=rss

.PHONY: ecb
ecb:
	ID=ecb1 \
	NAME=ecb-1 \
	PRIVATE_PORT=3011 \
    PUBLIC_PORT=3012 \
		cargo +nightly run --bin ecb-daily --features=ecb

.PHONY: ecb-prod
ecb-prod:
	ID=ecb1 \
	NAME=ecb-1 \
	PRIVATE_PORT=3011 \
    PUBLIC_PORT=3012 \
		cargo +nightly run --release --bin ecb-daily --features=ecb

.PHONY: build
build:
	cargo +nightly build

.PHONY: prod
prod:
	cargo +nightly build --release --bin rss --features=rss
	cargo +nightly build --release --bin pinger --features=pinger
	cargo +nightly build --release --bin ticker --features=ticker
	cargo +nightly build --release --bin iss-position --features=iss
	ls -lah target/release
