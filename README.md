# Solpay Blockchain Service

A worker node / consumer service with health check endpoints and RabbitMQ integration, built with Clean Architecture principles.

## Architecture

```
src/
├── domain/              # Core business logic (entities, errors, ports)
│   ├── entities/        # Business entities
│   ├── errors/          # Domain errors
│   └── ports/           # Interfaces (traits) for external services
├── application/         # Use cases and services
│   ├── services/        # Application services (e.g., HealthService)
│   └── use_cases/       # Business use cases (e.g., ProcessMessage)
├── infrastructure/      # External implementations
│   ├── config.rs        # Configuration
│   └── rabbitmq/        # RabbitMQ consumer implementation
├── presentation/        # HTTP layer
│   └── http/            # Actix-web handlers and server
├── lib.rs               # Library root
└── main.rs              # Application entry point
```

## Features

- **Health Check Endpoints**: `/health`, `/healthz`, `/livez`, `/readyz`
- **RabbitMQ Consumer**: Consumes messages from a configured queue
- **Clean Architecture**: Separation of concerns with domain, application, infrastructure, and presentation layers
- **Graceful Shutdown**: Handles SIGINT for clean shutdown
- **Auto-reconnect**: Automatically reconnects to RabbitMQ on connection failure

## Configuration

Create a `.env` file based on `.env.example`:

```bash
cp .env.example .env
```

### Environment Variables

| Variable                 | Default                             | Description                |
| ------------------------ | ----------------------------------- | -------------------------- |
| `SERVER_HOST`            | `0.0.0.0`                           | HTTP server host           |
| `SERVER_PORT`            | `8080`                              | HTTP server port           |
| `RABBITMQ_URL`           | `amqp://guest:guest@localhost:5672` | RabbitMQ connection URL    |
| `RABBITMQ_QUEUE_NAME`    | `solpay_queue`                      | Queue name to consume from |
| `RABBITMQ_EXCHANGE_NAME` | `solpay_exchange`                   | Exchange name (optional)   |
| `RABBITMQ_ROUTING_KEY`   | `solpay.#`                          | Routing key for binding    |

## Running

### Prerequisites

- Rust 1.75+
- RabbitMQ server

### Development

```bash
# Run with cargo
cargo run

# Run with logging
RUST_LOG=debug cargo run
```

### Production

```bash
# Build release
cargo build --release

# Run
./target/release/solpay-blockchain-service
```

## Health Check Endpoints

| Endpoint       | Description                                      |
| -------------- | ------------------------------------------------ |
| `GET /health`  | Full health status including RabbitMQ connection |
| `GET /healthz` | Alias for `/health`                              |
| `GET /livez`   | Liveness probe (always returns 200 OK)           |
| `GET /readyz`  | Readiness probe (checks RabbitMQ connection)     |

### Response Example

```json
{
  "status": "healthy",
  "service": "solpay-blockchain-service",
  "version": "0.1.0",
  "rabbitmq_connected": true
}
```

## Message Format

Messages consumed from RabbitMQ should be in this JSON format:

```json
{
  "id": "unique-message-id",
  "message_type": "payment.created",
  "payload": {
    "key": "value"
  },
  "created_at": "2024-01-01T00:00:00Z"
}
```

## License

MIT
