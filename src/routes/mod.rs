use actix_files as fs;

use actix_web::web::{scope, ServiceConfig};
use actix_web::Scope;
use books::{get_book_details, get_book_statistics, get_books_by_scholar, get_books_dropdown, create_book, update_book, delete_book};
use file_interactions::{
    check_file_like_status, create_comment, delete_comment, get_file_comments,
    get_file_download_stats, get_file_likes, get_my_download_history, get_pending_reports,
    like_file, report_file, resolve_report, unlike_file, update_comment,
};
use files::{
    get_all_files_for_play_all, get_files_by_book, get_recent_files, get_related_files, view_file, update_file, delete_file,
};
use follows::{
    check_follow_status, follow_scholar, get_my_followed_scholars, unfollow_scholar,
    update_follow_settings,
};
use permissions::{get_all_accesses, get_user_permissions, grant_access, revoke_access};
use play_history::{
    clear_play_history, get_file_play_stats, get_most_played_files, get_my_play_history,
    record_play,
};
use playlists::{
    add_file_to_playlist, create_playlist, delete_playlist, get_my_playlists, get_playlist,
    get_playlist_files, get_public_playlists, remove_file_from_playlist, update_playlist,
};
use related_files::get_file_suggestions;
use scholars::{get_scholar_details, get_scholar_statistics, get_scholars, get_scholars_by_state, get_scholars_dropdown, create_scholar, update_scholar, delete_scholar};
use search::full_text_search;
use states::get_states;
use subscriptions::{
    create_subscription, get_active_subscription, get_pending_subscriptions,
    get_subscription_plans, get_subscription_status, get_user_subscriptions, verify_subscription,
    expire_subscriptions,
};
use uploads::{download_file, track_download, upload_file};
use users::{
    change_password, deactivate_account, forgot_password, get_profile, login, register,
    reset_password, update_profile, refresh_token_endpoint, logout,
};
use settings::get_site_settings;
mod books;
mod file_interactions;
mod files;
mod follows;
mod health_check;
mod permissions;
mod play_history;
mod playlists;
mod related_files;
mod scholars;
mod search;
mod states;
mod subscriptions;
mod uploads;
mod users;
mod settings;

use crate::routes::health_check::*;
// const IMAGES_DIR: &str = "/home/mubarak/Documents/my-documents/muryar_sunnah/web/images";
// const IMAGES_DIR: &str = "./static/images";

fn util_routes() -> Scope {
    scope("")
        .service(get_states)
        .service(get_site_settings)
        .service(full_text_search)
        .service(health_check)
}

fn books_routes() -> Scope {
    scope("books")
        .service(get_files_by_book)
        .service(get_all_files_for_play_all)
        .service(get_book_details)
        .service(get_book_statistics)
        .service(upload_file)
        .service(get_books_dropdown)
        .service(create_book)
        .service(update_book)
        .service(delete_book)
}

fn files_routes() -> Scope {
    scope("files")
        .service(get_recent_files)
        .service(view_file)
        .service(get_related_files)
        .service(get_file_suggestions) // New endpoint for next/previous suggestions
        .service(download_file)
        .service(track_download) // Track downloads without downloading
        .service(update_file)
        .service(delete_file)
        // file_interactions_routes
        .service(report_file)
        .service(get_pending_reports)
        .service(resolve_report)
        .service(like_file)
        .service(unlike_file)
        .service(get_file_likes)
        .service(check_file_like_status)
        .service(create_comment)
        .service(get_file_comments)
        .service(update_comment)
        .service(delete_comment)
        .service(get_file_download_stats)
        .service(get_my_download_history)
}

fn auth_routes() -> Scope {
    scope("auth")
        // Removed old login service - u
        .service(register)
        .service(login)
        .service(refresh_token_endpoint)
        .service(logout)
        .service(get_profile)
        .service(update_profile)
        .service(change_password)
        .service(forgot_password)
        .service(reset_password)
        .service(deactivate_account)
        .service(get_user_permissions)
        .service(grant_access)
        .service(revoke_access)
        .service(get_all_accesses)
}

fn scholars_routes() -> Scope {
    scope("scholars")
        .service(get_scholars)
        .service(get_scholars_by_state)
        .service(get_scholar_details)
        .service(get_scholar_statistics)
        .service(get_books_by_scholar)
        .service(get_scholars_dropdown)
        .service(create_scholar)
        .service(update_scholar)
        .service(delete_scholar)
        // follow routes
        .service(follow_scholar)
        .service(unfollow_scholar)
        .service(update_follow_settings)
        .service(get_my_followed_scholars)
        .service(check_follow_status)
}

fn users_routes() -> Scope {
    scope("users")
        // .service(register)
        // .service(login)
        // .service(refresh_token_endpoint)
        // .service(logout)
        // .service(get_profile)
        // .service(update_profile)
        // .service(change_password)
        // .service(forgot_password)
        // .service(reset_password)
        // .service(deactivate_account)
}

fn subscriptions_routes() -> Scope {
    scope("subscriptions")
        .service(get_subscription_plans)
        .service(get_user_subscriptions)
        .service(get_subscription_status)
        .service(get_active_subscription)
        .service(create_subscription)
        .service(get_pending_subscriptions)
        .service(verify_subscription)
        .service(expire_subscriptions)
}

fn play_history_routes() -> Scope {
    scope("play-history")
        .service(record_play)
        .service(get_my_play_history)
        .service(get_most_played_files)
        .service(clear_play_history)
        .service(get_file_play_stats)
}

fn playlists_routes() -> Scope {
    scope("playlists")
        .service(create_playlist)
        .service(get_my_playlists)
        .service(get_public_playlists)
        .service(get_playlist)
        .service(update_playlist)
        .service(delete_playlist)
        .service(add_file_to_playlist)
        .service(remove_file_from_playlist)
        .service(get_playlist_files)
}

fn static_files_routes(config: &crate::core::config::AppConfig) -> Scope {
    scope("static")
        // Serve album images from `/static/images/`
        .service(fs::Files::new("/images", &config.app_paths.images_dir))
        // Serve audio files from `/static/audio/`
        .service(fs::Files::new("/audio", &config.app_paths.uploads_dir))
}

pub fn sunnah_audio_routes(conf: &mut ServiceConfig, config: &crate::core::config::AppConfig) {
    conf.service(
        scope("api/v1")
            .service(auth_routes())
            .service(scholars_routes())
            .service(books_routes())
            .service(files_routes())
            .service(users_routes())
            .service(subscriptions_routes())
            .service(play_history_routes())
            .service(playlists_routes())
            .service(static_files_routes(config))
            .service(util_routes()),
    );
}
