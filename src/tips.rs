use simplerand::rand_range;

const TIPS: &[&str] = &[
    "For different gif border colors, checkout the `--bg` option",
    "To add a pause at the end of the gif loop, use e.g. option `-e 3s`",
    "To add a pause at the beginning of the gif loop, use e.g. option `-s 500ms` option",
    "To prevent cutting out stall frames, checkout the `-n` option",
    "To remove the shadow around the gif, use the `-d none` option",
    "For a mp4 video, use the `-m` option",
    "To suppress the 'Ctrl+D' banner, use the `-q` option",
    "Add `--idle-pause <time>` to increase the time before idle frame optimization starts, this can improve readability in Demos",
    "For a beautiful macOS-style background, try `--wallpaper ventura`",
    "Use your own wallpaper with `-p /path/to/image.png` (supports PNG, JPEG, TGA)",
    "Adjust wallpaper padding with `--wallpaper-padding 100` (default: 60px)",
    "Save your favorite settings with `t-rec --init-config` and edit ~/.config/t-rec/config.toml",
    "Create named profiles in your config file and use them with `--profile demo`",
    "List available profiles from your config with `t-rec --list-profiles`",
];

///
/// needs to become random
pub fn show_tip() {
    let i = rand_range(0, TIPS.len() - 1);
    println!("💡 Tip: {}", &TIPS[i]);
}
