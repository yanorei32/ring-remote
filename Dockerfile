FROM rust:1.97.0-bookworm AS build-env
LABEL maintainer="yanorei32"

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

WORKDIR /usr/src
COPY . /usr/src/ring-remote/
WORKDIR /usr/src/ring-remote
RUN cargo build --release && cargo install cargo-license && cargo license \
	--authors \
	--do-not-bundle \
	--avoid-dev-deps \
	--avoid-build-deps \
	--filter-platform "$(rustc -vV | sed -n 's|host: ||p')" \
	> CREDITS

FROM debian:bookworm-slim@sha256:7b140f374b289a7c2befc338f42ebe6441b7ea838a042bbd5acbfca6ec875818

WORKDIR /

COPY --chown=root:root --from=build-env \
	/usr/src/ring-remote/CREDITS \
	/usr/src/ring-remote/LICENSE \
	/usr/share/licenses/ring-remote/

COPY --chown=root:root --from=build-env \
	/usr/src/ring-remote/target/release/ring-remote \
	/usr/bin/ring-remote

CMD ["/usr/bin/ring-remote"]
