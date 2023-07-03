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
RUN cargo build

# Remove any source code that cargo generated.
RUN rm src/*.rs

# Now that the dependency is built, copy your source code and build for real.
COPY ./src ./src
RUN cargo build

# Final base image.
FROM debian:buster-slim

# Copy the build artifact.
COPY --from=build /photo_manager_server/target/debug/photo_manager_server .

# Set startup command.
CMD ["./photo_manager_server"]
