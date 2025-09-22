use actix_files as fs;

use actix_web::web::{scope, ServiceConfig};
use actix_web::Scope;
use books::get_books_by_scholar;
use files::{get_files_by_book, get_recent_files, get_related_files, view_file};
use scholars::{get_scholars, get_scholars_by_state};
use search::full_text_search;
use states::get_states;
use permissions::{get_user_permissions, grant_access, revoke_access, get_all_accesses};
use uploads::{upload_file, download_file};
use users::{register, login, get_profile, update_profile, change_password, forgot_password, reset_password, deactivate_account};
mod books;
mod files;
mod health_check;
mod scholars;
mod search;
mod states;
mod permissions;
mod uploads;
mod users;

// Removed old auth login import - using new user authentication system
use crate::routes::health_check::*;
// use self::vas::{cable_networks::*, mobile_networks::*, data_plans::*};
const IMAGES_DIR: &str = "/home/mubarak/Documents/my-documents/muryar_sunnah/web/images";

fn util_routes() -> Scope {
    scope("")
        .service(get_states)
        .service(full_text_search)
        .service(health_check)
}

fn books_routes() -> Scope {
    scope("books")
        .service(get_files_by_book)
        .service(upload_file)
}

fn files_routes() -> Scope {
    scope("files")
        .service(get_recent_files)
        .service(view_file)
        .service(get_related_files)
        .service(download_file)
}

fn auth_routes() -> Scope {
    scope("auth")
        // Removed old login service - u
        .service(get_user_permissions)
        .service(grant_access)
        .service(revoke_access)
        .service(get_all_accesses)
}

fn scholars_routes() -> Scope {
    scope("scholars")
        .service(get_scholars)
        .service(get_scholars_by_state)
        .service(get_books_by_scholar)
}

fn users_routes() -> Scope {
    scope("users")
        .service(register)
        .service(login)
        .service(get_profile)
        .service(update_profile)
        .service(change_password)
        .service(forgot_password)
        .service(reset_password)
        .service(deactivate_account)
}

fn static_files_routes() -> Scope {
    scope("static")
        // Serve album images from `/static/images/`
        .service(fs::Files::new("/images", IMAGES_DIR))
        // Serve audio files from `/static/audio/`
        .service(fs::Files::new("/audio", "./static/audio").show_files_listing())
}

pub fn sunnah_audio_routes(conf: &mut ServiceConfig) {
    conf.service(
        scope("api/v1")
            .service(auth_routes())
            .service(scholars_routes())
            .service(books_routes())
            .service(files_routes())
            .service(users_routes())
            .service(static_files_routes())
            .service(util_routes()),
    );
}
