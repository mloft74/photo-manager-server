# https://dev.to/rogertorres/first-steps-with-docker-rust-30oi

# Rust as the base image.
FROM rust:1.70.0 as build

# Create a new empty shell project.
RUN cargo new --bin photo_manager_server
WORKDIR /photo_manager_server

# Copy our manifests.
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# Build only the dependencies to cache them.
RUN cargo build --release

# Remove any source code that cargo generated.
RUN rm src/*.rs

# Now that the dependency is built, copy your source code and build for real.
COPY ./src ./src
COPY ./migrations ./migrations
RUN cargo build --release

# Final base image.
FROM debian:trixie-slim

RUN apt-get update
RUN apt-get -y install --no-install-recommends libpq-dev

# Copy the build artifact.
COPY --from=build /photo_manager_server/target/release/photo_manager_server .
COPY ./.env ./.env

# Set startup command.
CMD ["./photo_manager_server"]
