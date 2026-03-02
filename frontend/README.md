# Frontend — Dashboard de Apuestas en Tiempo Real

Interfaz web construida con **Next.js 14** (App Router) que consume la API de alta concurrencia en Rust. Muestra odds en tiempo real, permite colocar apuestas y visualizar la actividad del motor de validación.

## 🛠️ Stack Tecnológico

- **Framework**: Next.js 14 (App Router, Server Components + Client Components)
- **Lenguaje**: TypeScript
- **Estado Global**: Zustand (stores granulares para odds, apuestas y auth)
- **Data Fetching**: TanStack Query (React Query) para datos REST
- **Tiempo Real**: WebSocket nativo con reconexión exponencial + heartbeat
- **Estilos**: Tailwind CSS + shadcn/ui
- **HTTP Client**: Axios con interceptores de auth

## 📂 Estructura

```
src/
├── app/                  # Rutas de Next.js (App Router)
│   ├── (auth)/           # Rutas de autenticación (login, registro)
│   ├── dashboard/        # Dashboard principal con odds en vivo
│   ├── layout.tsx        # Layout raíz con providers
│   └── page.tsx          # Página de inicio
├── components/           # Componentes reutilizables
│   ├── ui/               # Primitivos de shadcn/ui (Button, Card, Table, etc.)
│   ├── betting-slip.tsx  # Boleta de apuestas
│   ├── live-odds-row.tsx # Fila memoizada de odds en vivo
│   └── live-odds-table.tsx
├── hooks/                # Custom hooks
│   ├── use-live-odds.ts  # Selector granular de odds por partido
│   ├── use-place-bet.ts  # Mutación REST + pending en Zustand
│   ├── use-socket.ts     # Conexión WebSocket → stores
│   └── ...
├── lib/                  # Utilidades y clientes
│   ├── api.ts            # Cliente Axios preconfigurado
│   ├── socket.ts         # Cliente WebSocket con reconnect + heartbeat
│   └── mock-socket.ts   # Simulador de odds (sin backend)
├── store/                # Stores de Zustand
│   ├── auth-store.ts
│   ├── betting-store.ts
│   ├── odds-store.ts
│   └── selections-store.ts
└── types/                # Tipos TypeScript (espejo de domain/models.rs)
    └── domain.ts
```

## 🚀 Ejecución Local

```bash
# Instalar dependencias
npm install

# Iniciar servidor de desarrollo
npm run dev
```

Se abre en `http://localhost:3000`. El dashboard funciona con datos simulados (mock) sin necesidad del backend.

## 🏗️ Build de Producción

```bash
npm run build
npm start
```

El build utiliza `output: 'standalone'` para generar un contenedor autocontenido compatible con Docker.

## ⚙️ Variables de Entorno

Crear un archivo `.env` en la carpeta `frontend/`:

```env
NEXT_PUBLIC_API_URL=http://localhost:8000
NEXT_PUBLIC_WS_URL=ws://localhost:8000
```
