FROM rust:1.48 AS planner

WORKDIR app

# Only pay the installation cost once, it will be cached from the second 
# build onwards.
# To ensure a reproducible build consider pinning the cargo-chef version 
# with `--version X.X.X`
RUN cargo install cargo-chef

COPY . .

# Compute a lock-like file for the project
RUN cargo chef prepare --recipe-path recipe.json

###############################################################################

FROM rust:1.48 As cacher

WORKDIR app

RUN cargo install cargo-chef

COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

###############################################################################

# Builder stage
FROM rust:1.48 AS builder

# Switch the working directory to `app` (equivalent to `cd app`)
# The `app` folder will be created by Docker in case it does not exist.
WORKDIR app

# Copy over the cached dependencies
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo

# Copy all files from the working environment to the Docker image
COPY . .

# Built a binary!
ENV SQLX_OFFLINE true
RUN cargo build --release --bin zero2prod

###############################################################################

# Runtime stage
FROM debian:buster-slim AS runtime

WORKDIR app

# Install OpenSSL - it is dynamically linked by some of the dependencies
RUN apt-get update -y \
	&& apt-get install -y --no-install-recommends openssl \
	# Clean up
	&& apt-get autoremove -y \
	&& apt-get clean -y \
	&& rm -rf /var/lib/apt/lists/*


# Copy the compiled binary from the builder environment to the runtime 
# environment
COPY --from=builder /app/target/release/zero2prod zero2prod

# We need the configuration files at runtime!
COPY configuration configuration

# When `docker run` is executed, launch the binary!
ENV APP_ENVIRONMENT production
ENTRYPOINT ["./zero2prod"]
