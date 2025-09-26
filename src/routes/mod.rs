use actix_files as fs;

use actix_web::web::{scope, ServiceConfig};
use actix_web::Scope;
use books::get_books_by_scholar;
use files::{get_files_by_book, get_recent_files, get_related_files, view_file};
use scholars::{get_scholars, get_scholars_by_state, get_scholar_details, get_scholar_statistics};
use search::full_text_search;
use states::get_states;
use permissions::{get_user_permissions, grant_access, revoke_access, get_all_accesses};
use uploads::{upload_file, download_file};
use users::{register, login, get_profile, update_profile, change_password, forgot_password, reset_password, deactivate_account};
use subscriptions::{get_subscription_plans, get_user_subscriptions, get_subscription_status, get_active_subscription, create_subscription, get_pending_subscriptions, verify_subscription};
use follows::{follow_scholar, unfollow_scholar, update_follow_settings, get_my_followed_scholars, check_follow_status};
use play_history::{record_play, get_my_play_history, get_most_played_files, clear_play_history, get_file_play_stats};
use playlists::{create_playlist, get_my_playlists, get_public_playlists, get_playlist, update_playlist, delete_playlist, add_file_to_playlist, remove_file_from_playlist, get_playlist_files};
use file_interactions::{report_file, get_pending_reports, resolve_report, like_file, unlike_file, get_file_likes, check_file_like_status, create_comment, get_file_comments, update_comment, delete_comment, get_file_download_stats, get_my_download_history};
mod books;
mod files;
mod health_check;
mod scholars;
mod search;
mod states;
mod permissions;
mod uploads;
mod users;
mod subscriptions;
mod follows;
mod play_history;
mod playlists;
mod file_interactions;

use crate::routes::health_check::*;
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
        // follow routes
        .service(follow_scholar)
        .service(unfollow_scholar)
        .service(update_follow_settings)
        .service(get_my_followed_scholars)
        .service(check_follow_status)
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

fn subscriptions_routes() -> Scope {
    scope("subscriptions")
        .service(get_subscription_plans)
        .service(get_user_subscriptions)
        .service(get_subscription_status)
        .service(get_active_subscription)
        .service(create_subscription)
        .service(get_pending_subscriptions)
        .service(verify_subscription)
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
            .service(subscriptions_routes())
            .service(play_history_routes())
            .service(playlists_routes())
            .service(static_files_routes())
            .service(util_routes()),
    );
}
