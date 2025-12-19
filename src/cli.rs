//! Parse command line arguments.
//!
//! # Why no `default_value` on CliArgs?
//!
//! You might be tempted to use clap's `default_value` or `default_value_t` attributes. **Don't!**
//!
//! The problem: clap applies defaults *before* we can check if the user actually provided a value.
//! This breaks our config precedence model where config/profile values should override CLI defaults,
//! but explicit CLI args should override config.
//!
//! Example of what goes wrong with `default_value`:
//! - Config file has `fps = 10`
//! - User runs `t-rec` (no `--fps` flag)
//! - With `default_value_t = 4`: clap sets `fps = 4`, we can't tell user didn't specify it,
//!   config's `fps = 10` is ignored
//! - With `Option<u8>` (no default): clap sets `fps = None`, we know to use config's `fps = 10`
//!
//! By using `Option<T>` without defaults, `None` clearly means "user didn't specify, use config
//! or fall back to default". The actual defaults are applied later in `ProfileSettings` accessor
//! methods, after config merging.
//!
//! Note: The default values shown in help text (e.g., `[default: 4]`) must be kept in sync with
//! the constants in `config::defaults` module. This duplication is unfortunate but necessary
//! because Rust's `concat!` macro only works with literal strings, not const references.

use clap::Parser;

use crate::config::{resolve_settings, ProfileSettings};

/// Blazingly fast terminal recorder that generates animated gif images for the web
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct CliArgs {
    /// Enable verbose insights for the curious
    #[arg(short, long)]
    pub verbose: bool,

    /// Quiet mode, suppresses the banner: 'Press Ctrl+D to end recording'
    #[arg(short, long)]
    pub quiet: bool,

    /// Generates additionally to the gif a mp4 video of the recording
    #[arg(short = 'm', long)]
    pub video: bool,

    /// Generates only a mp4 video and not gif
    #[arg(short = 'M', long = "video-only", conflicts_with = "video")]
    pub video_only: bool,

    /// Decorates the animation with certain, mostly border effects [default: none]
    #[arg(short, long, value_parser = ["shadow", "none"])]
    pub decor: Option<String>,

    /// Wallpaper background. Use 'ventura' for built-in, or provide a path to a custom image (PNG, JPEG, TGA)
    #[arg(short = 'p', long)]
    pub wallpaper: Option<String>,

    /// Padding in pixels around the recording when using --wallpaper [default: 60]
    #[arg(long = "wallpaper-padding", value_parser = clap::value_parser!(u32).range(1..=500))]
    pub wallpaper_padding: Option<u32>,

    /// Background color when decors are used [default: transparent]
    #[arg(short, long, value_parser = ["white", "black", "transparent"])]
    pub bg: Option<String>,

    /// If you want a very natural typing experience and disable the idle detection and sampling optimization
    #[arg(short, long = "natural")]
    pub natural: bool,

    /// If you want to see a list of windows available for recording by their id
    #[arg(short = 'l', long = "ls-win", visible_alias = "ls")]
    pub list_windows: bool,

    /// Window Id (see --ls-win) that should be captured, instead of the current terminal
    #[arg(short = 'w', long = "win-id")]
    pub win_id: Option<u64>,

    /// Pause time at the end of the animation (e.g., "2s", "500ms")
    #[arg(short = 'e', long = "end-pause")]
    pub end_pause: Option<String>,

    /// Pause time at the start of the animation (e.g., "1s", "200ms")
    #[arg(short = 's', long = "start-pause")]
    pub start_pause: Option<String>,

    /// Max idle time before optimization kicks in. Can enhance readability [default: 3s]
    #[arg(short = 'i', long = "idle-pause")]
    pub idle_pause: Option<String>,

    /// Output file without extension [default: t-rec]
    #[arg(short = 'o', long = "output")]
    pub output: Option<String>,

    /// Capture framerate, 4-15 fps. Higher = smoother but larger files [default: 4]
    #[arg(short = 'f', long, value_parser = clap::value_parser!(u8).range(4..=15))]
    pub fps: Option<u8>,

    /// Shell or program to launch with optional arguments. Defaults to $SHELL.
    /// Use -- to separate t-rec options from shell arguments (e.g., t-rec -- /bin/bash -l)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub program: Vec<String>,

    // --- Config-related args (not part of recording settings) ---
    /// Use a named profile from the config file
    #[arg(long)]
    pub profile: Option<String>,

    /// Create a starter config file at `~/.config/t-rec/config.toml` (linux) or `~/Library/Application Support/t-rec/config.toml` (macOS)
    #[arg(long = "init-config")]
    pub init_config: bool,

    /// List available profiles from the config file
    #[arg(long = "list-profiles")]
    pub list_profiles: bool,
}

pub fn launch() -> CliArgs {
    CliArgs::parse()
}

/// Load config and resolve settings: config defaults -> profile -> CLI args
pub fn resolve_profiled_settings(args: &CliArgs) -> anyhow::Result<ProfileSettings> {
    let config = crate::config::load_config()?;
    let profile_name = args.profile.as_deref();

    resolve_settings(config.as_ref(), profile_name, args)
}
