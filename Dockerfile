# Build stage
FROM rust:1.86.0-slim as builder

WORKDIR /usr/src/app
COPY . .

# Build the application
RUN cargo build --release

# Final stage
FROM debian:bookworm-slim

# Copy the binary from the builder stage
COPY --from=builder /usr/src/app/target/release/minimal-async /usr/local/bin/minimal-async

# Run the application
CMD ["minimal-async"] 