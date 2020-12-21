#[macro_use]
extern crate diesel;

use crate::schema::memos;
use actix_web::{get, post, error, web, App, Error, HttpRequest, HttpResponse, HttpServer, middleware};
use diesel::{
    prelude::*,
    r2d2::{self, ConnectionManager},
    sqlite::SqliteConnection,
};
use serde::{Deserialize, Serialize};
use std::{str, env, io};
use tera::{Context, Tera};
use chrono::Local;

pub mod models;
pub mod schema;

async fn greet() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body("Hello, world!"))
}

#[derive(Serialize, Deserialize)]
pub struct FormParams {
    content: String,
}

async fn form(
    pool: web::Data<r2d2::Pool<ConnectionManager<SqliteConnection>>>,
    tmpl: web::Data<Tera>,
) -> Result<HttpResponse, Error> {
    let mut ctx = Context::new();
    let conn = pool.get().expect("couldn't get db connection from pool");

    let memos = memos::table
        .filter(memos::del.eq(0))
        .order(memos::created_at.desc())//added
        .limit(5)
        .load::<crate::models::Memo>(&conn)
        .expect("Error loading cards");
    ctx.insert("memos", &memos);
    let view = tmpl
        .render("form.html", &ctx)
        .map_err(|e| error::ErrorInternalServerError(e))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(view))
}

#[get("/memo/{id}")]
async fn form_one(
    pool: web::Data<r2d2::Pool<ConnectionManager<SqliteConnection>>>,
    tmpl: web::Data<Tera>,
    info: web::Path<(i32,)>,
) -> Result<HttpResponse, Error> {
    let info = info.into_inner();//info.0,path.into_inner().0
    let mut ctx = Context::new();
    let conn = pool.get().expect("couldn't get db connection from pool");
    let memos = memos::table
        .filter(memos::del.eq(0))
        .filter(memos::id.eq(info.0))
        .limit(1)
        .load::<crate::models::Memo>(&conn)
        .expect("Error loading cards");
    ctx.insert("memos", &memos);
    let view = tmpl
        .render("form.html", &ctx)
        .map_err(|e| error::ErrorInternalServerError(e))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(view))
}

async fn memo_form(
    req: HttpRequest,
    pool: web::Data<r2d2::Pool<ConnectionManager<SqliteConnection>>>,
    params: web::Form<FormParams>,
    tmpl: web::Data<Tera>,
) -> Result<HttpResponse, Error> {
    println!("{:?}", req);
    let new_memo = crate::models::NewMemo {
        content: String::from(&params.content),
        created_at: Local::now().naive_local(),
        del: 0,
    };
    let conn = pool.get().expect("couldn't get db connection from pool");
    diesel::insert_into(memos::table)
        .values(&new_memo)
        .execute(&conn)
        .unwrap();
    let mut ctx = Context::new();
    let memos = memos::table
        .filter(memos::del.eq(0))
        .order(memos::created_at.desc())//added
        .limit(5)
        .load::<crate::models::Memo>(&conn)
        .expect("Error loading cards");
    ctx.insert("memos", &memos);
    let view = tmpl
        .render("form.html", &ctx)
        .map_err(|e| error::ErrorInternalServerError(e))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(view))
}

#[post("/search")]
async fn search(
    pool: web::Data<r2d2::Pool<ConnectionManager<SqliteConnection>>>,
    params: web::Form<FormParams>,
    tmpl: web::Data<Tera>,
) -> Result<HttpResponse, Error> {
    let mut ctx = Context::new();
    let conn = pool.get().expect("couldn't get db connection from pool");

    let pattern = format!("%{}%", String::from(&params.content));
    let memos = memos::table
        .filter(memos::content.like(pattern))
        .filter(memos::del.eq(0))
        .load::<crate::models::Memo>(&conn)
        .expect("Error loading cards");
    
    ctx.insert("memos", &memos);
    let view = tmpl
        .render("form.html", &ctx)
        .map_err(|e| error::ErrorInternalServerError(e))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(view))
}

#[post("/delete/{id}")]
async fn delete(
    pool: web::Data<r2d2::Pool<ConnectionManager<SqliteConnection>>>,
    tmpl: web::Data<Tera>,
    info: web::Path<(i32,)>,
) -> Result<HttpResponse, Error> {
    let info = info.into_inner();//info.0,path.into_inner().0
    let mut ctx = Context::new();
    let conn = pool.get().expect("couldn't get db connection from pool");

    let target = memos::table.filter(memos::id.eq(info.0));
    diesel::update(target)
        .set(memos::del.eq(1))
        .execute(&conn)
        .unwrap();
    
    let memos = memos::table
        .filter(memos::del.eq(0))
        .order(memos::created_at.desc())//added
        .limit(5)
        .load::<crate::models::Memo>(&conn)
        .expect("Error loading cards");
    
    ctx.insert("memos", &memos);
    let view = tmpl
        .render("form.html", &ctx)
        .map_err(|e| error::ErrorInternalServerError(e))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(view))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();
    let templates = Tera::new("templates/**/*").unwrap();

    let database_url = "database.sqlite3";
    let db_pool = r2d2::Pool::builder()
        .build(ConnectionManager::<SqliteConnection>::new(database_url))
        .expect("failed to create db connection pool");
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .data(templates.clone())
            .data(db_pool.clone())
            .route("/", web::get().to(greet))
            .route("/form", web::get().to(form))
            .route("/form/memo", web::post().to(memo_form))
            .service(form_one)
            .service(search)
            .service(delete)
    })
    .bind("0.0.0.0:9999")?
    .run()
    .await
}
