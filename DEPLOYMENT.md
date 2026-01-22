# Cloud Deployment Guide 🚀

This project is designed to be compatible with major cloud providers. Below are the steps for each.

## 1. Render & AWS/Azure (Containerized)

The easiest way to deploy the API + Frontend as a single unit.

1.  **Platform:** Create a "Web Service" (Render) or "App Service" (Azure) / "App Runner" (AWS).
2.  **Source:** Connect your GitHub Repository.
3.  **Runtime:** Select **Docker**.
4.  **Environment Variables:**
    - `DATABASE_URL`: Your Postgres connection string.
    - `APP_ENVIRONMENT`: `production`
5.  **Expose Port:** `8000`.

The `Dockerfile` will automatically:

- Build the Rust API.
- Include the `public` folder.
- Run migrations on startup.

---

## 2. Vercel (Frontend Only)

If you want to host the frontend on Vercel for better global performance, and the API elsewhere (e.g., Render).

1.  **Vercel Dashboard:** "New Project" -> Select Repo.
2.  **Framework Preset:** Other / None.
3.  **Build Command:** (Empty)
4.  **Output Directory:** `public`
5.  **Environment Variables:**
    - Add `API_URL`: The URL of your backend (e.g., `https://your-api.onrender.com`).

_Notice: The `vercel.json` included handles the necessary rewrites._

---

## 3. Azure & AWS (Advanced)

For high-concurrency production:

### AWS

- **Database:** RDS (PostgreSQL).
- **Service:** ECS (Elastic Container Service) with Fargate.
- **CI/CD:** Use GitHub Actions to build the Docker image and push to ECR.

### Azure

- **Database:** Azure Database for PostgreSQL.
- **Service:** Azure Container Apps (Scales automatically based on HTTP traffic).

---

## Testing Localmente

1.  Asegúrate de tener Postgres corriendo (`docker-compose up -d`).
2.  Ejecuta: `cargo run`
3.  Abre `http://localhost:8000` en tu navegador.
