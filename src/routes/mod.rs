
use actix_web::web::{scope, ServiceConfig};
use actix_web::Scope;
use books::get_books_by_scholar;
use files::{get_files_by_book, get_recent_files};
use scholars::{get_scholars, get_scholars_by_state};
use states::get_states;
mod auth;
mod health_check;
mod states;
mod scholars;
mod books;
mod files;

use crate::routes::health_check::*;
use self::auth::auth_routes::login;
// use self::vas::{cable_networks::*, mobile_networks::*, data_plans::*};

fn util_routes() -> Scope {
    scope("")
        .service(get_states)
        .service(get_recent_files)
        .service(health_check)
}

fn books_routes() -> Scope {
    scope("books")
        .service(get_files_by_book)
        // .service(view_accounts)
        // .service(get_account_balance)
}

fn auth_routes() -> Scope {
    scope("auth").service(login)
}

fn scholars_routes() -> Scope {
    scope("scholars")
        .service(get_scholars)
        .service(get_scholars_by_state)
        .service(get_books_by_scholar)
}

pub fn sunnah_audio_routes(conf: &mut ServiceConfig) {
    conf.service(
        scope("api/v1")
            .service(auth_routes())
            .service(scholars_routes())
            .service(books_routes())
            .service(util_routes()),
    );
}
