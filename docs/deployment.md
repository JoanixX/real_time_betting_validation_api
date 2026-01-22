# Guía de Despliegue en la Nube 🚀

Este proyecto está diseñado para ser compatible con los principales proveedores de la nube. A continuación se detallan los pasos para cada uno.

## 1. Render & AWS/Azure (Contenerizado)

La forma más sencilla de desplegar la API y el Frontend como una sola unidad (container).

1.  **Plataforma:** Crea un "Web Service" (Render) o "App Service" (Azure) / "App Runner" (AWS).
2.  **Origen:** Conecta tu repositorio de GitHub.
3.  **Entorno de ejecución (Runtime):** Selecciona **Docker**.
4.  **Variables de Entorno:**
    - `DATABASE_URL`: Tu cadena de conexión de Postgres.
    - `APP_ENVIRONMENT`: `production`
5.  **Puerto Expuesto:** `8000`.

El `Dockerfile` automáticamente:

- Compilará la API de Rust.
- Ejecutará las migraciones al iniciar (si se configura).

---

## 2. Vercel (Solo Frontend)

Si deseas alojar el frontend en Vercel para un mejor rendimiento global, y la API en otro lugar (ej. Render).

1.  **Panel de Vercel:** "New Project" -> Selecciona el Repo.
2.  **Ajustes de Proyecto:** Configura el "Root Directory" como `frontend`.
3.  **Framework Preset:** Other / None.
4.  **Directorio de Salida (Output Directory):** `public`
5.  **Variables de Entorno:**
    - Añade `API_URL`: La URL de tu backend (ej. `https://tu-api.onrender.com`).

_Aviso: El archivo `vercel.json` incluido maneja las redirecciones (rewrites) necesarias._

---

## 3. Azure & AWS (Avanzado)

Para producción de alta concurrencia:

### AWS

- **Base de Datos:** RDS (PostgreSQL).
- **Servicio:** ECS (Elastic Container Service) con Fargate.
- **CI/CD:** Usa GitHub Actions para construir la imagen Docker y subirla a ECR.

### Azure

- **Base de Datos:** Azure Database for PostgreSQL.
- **Servicio:** Azure Container Apps (Escala automáticamente según el tráfico HTTP).

---

## Pruebas Locales

1.  Asegúrate de tener Postgres funcionando (`cd infrastructure && docker-compose up -d`).
2.  Ejecuta: `cd backend && cargo run`
3.  Abre `http://localhost:8000` en tu navegador (si el backend sirve los estáticos) o usa un servidor local para `frontend/public`.
