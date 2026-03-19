# Real-Time Betting Validation API (High Concurrency)

API de alto rendimiento desarrollada en **Rust** con **Arquitectura Hexagonal** (Ports & Adapters), diseñada para la validación crítica de apuestas en eventos en vivo. El motor está optimizado para baja latencia y alta disponibilidad, capaz de procesar ráfagas masivas de transacciones concurrentes.

## 🚀 Enfoque Principal: Alta Concurrencia

Este proyecto no es solo una API CRUD; es un ejercicio de ingeniería de rendimiento que implementa:

- **Arquitectura Hexagonal**: Dominio puro desacoplado de la infraestructura mediante puertos (traits) y adaptadores.
- **Asincronía Extrema**: Construido sobre `Actix-Web` y `Tokio` para maximizar el uso de CPU.
- **Validación con Baja Latencia**: Procesamiento sub-10ms por ticket de apuesta.
- **Eventual Consistency (Write-Behind Cache)**: Inserciones asíncronas hacia Postgres valiéndose de un background worker consumiendo Redis Streams.
- **SRE & Seguridad**: Middlewares para Rate Limiting (Token Bucket) y telemetría avanzada expuesta para Prometheus (`/metrics`).
- **Pooling Eficiente**: Conexiones a base de datos (Postgres via SQLx) y caché (Redis) optimizadas.
- **Observabilidad**: Tracing estructurado para identificar cuellos de botella en milisegundos.

## 🛠️ Stack Tecnológico

- **Backend**: Rust 1.88+ (Actix-Web, SQLx, Redis-RS). Ver [`backend/README.md`](backend/README.md) para detalles de arquitectura.
- **Caché**: Redis / Upstash (capa de validación rápida y concurrencia).
- **Persistencia**: PostgreSQL (Neon/Docker).
- **Infraestructura**: Docker Compose + GitHub Actions CI (strict clippy & rustfmt).
- **Testing**: k6 (Load Testing) + Integration Tests con infraestructura real.
- **Frontend UI**: Next.js 14 (App Router) y Zustand O(1).
    - **Fintech Precision**: Lógica monetaria en centavos enteros (zero floats).
    - **High Frequency UI**: Feedback visual instantáneo (Flash) con _Zero React Renders_.
    - **Sección de Historial**: Gestión de activos y resueltos con TanStack Query y composición de datos.
    - Ver [`frontend/README.md`](frontend/README.md).

## 📂 Estructura del Monorepo

```
real_time_betting_validation_api/
├── backend/                  ← API en Rust (Actix-Web, Arquitectura Hexagonal)
├── frontend/                 ← Next.js 14 (App Router, Zustand, TanStack Query)
├── infrastructure/           ← Docker Compose para servicios locales
├── scripts/                  ← Scripts de utilidad (setup de BD, etc.)
└── README.md                 ← Este archivo
```

## 📊 Simulación & Pruebas de Estrés

### 1. Levantar Infraestructura

```bash
cd infrastructure
docker-compose up -d
```

### 2. Ejecutar el Motor (Backend)

```bash
cd backend
sqlx migrate run
cargo run --release
```

### 3. Simulador UI (Frontend)

El dashboard interactivo provee un cliente de Real-Time Betting alimentado del Motor (Snapshot REST + Delta Websocket), probando el feedback High Frequency.

```bash
cd frontend
npm install
npm run dev
```

### 🚀 4. Load Testing con k6

Para validar que el sistema soporta miles de peticiones por segundo:

```bash
# Requiere k6 instalado localmente
cd backend/k6
k6 run load_test.js
```

## ⚙️ Variables de Entorno

Cada servicio tiene su propio `.env.example` con la plantilla de variables necesarias:

- **Backend**: `backend/.env.example`
- **Frontend**: `frontend/.env.example`

Copiar como `.env` (local) o `.env.production` (producción) y rellenar con los valores reales.

---

## License

This project is source-available under a custom proprietary license.
You may review the code, test it privately, make private evaluation-only modifications, and submit pull requests.
You may **not**, without prior written permission:

- sell or redistribute copies of the software;
- publish forks, binaries, or container images;
- offer it as SaaS, hosting, or an API service;
- use it in production or in commercial business operations;
- integrate it into another backend, platform, or product;
- provide paid setup, customization, support, or consulting services for third parties using this software.

Companies may evaluate the software privately and temporarily before entering a paid agreement, but that evaluation must remain non-production, non-commercial, and non-redistributive.

Contact: alvaradocjosorio@gmail.com / +51 933984317

**Desarrollado por Joaquin Alvarado | 2026**
