//! Handles Sonarr-Matrix interactions.

use crate::facades::{add_heading, add_quality, send_matrix_messages};
use crate::models::common::ArrHealthCheckResult;
use crate::models::sonarr::{
    SonarrEpisode, SonarrEpisodeFile, SonarrRelease, SonarrRenamedEpisodeFile, SonarrSeries,
    SonarrWebhook,
};
use actix_web::HttpResponse;
use anyhow::Result;
use yarrbot_db::models::Webhook;
use yarrbot_db::DbPool;
use yarrbot_matrix_client::message::{
    MatrixMessageDataPart, MessageData, MessageDataBuilder, SectionHeadingLevel,
};
use yarrbot_matrix_client::YarrbotMatrixClient;

/// Process webhook data pushed from Sonarr. This method will post messages to the rooms configured for
/// the webhook database record. The interaction differs based on the type of [SonarrWebhook] provided.
pub async fn handle_sonarr_webhook(
    webhook: &Webhook,
    data: &SonarrWebhook,
    pool: &DbPool,
    matrix_client: &YarrbotMatrixClient,
) -> Result<HttpResponse> {
    let message: MessageData = match data {
        SonarrWebhook::Test { series, episodes } => on_test(series, episodes),
        SonarrWebhook::Grab {
            series,
            episodes,
            release,
            ..
        } => on_grab(series, episodes, release),
        SonarrWebhook::Download {
            series,
            episodes,
            episode_file,
            is_upgrade,
            ..
        } => on_download(series, episodes, episode_file, is_upgrade),
        SonarrWebhook::Rename {
            series,
            renamed_episode_files,
        } => on_rename(series, renamed_episode_files),
        SonarrWebhook::SeriesDelete {
            series,
            deleted_files,
        } => on_series_delete(series, deleted_files),
        SonarrWebhook::EpisodeFileDelete {
            series,
            episodes,
            episode_file,
            delete_reason,
        } => on_episode_file_delete(series, episodes, episode_file, delete_reason),
        SonarrWebhook::Health {
            level,
            message,
            health_type,
            wiki_url,
        } => on_health_check(level, message, health_type, wiki_url),
    };

    match send_matrix_messages(pool, &webhook.id, matrix_client, &message).await {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(e) => {
            error!("Encountered error while sending Matrix messages: {:?}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}

fn add_episodes(builder: &mut MessageDataBuilder, episodes: &[SonarrEpisode]) {
    if episodes.is_empty() {
        builder.add_line("No episodes specified.");
        builder.break_character();
        return;
    }

    for episode in episodes {
        builder.add_key_value("Season", &episode.season_number.to_string());
        builder.add_key_value("Episode", &episode.episode_number.to_string());
        builder.add_key_value("Title", &episode.title);
        if episode.air_date_utc.is_some() {
            let air = episode.air_date_utc.unwrap();
            builder.add_key_value("Air Date (UTC)", &air.format("%Y-%m-%d").to_string());
        }
        builder.break_character();
    }
}

fn on_grab(
    series: &SonarrSeries,
    episodes: &[SonarrEpisode],
    release: &SonarrRelease,
) -> MessageData {
    let mut builder = MessageDataBuilder::new();
    add_heading(&mut builder, "Series Grabbed", &series.title);
    add_quality(&mut builder, &release.quality);
    builder.break_character();
    add_episodes(&mut builder, episodes);

    builder.to_message_data()
}

fn on_download(
    series: &SonarrSeries,
    episodes: &[SonarrEpisode],
    episode_file: &SonarrEpisodeFile,
    is_upgrade: &bool,
) -> MessageData {
    let mut builder = MessageDataBuilder::new();
    add_heading(&mut builder, "Series Downloaded", &series.title);
    add_quality(&mut builder, &episode_file.quality);
    builder.add_key_value("Is Upgrade", if *is_upgrade { "Yes" } else { "No" });
    builder.break_character();
    add_episodes(&mut builder, episodes);

    builder.to_message_data()
}

// TODO: This is extremely similar to the "webhook list" command; need to refactor and generalize a bit.
struct RenamedFiles(String, String);

impl RenamedFiles {
    fn new(items: &[SonarrRenamedEpisodeFile]) -> Self {
        if items.is_empty() {
            return RenamedFiles(
                String::from("No rename data found."),
                String::from("No rename data found."),
            );
        }

        let mut plain = String::from(' ');
        let mut html = String::from("<ul>");
        let length = items.len();
        for (i, item) in items.iter().enumerate() {
            if item.previous_relative_path.is_some() && item.relative_path.is_some() {
                let prev = item.previous_relative_path.as_ref().unwrap();
                let next = item.relative_path.as_ref().unwrap();
                let formatted = format!("{} --> {}", prev, next);
                plain.push_str(&formatted);

                html.push_str("<li><code>");
                html.push_str(&formatted);
                html.push_str("</code></li>");
            } else {
                let formatted = format!("(File #{} was missing path data)", i + 1);
                plain.push_str(&formatted);
                html.push_str(&formatted);
            }

            if i < (length - 1) {
                plain.push_str(", ");
            }
        }

        html.push_str("</ul>");

        RenamedFiles(plain, html)
    }
}

impl MatrixMessageDataPart for RenamedFiles {
    fn to_plain(&self, break_character: &str) -> String {
        format!(" {} {}", self.0, break_character)
    }

    fn to_html(&self, break_character: &str) -> String {
        format!(" {} {}", self.1, break_character)
    }
}

fn on_rename(
    series: &SonarrSeries,
    renamed_episode_files: &[SonarrRenamedEpisodeFile],
) -> MessageData {
    let mut builder = MessageDataBuilder::new();
    add_heading(&mut builder, "Series Renamed", &series.title);
    builder.add_matrix_message_part(RenamedFiles::new(renamed_episode_files));

    builder.to_message_data()
}

fn on_series_delete(series: &SonarrSeries, deleted_files: &bool) -> MessageData {
    let mut builder = MessageDataBuilder::new();
    add_heading(&mut builder, "Series Deleted", &series.title);
    builder.add_key_value("Files Deleted", if *deleted_files { "Yes" } else { "No" });

    builder.to_message_data()
}

fn on_episode_file_delete(
    series: &SonarrSeries,
    episodes: &[SonarrEpisode],
    episode_file: &SonarrEpisodeFile,
    reason: &Option<String>,
) -> MessageData {
    let mut builder = MessageDataBuilder::new();
    add_heading(&mut builder, "Series Episode Files Deleted", &series.title);
    builder.add_key_value(
        "Reason",
        reason
            .as_ref()
            .unwrap_or(&String::from("No Reason Given"))
            .as_str(),
    );
    add_quality(&mut builder, &episode_file.quality);
    builder.break_character();
    add_episodes(&mut builder, episodes);

    builder.to_message_data()
}

fn on_test(series: &SonarrSeries, episodes: &[SonarrEpisode]) -> MessageData {
    let mut builder = MessageDataBuilder::new();
    add_heading(&mut builder, "Sonarr Test", &series.title);
    add_episodes(&mut builder, episodes);

    builder.to_message_data()
}

fn on_health_check(
    level: &Option<ArrHealthCheckResult>,
    message: &Option<String>,
    health_type: &Option<String>,
    wiki_url: &Option<String>,
) -> MessageData {
    let mut builder = MessageDataBuilder::new();
    builder.add_heading(&SectionHeadingLevel::One, "Sonarr Health Check");
    if level.is_some() {
        let l = match level.as_ref().unwrap() {
            ArrHealthCheckResult::Ok => "Ok",
            ArrHealthCheckResult::Notice => "Notice",
            ArrHealthCheckResult::Warning => "Warning",
            ArrHealthCheckResult::Error => "Error",
            ArrHealthCheckResult::Unknown => "Unknown",
        };
        builder.add_key_value("Level", l);
    } else {
        builder.add_key_value("Level", "Unknown");
    }

    builder.add_key_value(
        "Message",
        message
            .as_ref()
            .unwrap_or(&String::from("No Message Given")),
    );
    builder.add_key_value(
        "Type",
        health_type
            .as_ref()
            .unwrap_or(&String::from("No Message Given")),
    );
    builder.add_key_value(
        "Wiki URL",
        wiki_url
            .as_ref()
            .unwrap_or(&String::from("No Message Given")),
    );

    builder.to_message_data()
}
