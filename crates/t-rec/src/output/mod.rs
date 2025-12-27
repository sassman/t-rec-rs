//! Output generation module.
//!
//! This module consolidates all post-processing and output generation:
//! - Applying effects to frames (corners, shadows, wallpaper)
//! - Processing screenshots
//! - Generating GIF output
//! - Generating MP4 video output

use crate::common::utils::{print_tree_list, HumanReadable};
use crate::generators::{check_for_mp4, generate_gif, generate_mp4};
use crate::post_processing::{
    post_process_effects, post_process_screenshots, PostProcessingOptions,
};
use crate::prompt::{start_background_prompt, PromptResult};
use crate::recorder::{PostProcessConfig, RecordingResult};
use crate::utils::{target_file, DEFAULT_EXT, MOVIE_EXT};
use crate::Result;
use std::borrow::Borrow;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Configuration for output generation.
pub struct OutputConfig {
    /// Output file path (without extension)
    pub output_path: PathBuf,
    /// Generate GIF output
    pub generate_gif: bool,
    /// Generate MP4 output
    pub generate_video: bool,
    /// Quiet mode (no prompts)
    pub quiet: bool,
    /// Post-processing configuration
    pub post_process: PostProcessConfig,
}

/// Generates output files from recording results.
///
/// The OutputGenerator handles:
/// 1. Applying visual effects to captured frames
/// 2. Processing screenshots with the same effects
/// 3. Generating GIF files
/// 4. Generating MP4 video files
pub struct OutputGenerator {
    /// Recording result containing frames and metadata
    result: RecordingResult,
    /// Output configuration
    config: OutputConfig,
}

impl OutputGenerator {
    /// Create a new output generator.
    pub fn new(result: RecordingResult, config: OutputConfig) -> Self {
        Self { result, config }
    }

    /// Process all outputs.
    ///
    /// This is the main entry point that handles:
    /// 1. Applying effects to frames
    /// 2. Processing screenshots
    /// 3. Generating GIF (if requested)
    /// 4. Generating video (if requested or user approves)
    ///
    /// Returns the total time spent on processing.
    pub fn process(self) -> Result<Duration> {
        println!();
        println!("🎆 Applying effects (might take a bit)");
        crate::tips::show_tip();

        // Build post-processing options
        let post_opts = self.build_post_processing_options();

        // Apply effects to frames
        self.apply_effects(&post_opts);

        let target = target_file(self.config.output_path.to_str().unwrap_or("t-rec"));

        // Process screenshots
        self.process_screenshots(&target, &post_opts);

        println!();

        let mut total_time = Duration::default();

        // Start video prompt in background if we might need to ask
        let video_prompt = if !self.config.generate_video && !self.config.quiet {
            start_background_prompt("🎬 Also generate MP4 video?", 15)
        } else {
            None
        };

        // Generate GIF
        if self.config.generate_gif {
            let start = Instant::now();
            generate_gif(
                &self.result.time_codes.lock().unwrap(),
                self.result.tempdir.lock().unwrap().borrow(),
                &format!("{}.{}", target, DEFAULT_EXT),
                Some(self.config.post_process.start_delay).filter(|d| !d.is_zero()),
                Some(self.config.post_process.end_delay).filter(|d| !d.is_zero()),
            )?;
            total_time += start.elapsed();
        }

        // Determine if we should generate video
        let should_generate_video = if self.config.generate_video {
            true
        } else if let Some(prompt) = video_prompt {
            match prompt.wait() {
                PromptResult::Yes => {
                    check_for_mp4()?;
                    true
                }
                PromptResult::No | PromptResult::Timeout => false,
            }
        } else {
            false
        };

        // Generate video
        if should_generate_video {
            let start = Instant::now();
            generate_mp4(
                &self.result.time_codes.lock().unwrap(),
                self.result.tempdir.lock().unwrap().borrow(),
                &format!("{}.{}", target, MOVIE_EXT),
                self.config.post_process.fps,
            )?;
            total_time += start.elapsed();
        }

        println!("Time: {}", total_time.as_human_readable());

        Ok(total_time)
    }

    /// Build post-processing options from config.
    fn build_post_processing_options(&self) -> PostProcessingOptions<'_> {
        if let Some((ref wallpaper, padding)) = self.config.post_process.wallpaper {
            PostProcessingOptions::new(
                &self.config.post_process.decor,
                &self.config.post_process.bg_color,
            )
            .with_wallpaper(wallpaper, padding)
        } else {
            PostProcessingOptions::new(
                &self.config.post_process.decor,
                &self.config.post_process.bg_color,
            )
        }
    }

    /// Apply visual effects to all captured frames.
    fn apply_effects(&self, opts: &PostProcessingOptions) {
        let temp_path = self.result.tempdir.lock().unwrap().path().to_path_buf();
        let codes = self.result.time_codes.lock().unwrap();
        let frame_files: Vec<_> = codes
            .iter()
            .map(|tc| temp_path.join(crate::utils::file_name_for(tc, crate::utils::IMG_EXT)))
            .collect();
        post_process_effects(&frame_files, opts);
    }

    /// Process screenshots with effects and save them.
    fn process_screenshots(&self, target: &str, opts: &PostProcessingOptions) {
        if !self.result.screenshots.is_empty() {
            println!();
            println!(
                "📸 Processing {} screenshot(s)...",
                self.result.screenshots.len()
            );
            let saved_screenshots =
                post_process_screenshots(&self.result.screenshots, target, opts);

            // Print saved screenshots with tree-style formatting
            if !saved_screenshots.is_empty() {
                println!("   Screenshots saved:");
                print_tree_list(&saved_screenshots);
            }
        }
    }
}
