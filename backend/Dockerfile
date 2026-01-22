# Etapa de construcción (Builder)
FROM rust:1.75-slim-bullseye AS builder

WORKDIR /app

# Copiamos los archivos de dependencias primero para aprovechar el caché de Docker
COPY Cargo.toml Cargo.lock ./

# Creamos un main dummy para compilar solo las dependencias y cachearlas
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Compilamos solo las dependencias
RUN cargo build --release

# Borramos el main dummy para copiar el código real
RUN rm src/main.rs

# Copiamos el código fuente real
COPY . .

# Compilamos la aplicación real
# Hacemos touch al main para forzar la recompilación del código fuente
RUN touch src/main.rs && cargo build --release

# Etapa final (Runtime)
FROM debian:bullseye-slim

WORKDIR /app

# Instalamos certificados SSL si hacen falta (aunque rustls ayuda, a veces se necesitan roots)
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copiamos el binario desde la etapa de construcción
COPY --from=builder /app/target/release/high_concurrency_api high_concurrency_api

# Copiamos la configuración necesaria
COPY configuration configuration
COPY migrations migrations

# Exponemos el puerto de la aplicación
EXPOSE 8000

# Seteamos el entorno a producción por defecto
ENV APP_ENVIRONMENT=production

# Ejecutamos el binario
CMD ["./high_concurrency_api"]
