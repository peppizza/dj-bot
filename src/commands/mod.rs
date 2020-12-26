pub mod db_testing;
pub mod dj_only;
pub mod help;
pub mod join;
pub mod loop_command;
pub mod lyrics;
pub mod mute;
pub mod now_playing;
pub mod pause;
pub mod perms;
pub mod ping;
pub mod play;
pub mod prefix;
pub mod queue;
pub mod remove;
pub mod restart;
pub mod resume;
pub mod shuffle;
pub mod skip;
pub mod stop;
pub mod volume;

mod util {
    use std::{sync::Arc, time::Duration};

    use serenity::utils::MessageBuilder;

    use songbird::{
        input::Metadata,
        tracks::{PlayMode, TrackHandle},
    };

    pub fn format_duration_to_mm_ss(duration: Duration) -> String {
        let seconds = duration.as_secs() % 60;
        let minutes = (duration.as_secs() / 60) % 60;
        let hours = (duration.as_secs() / 60) / 60;

        if seconds < 10 {
            format!("{}:{}:0{}", hours, minutes, seconds)
        } else {
            format!("{}:{}:{}", hours, minutes, seconds)
        }
    }

    pub async fn formatted_song_listing(
        metadata: Arc<Metadata>,
        track: &TrackHandle,
        include_pos: bool,
        new_line: bool,
        place_in_queue: Option<usize>,
    ) -> Result<MessageBuilder, Box<dyn std::error::Error + Send + Sync>> {
        let metadata = metadata.clone();

        let track_info = track.get_info().await?;

        let is_playing = matches!(track_info.playing, PlayMode::Play);

        let track_len = metadata.duration.unwrap_or_default();

        let track_len_mm_ss = format_duration_to_mm_ss(track_len);

        let track_title = metadata.title.clone();

        let mut response = MessageBuilder::new();

        if include_pos {
            let track_pos = track_info.position;

            let track_pos_mm_ss = format_duration_to_mm_ss(track_pos);

            if is_playing {
                response.push_bold(format!("[ {}/{} ]▶ ", track_pos_mm_ss, track_len_mm_ss));
            } else {
                response.push_bold(format!("[ {}/{} ]⏸", track_pos_mm_ss, track_len_mm_ss));
            }

            response.push(format!("{} ", track_title.unwrap()));

            if new_line {
                response.push("\n\n");
            }

            Ok(response)
        } else {
            response
                .push_bold(format!("[ {} ]", track_len_mm_ss))
                .push(format!("{} ", track_title.unwrap_or_default()));

            let place_in_queue = place_in_queue.unwrap_or_default();

            if new_line {
                response
                    .push_mono(format!("{}", place_in_queue + 1))
                    .push("\n\n");
            } else {
                response.push_mono(format!("{}", place_in_queue + 1));
            }

            Ok(response)
        }
    }
}
