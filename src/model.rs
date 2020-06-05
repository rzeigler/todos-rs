use futures::future::{ready, Future, FutureExt};
use rusqlite::{params, Connection, Result, NO_PARAMS};
use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::path::Path;
use std::sync::Arc;
use tokio::task;

#[derive(Serialize, Deserialize, Clone)]
pub struct Todo {
    id: u32,
    name: String,
    completed: bool,
}

impl Todo {
    pub fn new(id: u32, name: &str, completed: bool) -> Todo {
        Todo {
            id,
            name: name.to_owned(),
            completed,
        }
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn get_name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn get_completed(&self) -> bool {
        self.completed
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EphemeralTodo {
    name: String,
    completed: bool,
}

impl EphemeralTodo {
    pub fn get_name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn get_completed(&self) -> bool {
        self.completed
    }
}

#[derive(Clone)]
pub struct SqliteDb {
    path: Arc<OsString>,
}

impl SqliteDb {
    pub fn new(path: &str) -> SqliteDb {
        SqliteDb {
            path: Arc::new(OsString::from(path.to_owned())),
        }
    }

    pub fn get_connection(&self) -> impl Future<Output = Result<SqliteConn>> {
        let result = Ok(SqliteConn {
            path: self.path.clone(),
        });
        ready(result)
    }
}

pub struct SqliteConn {
    path: Arc<OsString>,
}

static SELECT_TODOS: &str = "SELECT id, name, completed FROM todos";
static INSERT_TODO: &str = "INSERT INTO todos(name, completed) VALUES(?, ?)";
static SELECT_LAST_ID: &str = "SELECT last_insert_rowid()";
static UPDATE_TODO: &str = "UPDATE todos set name=?, completed=? WHERE id=?";
static SELECT_TODO: &str = "SELECT id, name, completed FROM todos WHERE id  = ? LIMIT 1";

impl SqliteConn {
    pub fn list_todos(&self) -> impl Future<Output = Result<Vec<Todo>>> {
        let path = self.path.clone();
        let join = task::spawn_blocking(move || {
            let conn = Connection::open(Path::new(path.as_os_str()))?;
            let mut stmt = conn.prepare(SELECT_TODOS)?;
            let and_rows = stmt.query_and_then(NO_PARAMS, |row| {
                let id: u32 = row.get(0)?;
                let name = row.get(1)?;
                let completed = row.get(2)?;
                Ok(Todo {
                    id,
                    name,
                    completed,
                })
            })?;
            and_rows.collect()
        });
        join.map(|r| r.unwrap()) // Couldn't join... its a toy system
    }

    pub fn create_todo(self, todo: &EphemeralTodo) -> impl Future<Output = Result<u32>> {
        let path = self.path;
        let local = todo.clone();
        let join = task::spawn_blocking(move || {
            let mut conn = Connection::open(Path::new(path.as_os_str()))?;
            let tx = conn.transaction()?;
            tx.execute(INSERT_TODO, params![local.name, local.completed])?;
            let id: u32 = tx.query_row_and_then(SELECT_LAST_ID, NO_PARAMS, |row| row.get(0))?;
            tx.commit()?;
            Ok(id)
        });
        join.map(|r| r.unwrap())
    }

    pub fn update_todo(self, todo: &Todo) -> impl Future<Output = Result<bool>> {
        let path = self.path;
        let local = todo.clone();
        let join = task::spawn_blocking(move || {
            let conn = Connection::open(Path::new(path.as_os_str()))?;
            let result =
                conn.execute(UPDATE_TODO, params![local.name, local.completed, local.id])?;
            Ok(result > 0)
        });
        join.map(|r| r.unwrap())
    }

    pub fn get_todo(self, id: u32) -> impl Future<Output = Result<Option<Todo>>> {
        let path = self.path;
        let join = task::spawn_blocking(move || {
            let conn = Connection::open(Path::new(path.as_os_str()))?;
            let mut stmt = conn.prepare(SELECT_TODO)?;
            let mut result = stmt.query_and_then(params![id], |row| {
                let id: u32 = row.get(0)?;
                let name = row.get(1)?;
                let completed = row.get(2)?;
                Ok(Todo {
                    id,
                    name,
                    completed,
                })
            })?;
            result.next().transpose()
        });
        join.map(|r| r.unwrap())
    }
}
