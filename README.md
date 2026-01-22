# High Concurrency API Template (Rust)

## Vista General

Este proyecto es una base arquitectónica profesional para construir servicios backend de alto rendimiento en Rust. Diseñado para soportar cargas de alta concurrencia, servir como punto de partida para APIs productivas y demostrar mejores prácticas en Backend Engineering.

**NO es un proyecto educativo básico.** Está configurado con prácticas de producción desde el día 1.

## Arquitectura

El proyecto sigue una **Arquitectura en Capas (Layered Architecture)** optimizada para el rendimiento. A diferencia de arquitecturas más pesadas como la Hexagonal completa, este enfoque minimiza el "overhead" de abstracción en Rust, permitiendo que la ruta crítica de ejecución sea lo más directa posible.

### Estructura de Carpetas (Enterprise Monorepo)

- **`backend/`**: API de alto rendimiento en Rust (Actix-web).
- **`frontend/`**: Cliente estático (Glassmorphism & Vanilla JS).
- **`infrastructure/`**: Orquestación y configuración de despliegue (Docker, Docker-compose).
- **`docs/`**: Documentación de arquitectura, decisiones técnicas (ADRs) y guías.
- **`scripts/`**: Herramientas de automatización para CI/CD y DB.

### Decisiones Técnicas (Senior Rationales)

1.  **Rust & Actix-Web**: Elección basada en la necesidad de latencia cercana al metal y manejo eficiente de miles de conexiones concurrentes.
2.  **Arquitectura en Capas**: Estrategia de "Zero Cost Abstractions" donde la lógica de negocio y la persistencia se integran de forma eficiente sin intermediarios innecesarios.
3.  **SQLx & Connection Pooling**: Uso de `PgPool` configurado para manejar picos de tráfico sin degradación.
4.  **Observabilidad Estructurada**: Logs en formato JSON listos para ser ingeridos por sistemas como ELK o Datadog.

---

## Getting Started

### Setup Local

1.  **Levantar Infraestructura**:

    ```bash
    cd infrastructure
    docker-compose up -d
    ```

2.  **Preparar Base de Datos**:

    ```bash
    ./scripts/setup_db.sh
    ```

3.  **Ejecutar API**:
    ```bash
    cd backend
    cargo run
    ```

## Documentación

- **Arquitectura:** [docs/architecture.md](./docs/architecture.md)
- **Despliegue Cloud:** [docs/deployment.md](./docs/deployment.md)

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
