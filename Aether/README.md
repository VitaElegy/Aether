# Aether - Digital Consciousness

Aether is a minimalist, block-based personal knowledge management and blogging platform. It allows you to create, version, and visualize your thoughts as digital "transmissions".

## Features

-   **Deep Content Versioning**: Every save creates a new version. Diff any two versions to see how your thoughts evolved.
-   **Block-Based Editor**: Markdown-first, but supports dynamic blocks (Code, Video, etc.).
-   **Personal & Social**:
    -   **Profile System**: Customizable profiles with avatars, bios, and social links.
    -   **Settings**: Update your digital persona.
    -   **Reading Mode**: Distraction-free reading experience with auto-generated Table of Contents.
-   **Tech Stack**:
    -   **Backend**: Rust (Axum, SeaORM, Tokio) - High performance, type-safe.
    -   **Frontend**: Vue 3 (Composition API, Pinia, TailwindCSS) - Reactive and beautiful.
    -   **Database**: PostgreSQL (Production) / SQLite (Dev).

## Getting Started

### Prerequisites

-   Rust (latest stable)
-   Node.js (v18+)
-   PostgreSQL (optional, defaults to SQLite for dev)

### Backend Setup

```bash
cd backend
# Create .env file
echo "DATABASE_URL=sqlite://aether.db?mode=rwc" > .env
echo "JWT_SECRET=your_secret_key" >> .env

# Run server (Auto-migrations included)
cargo run
```

Server will start at `http://localhost:3000`.

### Frontend Setup

```bash
cd frontend
npm install
npm run dev
```

Frontend will start at `http://localhost:5173`.

## Architecture

See [ARCHITECTURE.md](doc/ARCHITECTURE.md) for a high-level overview.
See [TECHNICAL_REFERENCE.md](doc/TECHNICAL_REFERENCE.md) for implementation details.

## License

MIT

