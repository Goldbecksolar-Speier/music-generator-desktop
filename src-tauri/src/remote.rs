//! Optional remote engine: the ace-server instances run on another machine
//! (typically reached through an SSH tunnel) instead of as child processes of the app.
//!
//! Enabled with `ACE_REMOTE_SYNTH_URL` (and optionally `ACE_REMOTE_LM_URL`,
//! `ACE_REMOTE_LM_MODEL`). Models live on the remote machine; the app neither downloads
//! nor verifies them locally.

use crate::manifest;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Remote {
    pub synth: String,
    pub lm: String,
    pub lm_model: String,
}

fn clean(value: Option<String>) -> Option<String> {
    let value = value?.trim().trim_end_matches('/').to_string();
    (!value.is_empty()).then_some(value)
}

pub fn parse(synth: Option<String>, lm: Option<String>, lm_model: Option<String>) -> Option<Remote> {
    let synth = clean(synth)?;
    Some(Remote {
        lm: clean(lm).unwrap_or_else(|| synth.clone()),
        lm_model: clean(lm_model).unwrap_or_else(|| manifest::lm_file("1.7B")),
        synth,
    })
}

pub fn config() -> Option<Remote> {
    parse(
        std::env::var("ACE_REMOTE_SYNTH_URL").ok(),
        std::env::var("ACE_REMOTE_LM_URL").ok(),
        std::env::var("ACE_REMOTE_LM_MODEL").ok(),
    )
}

pub fn is_remote() -> bool {
    config().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unset_or_blank_means_local() {
        assert_eq!(parse(None, None, None), None);
        assert_eq!(parse(Some("  ".into()), Some("http://x".into()), None), None);
    }

    #[test]
    fn lm_defaults_to_synth_server_and_urls_are_trimmed() {
        let remote = parse(Some(" http://127.0.0.1:8085/ ".into()), None, None).unwrap();
        assert_eq!(remote.synth, "http://127.0.0.1:8085");
        assert_eq!(remote.lm, "http://127.0.0.1:8085");
        assert_eq!(remote.lm_model, "acestep-5Hz-lm-1.7B-Q8_0.gguf");
    }

    #[test]
    fn separate_lm_server_and_model_are_kept() {
        let remote = parse(
            Some("http://127.0.0.1:8085".into()),
            Some("http://127.0.0.1:8086".into()),
            Some("acestep-5Hz-lm-0.6B-Q8_0.gguf".into()),
        )
        .unwrap();
        assert_eq!(remote.lm, "http://127.0.0.1:8086");
        assert_eq!(remote.lm_model, "acestep-5Hz-lm-0.6B-Q8_0.gguf");
    }
}
