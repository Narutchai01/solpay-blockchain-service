# build state 
FROM rust:1.93.1-bookworm AS base



FROM base AS builder


# สร้าง Working directory
WORKDIR /app

# Trick: Copy แค่ Cargo.toml และ Cargo.lock มาก่อนเพื่อทำ Dependency Caching
# วิธีนี้จะช่วยให้ไม่ต้องโหลด library ใหม่ทุกครั้งที่แก้ Code
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -f target/release/deps/solpay-blockchain-service*

# Copy source code จริงลงไป
COPY . .

# Build โปรเจกต์ (ตั้งชื่อ binary ตามที่ระบุใน Cargo.toml)
RUN cargo build --release

# --- Stage 2: Run ---
# ใช้ image เล็กๆ อย่าง debian-slim หรือ alpine สำหรับรัน
FROM debian:bookworm-slim

WORKDIR /app

# ติดตั้ง dependencies ที่จำเป็น (เช่น OpenSSL ถ้ามีการใช้)
RUN apt-get update && apt-get install -y ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy binary ที่ build เสร็จแล้วมาจาก Stage แรก
COPY --from=builder /app/target/release/solpay-blockchain-service .

# สั่งให้รันแอปพลิเคชัน
CMD ["./solpay-blockchain-service"]