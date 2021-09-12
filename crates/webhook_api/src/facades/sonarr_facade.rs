//! Handles Sonarr-Matrix interactions.

use crate::facades::{add_heading, add_quality, on_health_check};
use crate::models::sonarr::{
    SonarrEpisode, SonarrEpisodeDeletedFile, SonarrEpisodeFile, SonarrRelease,
    SonarrRenamedEpisodeFile, SonarrSeries, SonarrWebhook,
};
use anyhow::Result;
use yarrbot_matrix_client::message::{MatrixMessageDataPart, MessageData, MessageDataBuilder};
use yarrbot_db::enums::ArrType;

/// Process webhook data pushed from Sonarr. The interaction differs based on the type of [SonarrWebhook] provided.
pub async fn handle_sonarr_webhook(
    data: &SonarrWebhook,
) -> Result<MessageData> {
    let message = match data {
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
        } => on_health_check(&ArrType::Sonarr, level, message, health_type, wiki_url),
    };

    Ok(message)
}

fn add_episodes(builder: &mut MessageDataBuilder, episodes: &[SonarrEpisode]) {
    if episodes.is_empty() {
        builder.add_line("No episodes specified.");
        builder.break_character();
        return;
    }

    for episode in episodes {
        builder.add_key_value("Season", &format!("{:0>2}", &episode.season_number));
        builder.add_key_value("Episode", &format!("{:0>2}", &episode.episode_number));
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
    info!("Received Grab webhook from Sonarr.");
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
    info!("Received Download webhook from Sonarr.");
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
    info!("Received Rename webhook from Sonarr.");
    let mut builder = MessageDataBuilder::new();
    add_heading(&mut builder, "Series Renamed", &series.title);
    builder.add_matrix_message_part(RenamedFiles::new(renamed_episode_files));

    builder.to_message_data()
}

fn on_series_delete(series: &SonarrSeries, deleted_files: &bool) -> MessageData {
    info!("Received Series Delete webhook from Sonarr.");
    let mut builder = MessageDataBuilder::new();
    add_heading(&mut builder, "Series Deleted", &series.title);
    builder.add_key_value("Files Deleted", if *deleted_files { "Yes" } else { "No" });

    builder.to_message_data()
}

fn on_episode_file_delete(
    series: &SonarrSeries,
    episodes: &[SonarrEpisode],
    episode_file: &SonarrEpisodeDeletedFile,
    reason: &Option<String>,
) -> MessageData {
    info!("Received Episode File Delete webhook from Sonarr.");
    let mut builder = MessageDataBuilder::new();
    add_heading(&mut builder, "Series Episode Files Deleted", &series.title);
    builder.add_key_value(
        "Reason",
        reason
            .as_ref()
            .unwrap_or(&String::from("No Reason Given"))
            .as_str(),
    );
    let q = if let Some(quality) = &episode_file.quality {
        quality.quality.name.clone()
    } else {
        String::from("(No Quality Given)")
    };
    add_quality(&mut builder, &Some(q));
    builder.break_character();
    add_episodes(&mut builder, episodes);

    builder.to_message_data()
}

fn on_test(series: &SonarrSeries, episodes: &[SonarrEpisode]) -> MessageData {
    info!("Received Test webhook from Sonarr.");
    let mut builder = MessageDataBuilder::new();
    add_heading(&mut builder, "Sonarr Test", &series.title);
    add_episodes(&mut builder, episodes);

    builder.to_message_data()
}