pub mod help;
pub mod join;
pub mod leave;
pub mod loop_command;
pub mod mute;
pub mod now_playing;
pub mod pause;
pub mod ping;
pub mod play;
pub mod queue;
pub mod remove;
pub mod restart;
pub mod resume;
pub mod skip;
pub mod stop;
pub mod volume;

mod util {
    use std::time::Duration;

    pub fn format_duration_to_mm_ss(duration: Duration) -> String {
        let seconds = duration.as_secs() % 60;
        let minutes = (duration.as_secs() / 60) % 60;

        if seconds < 10 {
            format!("{}:0{}", minutes, seconds)
        } else {
            format!("{}:{}", minutes, seconds)
        }
    }
}
