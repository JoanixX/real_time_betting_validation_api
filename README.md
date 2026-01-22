# High Concurrency API Template (Rust)

## Vista General

Este proyecto es una base arquitectónica profesional para construir servicios backend de alto rendimiento en Rust. Diseñado para soportar cargas de alta concurrencia, servir como punto de partida para APIs productivas y demostrar mejores prácticas en Backend Engineering.

**NO es un proyecto educativo básico.** Está configurado con prácticas de producción desde el día 1.

## Arquitectura

El proyecto sigue una arquitectura estratificada (Layered Architecture) para garantizar la separación de responsabilidades y la testabilidad.

### Estructura de Carpetas (Enterprise Monorepo)

- **`backend/`**: API de alto rendimiento en Rust.
- **`frontend/`**: Cliente estático (Glassmorphism).
- **`infrastructure/`**: Orquestación y configuración de despliegue (Docker, Nginx).
- **`docs/`**: Documentación de arquitectura, decisiones técnicas (ADRs) y guías.
- **`scripts/`**: Herramientas de automatización para CI/CD y DB.

### Decisiones Técnicas

1.  **Rust & Actix-Web**: Performance "bare-metal" y seguridad de memoria.
2.  **Arquitectura Hexagonal**: Separación clara entre el dominio de negocio y la infraestructura.
3.  **SQLx**: Queries verificadas en compilación.
4.  **Observabilidad**: Tracing estructurado para depuración bajo alta carga.

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
