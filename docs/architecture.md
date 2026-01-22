# Vista General de la Arquitectura del Sistema

## Estrategia de Monorepo

El proyecto está organizado como un monorepo para mantener la atomicidad entre la API y el Cliente.

## Capas

- **Frontend (Lado del Cliente):** Implementación pura de archivos estáticos servida vía CDN (producción) o servidor estático local.
- **Backend (Lado del Servidor):** API de Rust de alto rendimiento utilizando Actix-Web. Sigue la **Arquitectura Hexagonal**.
- **Infraestructura:** Orquestación de contenedores a través de Docker y configuraciones listas para la nube.

## Arquitectura del Backend (Hexagonal / Puertos y Adaptadores)

1. **Capa de Dominio:** Lógica de negocio pura. Sin dependencias de bases de datos o frameworks web.
2. **Handlers (Adaptadores):** Traduce las peticiones HTTP en comandos de dominio.
3. **Almacenamiento (Adaptadores):** Implementación de SQLx para PostgreSQL.
4. **Configuración:** Configuración jerárquica basada en el entorno.

## Flujo de Datos

`Cliente -> Balanceador de Carga (opcional) -> Handlers de la API -> Lógica de Dominio -> Pool de DB -> PostgreSQL`

## Escalabilidad

- **Escalabilidad Horizontal:** La API es "stateless" (sin estado) y puede ser replicada.
- **Escalabilidad de Base de Datos:** PostgreSQL con pool de conexiones (SQLx) optimizado para alta concurrencia.
