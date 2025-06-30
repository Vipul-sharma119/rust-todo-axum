use axum::{
    routing::{get, post, put, delete},
    extract::{State, Path},
    response::{Html, IntoResponse},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{sync::{Arc, Mutex}, net::SocketAddr};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Todo {
    id: String,
    title: String,
    completed: bool,
}

type TodoList = Arc<Mutex<Vec<Todo>>>;

#[tokio::main]
async fn main() {
    let todos: TodoList = Arc::new(Mutex::new(vec![]));

    let app = Router::new()
        .route("/", get(serve_html))
        .route("/todos", get(get_todos).post(add_todo))
        .route("/todos/:id", put(mark_done).delete(delete_todo))
        .with_state(todos);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running on http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn serve_html() -> impl IntoResponse {
    Html(
        r#"
<!DOCTYPE html>
<html>
<head>
  <title>Rust To-Do List</title>
  <style>
    body { font-family: Arial; padding: 20px; }
    input { padding: 6px; }
    button { margin-left: 5px; }
    li.done { text-decoration: line-through; color: gray; }
  </style>
</head>
<body>
  <h1>📝 Rust To-Do List</h1>
  <input id="todo-input" type="text" placeholder="New task..." />
  <button onclick="addTodo()">Add</button>
  <ul id="todo-list"></ul>

  <script>
    const apiUrl = "/todos";

    async function fetchTodos() {
      const res = await fetch(apiUrl);
      const todos = await res.json();
      const list = document.getElementById("todo-list");
      list.innerHTML = "";

      todos.forEach(todo => {
        const li = document.createElement("li");
        li.textContent = todo.title;
        if (todo.completed) li.classList.add("done");

        const doneBtn = document.createElement("button");
        doneBtn.textContent = "✓";
        doneBtn.onclick = () => markDone(todo.id);

        const delBtn = document.createElement("button");
        delBtn.textContent = "🗑️";
        delBtn.onclick = () => deleteTodo(todo.id);

        li.appendChild(doneBtn);
        li.appendChild(delBtn);
        list.appendChild(li);
      });
    }

    async function addTodo() {
      const input = document.getElementById("todo-input");
      const title = input.value.trim();
      if (!title) return;

      await fetch(apiUrl, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ title })
      });

      input.value = "";
      fetchTodos();
    }

    async function markDone(id) {
      await fetch(`${apiUrl}/${id}`, { method: "PUT" });
      fetchTodos();
    }

    async function deleteTodo(id) {
      await fetch(`${apiUrl}/${id}`, { method: "DELETE" });
      fetchTodos();
    }

    fetchTodos();
  </script>
</body>
</html>
"#,
    )
}

async fn get_todos(State(state): State<TodoList>) -> Json<Vec<Todo>> {
    let todos = state.lock().unwrap();
    Json(todos.clone())
}

#[derive(Deserialize)]
struct NewTodo {
    title: String,
}

async fn add_todo(
    State(state): State<TodoList>,
    Json(payload): Json<NewTodo>,
) -> Json<Todo> {
    let new_todo = Todo {
        id: Uuid::new_v4().to_string(),
        title: payload.title,
        completed: false,
    };

    let mut todos = state.lock().unwrap();
    todos.push(new_todo.clone());

    Json(new_todo)
}

async fn mark_done(
    Path(id): Path<String>,
    State(state): State<TodoList>,
) -> Json<&'static str> {
    let mut todos = state.lock().unwrap();
    if let Some(todo) = todos.iter_mut().find(|t| t.id == id) {
        todo.completed = true;
        Json("Marked as done")
    } else {
        Json("Todo not found")
    }
}

async fn delete_todo(
    Path(id): Path<String>,
    State(state): State<TodoList>,
) -> Json<&'static str> {
    let mut todos = state.lock().unwrap();
    let original_len = todos.len();
    todos.retain(|t| t.id != id);
    if todos.len() < original_len {
        Json("Deleted")
    } else {
        Json("Todo not found")
    }
}
