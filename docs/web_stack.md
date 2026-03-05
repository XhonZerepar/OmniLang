# OmniLang Web Stack - Full-Stack Development Guide

This document describes the comprehensive web development capabilities added to OmniLang, enabling full-stack development with a single language.

## Table of Contents

1. [Backend Essentials](#backend-essentials)
2. [Frontend Essentials](#frontend-essentials)
3. [Full-Stack Glue](#full-stack-glue)
4. [Tooling and Compiler Enhancements](#tooling-and-compiler-enhancements)
5. [Examples](#examples)

---

## Backend Essentials

### HTTP Server Module

OmniLang includes a built-in HTTP server with async/await support:

```omni
import std::web::server::{server, Server, Router};

fn main(args: [String]) -> Int {
    let app = server()
        .with_port(8080)
        .use(middleware::cors())
        .use(middleware::logging())
        .get("/api/users", get_users)
        .post("/api/users", create_user);
    
    // app.listen(8080);
    0
}
```

**Features:**
- Async/await for concurrent request handling
- Middleware support (CORS, logging, auth, rate limiting)
- Declarative routing
- Integration with ownership model for safe concurrency

### Routing and Handlers

```omni
// Simple route handler
fn get_tasks(req: Request) -> Response {
    Response::new()
        .with_status(HttpStatus::ok())
        .with_json(tasks)
}

// Route with parameters
.post("/api/tasks/:id/complete", complete_task)

// Query parameters
let task_id = req.query_params.get("id");
```

### Data Persistence

SQLite integration via FFI:

```omni
import std::db::{Database, Store};

// Open database
let db = Database::open("app.db")?;

// Execute queries
db.execute("CREATE TABLE users (...)")?;
let result = db.query("SELECT * FROM users")?;

// Key-value store
let store = Store::open("app.db")?;
store.set("user:1", json_data)?;
let data = store.get("user:1");
```

### API Serialization

Auto-JSON via traits:

```omni
#[derive(Serialize, Deserialize)]
struct Task {
    id: Int,
    title: String,
    completed: Bool,
}

// Automatic JSON conversion
let json = task.to_json();
let task = Task::from_json(json_str)?;
```

### Auth/Security Primitives

JWT and session handling:

```omni
import std::auth::{jwt, session, password};

// Create JWT token
let token = jwt::builder()
    .secret("secret-key")
    .subject(user_id)
    .expires_in(3600)
    .build();

// Verify token
let payload = jwt::verify(token, "secret-key")?;

// Password hashing
let hash = password::hash("user_password");
let valid = password::verify("user_password", hash);
```

---

## Frontend Essentials

### WASM Compilation

Compile OmniLang to WebAssembly:

```bash
omc build --target=wasm frontend.omni
```

This produces:
- `frontend.wasm` - WebAssembly binary
- `frontend.js` - JavaScript glue code

### DOM Manipulation

Safe, reactive DOM bindings:

```omni
import std::web::dom::{dom, Element};

// Get elements
let elem = dom::get("my-id");
let items = dom::query_all(".items");

// Manipulate
elem.set_text("Hello World");
elem.set_html("<h1>Title</h1>");
elem.set_style("color", "red");
elem.add_class("active");
elem.remove_class("hidden");

// Events
elem.on("click", || {
    println("Clicked!");
});
```

### Animation/Motion Library

Declarative animations:

```omni
import std::ui::motion::{motion, AnimationConfig, Easing};

// Simple fade
motion::fade_in(&element);
motion::fade_out(&element);

// Custom tween
motion::tween(&elem,
    HashMap::from([("opacity".to_string(), "0")]),
    HashMap::from([("opacity".to_string(), "1")]),
    AnimationConfig::new()
        .duration(300)
        .easing(Easing::EaseInOut)
);

// Wheel spin (Lizard Mode)
motion::wheel_spin(&wheel, 5);
```

### Reactive State Management

Signals and effects:

```omni
import std::ui::reactive::{signal, effect};

// Create reactive state
let count = signal(0);

// Subscribe to changes
count.subscribe(|val| {
    println("Count changed: " + val);
});

// Update triggers notification
count.set(42);
```

### Event Handling

Async event handlers:

```omni
// Button click with API call
async fn handle_click() -> () {
    let data = await fetch("/api/data").text()?;
    element.set_text(data);
}

elem.on("click", handle_click);
```

---

## Full-Stack Glue

### HTTP Client

For frontend-backend communication:

```omni
import std::web::client::{client, Client};

// Create client
let client = client::new()
    .with_base_url("http://localhost:8080")
    .with_auth(token);

// Make requests
let response = await client.get("/api/tasks").send()?;
let tasks = await client.get("/api/tasks").json::<Vec<Task>>();
let result = await client.post("/api/tasks")
    .json(&new_task)
    .send()?;
```

### Shared Types

Define once, use everywhere:

```omni
// shared.omni
#[derive(Serialize, Deserialize)]
struct Task {
    id: Int,
    title: String,
    completed: Bool,
}
```

Import in both backend and frontend files.

### WebSocket/SSE for Realtime

Real-time communication:

```omni
import std::web::realtime::{WebSocket, EventSource};

// WebSocket
let ws = WebSocket::connect("ws://localhost:8080/ws")
    .on_message(|data| {
        handle_update(data);
    });

// Server-Sent Events
let es = EventSource::new("http://localhost:8080/events")
    .on_event("notification", |data| {
        show_notification(data);
    });
```

### AI/ML Integration

Client-side AI for natural language processing:

```omni
import std::ai::oracle::{Oracle, TaskSpec};

// Create task from natural language
let oracle = Oracle::new();
let task = oracle.create_task("Finish quarterly report by Friday")?;
```

---

## Tooling and Compiler Enhancements

### Single-Command Full-Stack Build

Build both backend and frontend:

```bash
omc build --fullstack --backend server.omni --frontend frontend.omni
```

This produces:
- `dist/server` - Native executable
- `dist/frontend.wasm` - WebAssembly binary
- `dist/frontend.js` - JavaScript glue
- `dist/index.html` - HTML host page

### Macros for Boilerplate Reduction

```omni
#[web_app]
struct MyApp {
    // Automatically generates main(), initialization, etc.
}
```

### Testing Framework

```omni
import std::test::web;

test "dashboard loads" {
    browser.open("/");
    assert_dom("#energy-bar");
    assert_text("#status", "Ready");
}
```

### Deployment Helpers

```omni
// Deploy to Netlify
deploy.to_netlify();

// Deploy to Vercel  
deploy.to_vercel();

// Custom deployment
deploy.to("my-server.com", 22, "/path");
```

---

## Examples

### Backend Example

See `examples/web_backend.omni` for a complete backend example with:
- REST API endpoints
- Database integration
- Authentication (JWT)
- AI-powered task creation

### Frontend Example

See `examples/web_frontend.omni` for a complete frontend example with:
- Reactive state management
- DOM manipulation
- Animations (energy bars, wheel spin)
- Real-time updates (WebSocket/SSE)
- API integration

### Full-Stack Example

Combine both files for a complete full-stack application:

```bash
# Build
omc build --fullstack --backend examples/web_backend.omni --frontend examples/web_frontend.omni

# Run
./dist/server &
# Open browser to http://localhost:8080
```

---

## Standard Library Modules

| Module | Description |
|--------|-------------|
| `std::web` | Core web types (Request, Response, JSON) |
| `std::web::server` | HTTP server and routing |
| `std::web::client` | HTTP client |
| `std::web::dom` | DOM manipulation (WASM) |
| `std::web::realtime` | WebSocket and SSE |
| `std::async` | Async runtime and primitives |
| `std::db` | Database (SQLite) |
| `std::auth` | Authentication (JWT, sessions) |
| `std::ui` | UI and animations |
| `std::ai` | AI/ML integration |

---

## Roadmap

- [ ] Complete WASM backend implementation
- [ ] Add more database drivers (PostgreSQL, MySQL)
- [ ] Implement WebSocket server support
- [ ] Add server-side rendering
- [ ] Complete testing framework
- [ ] Deployment automation

---

## Contributing

See CONTRIBUTING.md for guidelines.
