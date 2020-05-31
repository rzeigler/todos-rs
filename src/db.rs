use rusqlite::{
    params, Connection, DropBehavior, Result, Row, ToSql, Transaction, TransactionBehavior,
    NO_PARAMS,
};
use std::convert::From;
use std::ffi::OsString;
use std::path::Path;
use std::sync::Arc;

pub struct Db {
    path: Arc<OsString>,
}

impl Db {
    pub fn new(path: &str) -> Db {
        Db {
            path: Arc::new(OsString::from(path)),
        }
    }

    pub fn get_handle(&self) -> Result<Handle> {
        let conn = Connection::open(Path::new(self.path.as_os_str()))?;
        Ok(Handle { conn })
    }

    pub fn new_initialized(path: &str) -> Result<Db> {
        let db = Db::new(path);
        let handle = db.get_handle()?;
        lazy_initialize(&handle.conn)?;
        Ok(db)
    }
}

fn lazy_initialize(conn: &Connection) -> Result<()> {
    if needs_initialize(conn)? {
        initialize(conn)?;
    }
    Ok(())
}

static COUNT_TODO_TABLE: &str =
    "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='todo'";

fn needs_initialize(conn: &Connection) -> Result<bool> {
    let count: u32 = conn.query_row(COUNT_TODO_TABLE, NO_PARAMS, |row| row.get(0))?;
    Ok(count == 0)
}

static CREATE_TODO_TABLE: &str = r#"
CREATE TABLE todo (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    completed INT DEFAULT 0
)
"#;

fn initialize(conn: &Connection) -> Result<()> {
    conn.execute(CREATE_TODO_TABLE, NO_PARAMS)?;
    Ok(())
}

#[derive(Debug)]
pub struct Todo {
    pub id: u32,
    pub name: String,
    pub completed: bool,
}

impl Todo {
    pub fn from_row(row: &Row) -> Result<Todo> {
        let completed: u32 = row.get("completed")?;
        Ok(Todo {
            id: row.get("id")?,
            name: row.get("name")?,
            completed: completed != 0u32,
        })
    }
}

pub struct TransientTodo {
    pub name: String,
    pub completed: bool,
}

impl TransientTodo {
    pub fn to_params(&self) -> Vec<&dyn ToSql> {
        vec![&self.name, &self.completed]
    }

    pub fn from_todo(todo: &Todo) -> TransientTodo {
        TransientTodo {
            name: todo.name.clone(),
            completed: todo.completed,
        }
    }
}

pub enum CompleteState {
    All,
    Complete,
    Incomplete,
}

static ALL_TODO_QUERY: &str = "SELECT id, name, completed FROM todo";
static COMPLETE_TODO_QUERY: &str = "SELECT id, name, completed FROM todo WHERE completed <> 0";
static INCOMPLETE_TODO_QUERY: &str = "SELECT id, name, completed FROM todo WHERE completed = 0";

fn select_query(state: CompleteState) -> &'static str {
    match state {
        CompleteState::All => ALL_TODO_QUERY,
        CompleteState::Incomplete => INCOMPLETE_TODO_QUERY,
        CompleteState::Complete => COMPLETE_TODO_QUERY,
    }
}

static INSERT_TODO_QUERY: &str = "INSERT INTO todo(name, completed) VALUES (?, ?)";
static LAST_ROW_ID: &str = "SELECT last_insert_rowid()";

static SELECT_TODO_QUERY: &str = "SELECT id, name, completed FROM todo WHERE id = ? LIMIT 1";
static UPDATE_TODO_QUERY: &str = "UPDATE todo SET name = ?, completed = ? WHERE id = ?";

pub struct Handle {
    conn: Connection,
}

impl Handle {
    pub fn list_todos(&self, state: CompleteState) -> Result<Vec<Todo>> {
        let q = select_query(state);
        let mut stmt = self.conn.prepare(q)?;
        let result = stmt.query_and_then(NO_PARAMS, Todo::from_row)?;
        result.collect()
    }

    pub fn insert_todo(&mut self, todo: &TransientTodo) -> Result<u32> {
        let tx = {
            let mut tx = Transaction::new(&mut self.conn, TransactionBehavior::Deferred)?;
            tx.set_drop_behavior(DropBehavior::Commit);
            tx
        };
        tx.execute(INSERT_TODO_QUERY, todo.to_params())?;
        Ok(tx.query_row(LAST_ROW_ID, NO_PARAMS, |row| row.get(0))?)
    }

    pub fn get_todo(&self, id: u32) -> Result<Option<Todo>> {
        let mut stmt = self.conn.prepare(SELECT_TODO_QUERY)?;
        let mut results = stmt.query_and_then(params![id], Todo::from_row)?;
        results.next().transpose()
    }

    pub fn update_todo(&self, todo: &Todo) -> Result<()> {
        self.conn.execute(
            UPDATE_TODO_QUERY,
            params![todo.name, todo.completed, todo.id],
        )?;
        Ok(())
    }
}
