# Architecture Decision Record (ADR) - High Concurrency API

## Contexto

Este servicio está diseñado para manejar un volumen masivo de peticiones concurrentes con latencia mínima. En Rust, la elección de la estructura del código impacta no solo en la mantenibilidad sino también en la facilidad del compilador para realizar optimizaciones de "inlining" y reducir el overhead.

## Decisión: Arquitectura en Capas (Layered)

Hemos optado por una **Arquitectura en Capas** en lugar de una Hexagonal estricta.

### Razón Técnica

1. **Performance First**: Menos niveles de indirección (traits dinámicos) permiten que el compilador de Rust optimice mejor la ruta crítica (`hot path`).
2. **Simplicidad**: Para un servicio cuyo core es transaccional y veloz, una estructura HANDLER -> SERVICE -> REPOSITORY es más directa y fácil de depurar bajo carga.
3. **Zero-Cost Abstractions**: Aprovechamos el sistema de tipos de Rust para garantizar seguridad sin pagar el precio de arquitecturas excesivamente abstractas.

## Consecuencias

- **Positivas**: Menor latencia, menor consumo de CPU por request, facilidad para escalar horizontalmente.
- **Negativas**: El dominio está un poco más acoplado a la infraestructura (SQLx), lo que requeriría más esfuerzo si se cambiara de motor de base de datos (poco probable en este tipo de servicios).

---

_Documento creado por el Staff Engineer para el equipo de Backend._
