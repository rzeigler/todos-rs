#![warn(clippy::all)]
#![allow(dead_code)]

mod model;

use model::*;
use warp::Filter;

#[derive(Debug)]
struct Explode(rusqlite::Error);

impl warp::reject::Reject for Explode {}

mod handler {
    use super::model::*;
    use super::Explode;
    use warp::http::StatusCode;
    use warp::reject;
    use warp::reject::Rejection;
    use warp::reply::json;
    pub async fn list_todos(conn: SqliteConn) -> Result<impl warp::Reply, Rejection> {
        let todos = conn
            .list_todos()
            .await
            .map_err(|e| reject::custom(Explode(e)))?;
        Ok(json(&todos))
    }

    pub async fn create_todo(
        todo: EphemeralTodo,
        conn: SqliteConn,
    ) -> Result<impl warp::Reply, Rejection> {
        let id = conn
            .create_todo(&todo)
            .await
            .map_err(|e| reject::custom(Explode(e)))?;
        Ok(json(&Todo::new(id, todo.get_name(), todo.get_completed())))
    }

    pub async fn update_todo(
        id: u32,
        todo: EphemeralTodo,
        conn: SqliteConn,
    ) -> Result<Box<dyn warp::Reply>, Rejection> {
        let todo = Todo::new(id, todo.get_name(), todo.get_completed());
        let result = conn
            .update_todo(&todo)
            .await
            .map_err(|e| reject::custom(Explode(e)))?;
        if result {
            Ok(Box::new(json(&todo)))
        } else {
            Ok(Box::new(warp::reply::with_status(
                warp::reply(),
                StatusCode::NOT_FOUND,
            )))
        }
    }

    pub async fn get_todo(id: u32, conn: SqliteConn) -> Result<Box<dyn warp::Reply>, Rejection> {
        if let Some(todo) = conn
            .get_todo(id)
            .await
            .map_err(|e| reject::custom(Explode(e)))?
        {
            Ok(Box::new(json(&todo)))
        } else {
            Ok(Box::new(warp::reply::with_status(
                warp::reply(),
                StatusCode::NOT_FOUND,
            )))
        }
    }
}

mod filter {
    use super::handler;
    use super::model::*;
    use super::Explode;
    use futures::FutureExt;
    use warp::reject;
    use warp::Filter;

    fn conn(db: SqliteDb) -> impl Filter<Extract = (SqliteConn,), Error = warp::Rejection> + Clone {
        warp::any().and_then(move || {
            db.get_connection()
                .map(|r| r.map_err(|e| reject::custom(Explode(e))))
        })
    }

    pub fn list_todos(
        db: SqliteDb,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("todos")
            .and(warp::get())
            .and(conn(db))
            .and_then(handler::list_todos)
    }

    pub fn create_todo(
        db: SqliteDb,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("todos")
            .and(warp::post())
            .and(warp::body::json())
            .and(conn(db))
            .and_then(handler::create_todo)
    }

    pub fn update_todo(
        db: SqliteDb,
    ) -> impl Filter<Extract = (Box<dyn warp::Reply>,), Error = warp::Rejection> + Clone {
        warp::path!("todos" / u32)
            .and(warp::put())
            .and(warp::body::json())
            .and(conn(db))
            .and_then(handler::update_todo)
    }

    pub fn get_todo(
        db: SqliteDb,
    ) -> impl Filter<Extract = (Box<dyn warp::Reply>,), Error = warp::Rejection> + Clone {
        warp::path!("todos" / u32)
            .and(warp::get())
            .and(conn(db))
            .and_then(handler::get_todo)
    }

    pub fn api(
        db: SqliteDb,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        list_todos(db.clone())
            .or(create_todo(db.clone()))
            .or(get_todo(db.clone()))
            .or(update_todo(db))
    }
}

#[tokio::main]
async fn main() {
    let db = SqliteDb::new("todos.sqlite");

    let api = filter::api(db);

    // View access logs by setting `RUST_LOG=todos`.
    let routes = api.with(warp::log("todos"));
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
