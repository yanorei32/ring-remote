FROM rust:1.93.0-bookworm AS build-env
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

FROM debian:bookworm-slim@sha256:74d56e3931e0d5a1dd51f8c8a2466d21de84a271cd3b5a733b803aa91abf4421

WORKDIR /

COPY --chown=root:root --from=build-env \
	/usr/src/ring-remote/CREDITS \
	/usr/src/ring-remote/LICENSE \
	/usr/share/licenses/ring-remote/

COPY --chown=root:root --from=build-env \
	/usr/src/ring-remote/target/release/ring-remote \
	/usr/bin/ring-remote

CMD ["/usr/bin/ring-remote"]
