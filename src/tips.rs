use simplerand::rand_range;

const TIPS: &[&str] = &[
    "For different gif border colors, checkout the `--bg` option",
    "To add a pause at the end of the gif loop, use e.g. option `-e 3s`",
    "To add a pause at the beginning of the gif loop, use e.g. option `-s 500ms` option",
    "To prevent cutting out stall frames, checkout the `-n` option",
    "To remove the shadow around the gif, use the `-d none` option",
    "To double the capturing framerate, use the option `-f 8`",
    "For a mp4 video, use the `-m` option",
    "To suppress the 'Ctrl+D' banner, use the `-q` option",
];

///
/// needs to become random
pub fn show_tip() {
    let i = rand_range(0, TIPS.len() - 1);
    println!("💡 Tip: {}", &TIPS[i]);
}
