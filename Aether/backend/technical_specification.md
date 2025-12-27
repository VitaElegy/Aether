# Aether Blog Platform - Technical Specification

## 1. Project Overview
Aether is a modern, high-performance blogging platform designed for "Digital Consciousness." It features a decoupled architecture with a Rust (Axum) backend and a Vue 3 frontend.

## 2. Technology Stack

### Backend
- **Language**: Rust (2021 Edition)
- **Framework**: Axum 0.7
- **Database ORM**: SeaORM (Async)
- **Database Support**: PostgreSQL (Production) / SQLite (Dev)
- **Serialization**: Serde / Serde JSON
- **Authentication**: JWT (Json Web Tokens)
- **Async Runtime**: Tokio

### Frontend
- **Framework**: Vue 3 (Composition API)
- **Build Tool**: Vite
- **Styling**: TailwindCSS (Utility-first)
- **State Management**: Pinia
- **Router**: Vue Router 4
- **HTTP Client**: Axios

## 3. Core Features

### Content Management
- **CRUD Operations**: Complete lifecycle management for Articles.
- **Versioning**: Automatic version control for every edit, allowing diff views.
- **Markdown Support**: Native rendering of Markdown content.
- **Privacy Controls**: Visibility levels (Public, Internal, Private).

### User System
- **Role-Based Access**: Bitmask-based permission system.
- **Profile**: Customizable user profiles with avatars (DiceBear integration).

## 4. Search Algorithm (Weighted Relevance Ranking)

To ensure high-quality search results even with massive datasets, Aether implements a **Weighted Relevance Ranking Algorithm** directly within the database query layer.

### Logic
The relevance score is calculated dynamically during the SQL query execution:

$$ Score = (TitleMatch \times 10) + (TagMatch \times 5) + (BodyMatch \times 1) $$

*   **Title Match (10 pts)**: High priority. If the keyword appears in the title, it is likely the user's target.
*   **Tag Match (5 pts)**: Medium priority. Indicates the article belongs to the relevant topic.
*   **Body Match (1 pts)**: Low priority. Mentions in the text are relevant but less significant than structural matches.

### Implementation Details
- **SQL Injection**: We use standard SQL `CASE` expressions via SeaORM's `Expr::cust_with_values` to ensure compatibility with both SQLite and PostgreSQL.
- **Sorting**: Results are ordered by `Score DESC`, then `CreatedAt DESC` to show the most relevant and recent content first.

## 5. Directory Structure
```
Aether/
├── backend/            # Rust Core
│   ├── src/
│   │   ├── domain/     # Business Logic & Models
│   │   ├── infrastructure/ # DB & Auth Adapters
│   │   └── interface/  # API Handlers
├── frontend/           # Vue 3 App
│   ├── src/
│   │   ├── components/ # Reusable UI (SearchBar, etc)
│   │   ├── views/      # Page Views (Home, Search, Editor)
│   │   └── stores/     # State Management
```
