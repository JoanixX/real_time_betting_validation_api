# High Concurrency API Template (Rust)

## Overview

Este proyecto es una base arquitectónica profesional para construir servicios backend de alto rendimiento en Rust. Diseñado para soportar cargas de alta concurrencia, servir como punto de partida para APIs productivas y demostrar mejores prácticas en Backend Engineering.

**NO es un proyecto educativo básico.** Está configurado con prácticas de producción desde el día 1.

## Architecture

El proyecto sigue una arquitectura estratificada (Layered Architecture) para garantizar la separación de responsabilidades y la testabilidad.

### Estructura de Carpetas (Monorepo)

- **`backend/`**: Código fuente de la API en Rust.
  - `src/main.rs`: Entry Point, inicializa el servidor.
  - `src/handlers/`: Controladores HTTP.
  - `src/domain/`: Lógica de negocio pura.
  - `src/db/`: Configuración y pool de base de datos.
  - `migrations/`: Migraciones SQL (sqlx).

- **`frontend/`**: Código del cliente (Static/SPA).
  - `public/`: HTML/CSS/JS Assets.
  - `vercel.json`: Configuración de deploy.

- **`docker-compose.yml`**: Orquestación de servicios en local.

### Decisiones Técnicas

1.  **Rust & Actix-Web**: Elegidos por su performance "bare-metal", seguridad de memoria y ecosistema asíncrono maduro (`tokio`). Actix-Web domina en benchmarks de throughput.
2.  **SQLx**: Pure Rust SQL mapper. Provee verificación de queries en tiempo de compilación (compile-time checked queries) y manejo asíncrono nativo.
3.  **Observabilidad (Tracing)**: En lugar de logs de texto plano, usamos `tracing` con formato Bunyan (JSON) para permitir ingestión estructurada en ELK/Datadog. Esto es obligatorio para debugging en sistemas distribuidos.
4.  **Configuración Jerárquica**: Base YAML + Overrides por Entorno + Variables de Entorno. Permite gestión flexible de secretos y configs por deploy.
5.  **Docker Multi-stage Build**: Imagen final optimizada (distroless/slim) para reducir superficie de ataque y tamaño de imagen.

## Performance Tuning

### Database Pool

Configurado para alta concurrencia en `src/db/mod.rs`:

- `max_connections`: 100 (ajustar según límitaciones de la instancia DB)
- `max_lifetime`: 30 minutos (evita conexiones stale)
- `acquire_timeout`: 2 segundos (fail-fast si la DB está saturada)

### Load Testing

Se incluye configuración para **k6** en `k6/load_test.js`.
Objetivo: Validar que el sistema maneja picos de tráfico manteniendo latencias p95/p99 estables.

## Getting Started

### Prerrequisitos

- Rust & Cargo
- Docker & Docker Compose
- psql (PostgreSQL client)

### Setup Local

1.  **Levantar infrastructura**:

    ```bash
    docker-compose up -d
    ```

2.  **Preparar la Base de Datos**:

    ```bash
    # (Opcional si usas sqlx-cli)
    cargo install sqlx-cli
    sqlx database create
    sqlx migrate run
    ```

    _Nota: El script `scripts/init_db.sh` automatiza esto._

3.  **Ejecutar la API**:
    ```bash
    cargo run
    ```

### Ejecutar Tests

```bash
cargo test
```

### Ejecutar Load Tests (k6)

```bash
# Instalar k6 si no está instalado
k6 run k6/load_test.js
```

## Cloud Deployment

Para instrucciones detalladas sobre cómo desplegar en **AWS, Azure, Render y Vercel**, consulta [DEPLOYMENT.md](./DEPLOYMENT.md).

## Estado del Proyecto

Actualmente en **Fase de Inicialización**.

- [x] Estructura base completa
- [x] Dockerización optimizada
- [x] Configuración de logging/tracing
- [x] Conexión a BD resiliente
- [x] Frontend de pruebas (Glassmorphism UI)
- [x] Soporte para despliegue Cloud (AWS, Azure, Render, Vercel)
- [ ] Endpoints transaccionales complejos

## Contribución

Todo cambio debe pasar tests y `cargo clippy`.
Las migraciones de base de datos deben ser inmutables.
