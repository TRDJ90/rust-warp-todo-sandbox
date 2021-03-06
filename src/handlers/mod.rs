/// These are out API handlers, the end of each filter.chain.
/// Notice how thanks to using 'Filter::and', we can define a function
/// with the exact arguments we'd expect from each filter in the chain.
/// No tuples are needed, it's auto flattened for the functions.
use super::models::{Db, ListOptions, Todo};
use std::convert::Infallible;
use warp::http::StatusCode;

pub async fn list_todos(opts: ListOptions, db: Db) -> Result<impl warp::Reply, Infallible> {
    // Just return a JSON array of todos, applying the limit and offset.
    let todos = db.lock().await;
    let todos: Vec<Todo> = todos
        .clone()
        .into_iter()
        .skip(opts.offset.unwrap_or(0))
        .take(opts.limit.unwrap_or(std::usize::MAX))
        .collect();
    Ok(warp::reply::json(&todos))
}

pub async fn create_todo(create: Todo, db: Db) -> Result<impl warp::Reply, Infallible> {
    log::debug!("create_todo: {:?}", create);

    let mut vec = db.lock().await;

    for todo in vec.iter() {
        if todo.id == create.id {
            log::debug!("   -> id already exists: {}", create.id);
            // Todo with id already exists, return '400 BadRequest'.
            return Ok(StatusCode::BAD_REQUEST);
        }
    }
    vec.push(create);

    Ok(StatusCode::CREATED)
}

pub async fn update_todo(id: u64, update: Todo, db: Db) -> Result<impl warp::Reply, Infallible> {
    log::debug!("update_todo: id={}, todo={:?}", id, update);
    let mut vec = db.lock().await;

    // Look for the specified Todo...
    for todo in vec.iter_mut() {
        if todo.id == id {
            *todo = update;
            return Ok(StatusCode::OK);
        }
    }

    log::debug!("    -> todo id not found!");

    // If the for loop didn't return OK, then the ID doesn't exist...
    Ok(StatusCode::NOT_FOUND)
}

pub async fn delete_todo(id: u64, db: Db) -> Result<impl warp::Reply, Infallible> {
    log::debug!("delete_todo: id={}", id);

    let mut vec = db.lock().await;

    let len = vec.len();
    vec.retain(|todo| {
        // Retain all Todos that aren't this id...
        // In other words, remove all that *are* this id...
        todo.id != id
    });

    // If the vec is smaller, we found and deleted a Todo
    let deleted = vec.len() != len;

    if deleted {
        // respond with a '204 No Content', which means successful,
        // yet no body expected...
        Ok(StatusCode::NO_CONTENT)
    } else {
        log::debug!("   -> todo id not found!");
        Ok(StatusCode::NOT_FOUND)
    }
}
