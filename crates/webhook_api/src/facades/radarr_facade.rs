//! Processes a [RadarrWebhook] into a [MessageData] to send to Matrix.

use crate::facades::{add_heading, add_quality, on_health_check};
use crate::models::radarr::{
    RadarrMovie, RadarrMovieFile, RadarrRelease, RadarrRemoteMovie, RadarrWebhook,
};
use anyhow::Result;
use tracing::{debug, info};
use yarrbot_matrix_client::message::{MessageData, MessageDataBuilder};

pub const RADARR_NAME: &str = "Radarr";

/// Process webhook data pushed from Radarr. The interaction differs based on the type of [RadarrWebhook] provided.
#[tracing::instrument(skip(data))]
pub async fn handle_radarr_webhook(
    data: RadarrWebhook,
    server_name: &Option<String>,
) -> Result<MessageData> {
    debug!("Processing Radarr webhook.");
    let message = match data {
        RadarrWebhook::Test {
            movie,
            remote_movie,
            release,
        } => on_test(movie, remote_movie, release, server_name),
        RadarrWebhook::Grab {
            movie,
            remote_movie,
            release,
            ..
        } => on_grab(movie, remote_movie, release, server_name),
        RadarrWebhook::Download {
            movie,
            remote_movie,
            movie_file,
            is_upgrade,
            ..
        } => on_download(movie, remote_movie, movie_file, is_upgrade, server_name),
        RadarrWebhook::Rename { movie } => on_rename(movie, server_name),
        RadarrWebhook::MovieDelete {
            movie,
            deleted_files,
        } => on_movie_delete(movie, deleted_files, server_name),
        RadarrWebhook::MovieFileDelete {
            movie,
            movie_file,
            delete_reason,
        } => on_movie_file_delete(movie, movie_file, delete_reason, server_name),
        RadarrWebhook::Health {
            level,
            message,
            health_type,
            wiki_url,
        } => on_health_check(
            RADARR_NAME,
            level,
            message,
            health_type,
            wiki_url,
            server_name,
        ),
    };

    Ok(message)
}

fn format_title_with_remote(remote_movie: &RadarrRemoteMovie) -> String {
    let mut result = String::from(&remote_movie.title);
    if remote_movie.year.is_some() {
        let year = remote_movie.year.unwrap();
        result.push_str(" (");
        result.push_str(&year.to_string());
        result.push(')');
    }

    result
}

fn format_title_with_movie(movie: &RadarrMovie) -> String {
    let mut result = String::from(&movie.title);
    if movie.release_date.is_some() {
        let release = movie.release_date.unwrap();
        let year = release.format("%Y").to_string();
        result.push_str(" (");
        result.push_str(&year);
        result.push(')');
    }

    result
}

fn add_release_date(builder: &mut MessageDataBuilder, movie: RadarrMovie) {
    if movie.release_date.is_some() {
        let release = movie.release_date.unwrap();
        builder.add_key_value("Release Date", &release.format("%Y-%m-%d").to_string())
    }
}

fn on_test(
    movie: RadarrMovie,
    remote_movie: RadarrRemoteMovie,
    release: RadarrRelease,
    server_name: &Option<String>,
) -> MessageData {
    info!("Received Test webhook from Radarr.");
    let mut builder = MessageDataBuilder::new();
    add_heading(
        &mut builder,
        "Radarr Test",
        &format_title_with_remote(&remote_movie),
        server_name,
    );
    add_release_date(&mut builder, movie);
    add_quality(&mut builder, &release.quality);

    builder.to_message_data()
}

fn on_grab(
    movie: RadarrMovie,
    remote_movie: RadarrRemoteMovie,
    release: RadarrRelease,
    server_name: &Option<String>,
) -> MessageData {
    info!("Received Grab webhook from Radarr.");
    let mut builder = MessageDataBuilder::new();
    add_heading(
        &mut builder,
        "Movie Grabbed",
        &format_title_with_remote(&remote_movie),
        server_name,
    );
    add_release_date(&mut builder, movie);
    add_quality(&mut builder, &release.quality);

    builder.to_message_data()
}

fn on_download(
    movie: RadarrMovie,
    remote_movie: RadarrRemoteMovie,
    movie_file: RadarrMovieFile,
    is_upgrade: bool,
    server_name: &Option<String>,
) -> MessageData {
    info!("Received Download webhook from Radarr.");
    let mut builder = MessageDataBuilder::new();
    add_heading(
        &mut builder,
        "Movie Downloaded",
        &format_title_with_remote(&remote_movie),
        server_name,
    );
    add_release_date(&mut builder, movie);
    add_quality(&mut builder, &movie_file.quality);
    builder.add_key_value("Is Upgrade", if is_upgrade { "Yes" } else { "No" });

    builder.to_message_data()
}

fn on_rename(movie: RadarrMovie, server_name: &Option<String>) -> MessageData {
    info!("Received Rename webhook from Radarr.");
    let mut builder = MessageDataBuilder::new();
    add_heading(
        &mut builder,
        "Movie Renamed",
        &format_title_with_movie(&movie),
        server_name,
    );
    if movie.file_path.is_some() {
        let optional_path = movie.file_path;
        builder.add_key_value_with_code("Path", &optional_path.unwrap());
    }

    builder.to_message_data()
}

fn on_movie_delete(
    movie: RadarrMovie,
    deleted_files: bool,
    server_name: &Option<String>,
) -> MessageData {
    info!("Received Movie Delete webhook from Radarr.");
    let mut builder = MessageDataBuilder::new();
    add_heading(
        &mut builder,
        "Movie Deleted",
        &format_title_with_movie(&movie),
        server_name,
    );
    builder.add_key_value("Files Deleted", if deleted_files { "Yes" } else { "No" });

    builder.to_message_data()
}

fn on_movie_file_delete(
    movie: RadarrMovie,
    movie_file: RadarrMovieFile,
    delete_reason: Option<String>,
    server_name: &Option<String>,
) -> MessageData {
    info!("Received Movie File Delete webhook from Radarr.");
    let mut builder = MessageDataBuilder::new();
    add_heading(
        &mut builder,
        "Movie File Deleted",
        &format_title_with_movie(&movie),
        server_name,
    );
    builder.add_key_value(
        "Reason",
        delete_reason
            .unwrap_or_else(|| String::from("No Reason Given"))
            .as_str(),
    );
    builder.add_key_value_with_code("Path", &movie_file.relative_path);

    builder.to_message_data()
}
