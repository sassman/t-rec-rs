pub const STARTER_CONFIG: &str = r#"# t-rec configuration file
# See: https://github.com/sassman/t-rec-rs

# Default settings applied to all recordings
[default]
# fps = 4
# wallpaper = "ventura"
# wallpaper-padding = 60
# start-pause = "2s"

# Named profiles for different use cases
# Use with: t-rec --profile demo

[profiles.demo]
wallpaper = "ventura"
wallpaper-padding = 100
start-pause = "5s"
idle-pause = "5s"

[profiles.smooth]
fps = 10
idle-pause = "2s"

[profiles.quick]
quiet = true
idle-pause = "1s"

# Example with custom wallpaper (use $HOME for home directory)
# [profiles.custom]
# wallpaper = "$HOME/Pictures/my-wallpaper.png"
# wallpaper-padding = 80
"#;
