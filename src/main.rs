mod db;

use db::*;

fn main() {
    let db = Db::new_initialized("todo.sqlite").unwrap();
    let mut handle = db.get_handle().unwrap();
    let todos = handle.list_todos(CompleteState::All).unwrap();
    println!("Todos: {:?}", todos);
    handle
        .insert_todo(&TransientTodo {
            name: "More".to_string(),
            completed: false,
        })
        .unwrap();
    handle
        .insert_todo(&TransientTodo {
            name: "More 2".to_string(),
            completed: true,
        })
        .unwrap();
    let todos = handle.list_todos(CompleteState::All).unwrap();
    println!("Todos: {:?}", todos);
    let todo = handle.get_todo(100).unwrap();
    println!("Todo 1: {:?}", todo);
}
