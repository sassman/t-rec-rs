use crate::common::identify_transparency::identify_transparency;
use crate::common::image::convert_bgra_to_rgba;
use crate::{Margin, PlatformApi, Result, WindowId, WindowList};

use crate::common::Frame;
use anyhow::Context;
use log::debug;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::rust_connection::{DefaultStream, RustConnection};

x11rb::atom_manager! {
    pub Atoms: AtomsCookie {
        _NET_WM_NAME,
        _NET_WM_WINDOW_TYPE,
        _NET_WM_WINDOW_TYPE_DESKTOP,
        _NET_WM_WINDOW_TYPE_DOCK,
        _NET_WM_WINDOW_TYPE_TOOLBAR,
        _NET_WM_WINDOW_TYPE_MENU,
        _NET_WM_WINDOW_TYPE_UTILITY,
        _NET_WM_WINDOW_TYPE_SPLASH,
        _NET_WM_WINDOW_TYPE_DIALOG,
        _NET_WM_WINDOW_TYPE_NORMAL,
        _NET_ACTIVE_WINDOW,
        UTF8_STRING,
    }
}

pub struct X11Api {
    conn: RustConnection<DefaultStream>,
    screen_num: usize,
    atoms: Atoms,
    margin: Option<Margin>,
}

impl X11Api {
    pub fn new() -> Result<Self> {
        let (conn, screen_num) = RustConnection::connect(None)?;
        let atoms = Atoms::new(&conn)?.reply()?;
        Ok(Self {
            conn,
            screen_num,
            atoms,
            margin: None,
        })
    }

    pub fn screen(&self) -> &Screen {
        &self.conn.setup().roots[self.screen_num]
    }

    pub fn get_window_name(&self, window: &WindowId) -> Result<Option<String>> {
        let conn = &self.conn;
        let window = *window as Window;
        let atoms = Atoms::new(conn)?.reply()?;

        let prop = conn
            .get_property(
                false,
                window,
                atoms._NET_WM_NAME,
                atoms.UTF8_STRING,
                0,
                u32::MAX,
            )?
            .reply()?;
        let name = if prop.type_ == x11rb::NONE {
            let prop = conn
                .get_property(
                    false,
                    window,
                    AtomEnum::WM_NAME,
                    AtomEnum::STRING,
                    0,
                    u32::MAX,
                )?
                .reply()?;
            std::str::from_utf8(&prop.value)?.to_owned()
        } else {
            std::str::from_utf8(&prop.value)?.to_owned()
        };

        Ok(if !name.is_empty() { Some(name) } else { None })
    }

    pub fn get_all_sub_windows(&self, root: &WindowId) -> Result<Vec<WindowId>> {
        let root = *root as Window;
        let conn = &self.conn;
        let tree = conn.query_tree(root)?.reply()?;
        let mut result = vec![];
        for window in tree.children {
            let window_id = window as WindowId;
            let (_, _, width, height) = self.get_window_geometry(&window_id)?;
            if width > 1 && height > 1 {
                let attr = conn.get_window_attributes(window)?.reply()?;
                if let MapState::VIEWABLE = attr.map_state {
                    result.push(window as WindowId);
                } else {
                    debug!(
                        "Window {} with {} x {} is unmapped",
                        window_id, width, height
                    );
                }
            } else {
                debug!("Window {} with {} x {}", window_id, width, height);
            }
            let mut sub_windows = self.get_all_sub_windows(&window_id)?;
            result.append(&mut sub_windows);
        }

        Ok(result)
    }

    pub fn get_visible_windows(&self) -> Result<Vec<WindowId>> {
        let screen = self.screen();
        self.get_all_sub_windows(&(screen.root as WindowId))
    }

    pub fn get_window_geometry(&self, window: &WindowId) -> Result<(i32, i32, u32, u32)> {
        let conn = &self.conn;
        let window = *window as Window;
        let geom = conn.get_geometry(window)?.reply()?;
        Ok((geom.x as _, geom.y as _, geom.width as _, geom.height as _))
    }
}

impl PlatformApi for X11Api {
    /// 1. error if no screenshot is capture-able
    /// 2. it checks for transparent margins and configures the api
    ///     to cut them away in further screenshots
    fn calibrate(&mut self, window_id: WindowId) -> Result<()> {
        let image = self.capture_window_screenshot(window_id)?;
        self.margin = identify_transparency(&image)?;

        Ok(())
    }

    fn window_list(&self) -> Result<WindowList> {
        let windows = self.get_visible_windows()?;
        let mut wins = vec![];
        for window in windows {
            if let Ok(Some(name)) = self.get_window_name(&window) {
                let name = if let Ok((_, _, w, h)) = self.get_window_geometry(&window) {
                    format!("{} ({}x{})", name, w, h)
                } else {
                    name
                };
                wins.push((Some(name), window as u64));
            }
        }

        Ok(wins)
    }

    fn capture_window_screenshot(&self, window_id: WindowId) -> Result<Frame> {
        let (_, _, mut width, mut height) = self.get_window_geometry(&window_id)?;
        let (mut x, mut y) = (0_i32, 0_i32);
        if let Some(margin) = self.margin.as_ref() {
            if !margin.is_zero() {
                width -= (margin.left + margin.right) as u32;
                height -= (margin.top + margin.bottom) as u32;
                x = margin.left as _;
                y = margin.top as _;
            }
        }
        let image = self
            .conn
            // NOTE: x and y are not the absolute coordinates but relative to the windows dimensions, that is why 0, 0
            .get_image(
                ImageFormat::Z_PIXMAP,
                window_id as Drawable,
                x as _,
                y as _,
                width as _,
                height as _,
                !0,
            )?
            .reply()
            .context(format!(
                "Cannot fetch the image data for window {}",
                window_id
            ))?;

        let mut samples = image.data;
        convert_bgra_to_rgba(&mut samples);

        if image.depth == 24 {
            let stride = 3;
            // NOTE: in this case the alpha channel is 0, but should be set to 0xff
            // the index into the alpha channel
            let mut i = stride;
            let len = samples.len();
            while i < len {
                let alpha = samples.get_mut(i).unwrap();
                if alpha == &0 {
                    *alpha = 0xff;
                } else {
                    // NOTE: the assumption here is, if one pixel is fine, then all might be fine :)
                    break;
                }

                // going one pixel further, still pointing to the alpha channel
                i += stride;
            }
        }
        if self.margin.is_some() {
            let stride = 3;
            // once first image is captured, we make sure that transparency is removed
            // even in cases where `margin.is_zero()`
            let mut i = 3;
            let len = samples.len();
            while i < len {
                let alpha = samples.get_mut(i).unwrap();
                if alpha != &0xff {
                    *alpha = 0xff;
                }

                // going one pixel further, still pointing to the alpha channel
                i += stride;
            }
        }
        debug!("Image dimensions: {}x{}", width, height);

        let channels = 4;
        Ok(Frame::from_bgra(samples, channels, width, height))
        // let color = ColorType::Rgba8;
        // let mut buffer = FlatSamples {
        //     samples,
        //     layout: SampleLayout::row_major_packed(channels, width as u32, height as u32),
        //     color_hint: Some(color),
        // };
        // Ok(ImageOnHeap::new(buffer))
    }

    fn get_active_window(&self) -> Result<WindowId> {
        let screen = self.screen();
        let conn = &self.conn;
        let atoms = &self.atoms;
        let prop = conn
            .get_property(
                false,
                screen.root,
                atoms._NET_ACTIVE_WINDOW,
                AtomEnum::WINDOW,
                0,
                u32::MAX,
            )?
            .reply()?;
        let window = prop
            .value32()
            .context(
                "Window Manager does not have an active window property (NET_ACTIVE_WINDOW) set.",
            )?
            .next()
            .unwrap();

        Ok(window as WindowId)
    }
}

#[cfg(feature = "e2e_tests")]
#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::IMG_EXT;
    use image::flat::View;
    use image::{save_buffer, GenericImageView, Rgba};

    #[test]
    fn calibrate() -> Result<()> {
        let mut api = X11Api::new()?;
        let win = api.get_active_window()?;
        let image_raw = api.capture_window_screenshot(win)?;
        let image: View<_, Rgba<u8>> = image_raw.as_view().unwrap();
        let (width, height) = image.dimensions();

        api.calibrate(win)?;
        let image_calibrated_raw = api.capture_window_screenshot(win)?;
        let image_calibrated: View<_, Rgba<u8>> = image_calibrated_raw.as_view().unwrap();
        let (width_new, height_new) = image_calibrated.dimensions();
        dbg!(width, width_new, height, height_new);

        let Rgba([_, _, _, alpha]) = image.get_pixel(width / 2, 0);
        dbg!(alpha);
        if alpha == 0 {
            // if that pixel was full transparent, for example on ubuntu / GNOME, caused by the drop shadow
            // then we expect the calibrated image to be smaller and cropped by this area
            assert!(api.margin.is_some());
            assert!(!api.margin.as_ref().unwrap().is_zero());
            assert!(height > height_new);
            assert!(width > width_new);
        } else {
            assert!(height >= height_new);
            assert!(width >= width_new);
        }

        // Note: visual validation is sometimes helpful:
        // save_buffer(
        //     format!("frame-raw-{}.tga", win),
        //     &image_raw.samples,
        //     image_raw.layout.width,
        //     image_raw.layout.height,
        //     image_raw.color_hint.unwrap(),
        // )
        // .context("Cannot save a frame.")?;
        //
        // save_buffer(
        //     format!("frame-calibrated-{}.tga", win),
        //     &image_calibrated_raw.samples,
        //     image_calibrated_raw.layout.width,
        //     image_calibrated_raw.layout.height,
        //     image_calibrated_raw.color_hint.unwrap(),
        // )
        // .context("Cannot save a frame.")?;

        Ok(())
    }

    /// reason for this test is the strange tree of windows:
    /// ```sh
    /// $ xwininfo -root -tree -int
    ///
    /// xwininfo: Window id: 106 (the root window) (has no name)
    ///
    ///   Root window id: 106 (the root window) (has no name)
    ///   Parent window id: 0 (none)
    ///      32 children:
    ///      44040208 "d34dl0ck@bottled-logos: /media/psf/Home/workspaces/terminal-recorder/final/t-rec": ("gnome-terminal-server" "Gnome-terminal")  1020x965+551+209  +551+209
    ///         1 child:
    ///         44040209 (has no name): ()  1x1+-1+-1  +550+208
    /// ```
    ///
    /// where you can see that the main terminal, has a child that has very strange dimensions
    ///
    #[test]
    fn should_inspect_screenshots() -> Result<()> {
        let api = X11Api::new()?;
        let win = api.get_active_window()?;
        let image_raw = api.capture_window_screenshot(win)?;
        let image: View<_, Rgba<u8>> = image_raw.as_view().unwrap();
        let (width, height) = image.dimensions();

        let Rgba([red, green, blue, alpha]) = image.get_pixel(width / 2, height / 2);
        assert_ne!(blue, 0);
        assert_ne!(green, 0);
        assert_ne!(red, 0);
        assert_ne!(alpha, 0, "alpha is unexpected");

        // Note: visual validation is sometimes helpful:
        let file = format!("frame-{win}.{IMG_EXT}");
        save_buffer(
            file,
            &image_raw.samples,
            image_raw.layout.width,
            image_raw.layout.height,
            image_raw.color_hint.unwrap(),
        )
        .context("Cannot save a frame.")?;

        Ok(())
    }

    #[test]
    fn should_instantiate_a_new_api() -> Result<()> {
        let _api = X11Api::new()?;

        Ok(())
    }

    #[test]
    fn should_always_have_an_active_window() -> Result<()> {
        let api = X11Api::new()?;
        let window = api.get_active_window();

        assert!(window.is_ok(), "Active window was not set.");
        Ok(())
    }

    #[test]
    fn should_list_current_active_window() -> Result<()> {
        let api = X11Api::new()?;
        let windows = api.get_visible_windows()?;
        assert!(!windows.is_empty(), "Window list should never be empty!");
        let window = api.get_active_window()?;
        assert!(
            windows.contains(&window),
            "Active window was not found in visible list"
        );
        for win in windows {
            let name = api.get_window_name(&win)?;
            assert!(name.is_some(), "A window should always have a name");
        }

        let name = api.get_window_name(&window)?;
        if let Some(name) = name {
            assert!(!name.is_empty());
            println!("Active window: {:?}", name);
        } else {
            panic!("this should not have happened");
        }

        Ok(())
    }

    #[test]
    fn should_list_all_visible_windows() -> Result<()> {
        let api = X11Api::new()?;
        let windows = api.get_visible_windows()?;
        assert!(!windows.is_empty());
        for win in windows {
            let (_, _, width, height) = api.get_window_geometry(&win)?;
            assert!(width > 1);
            assert!(height > 1);
        }

        Ok(())
    }

    #[test]
    fn should_get_window_name_and_class() -> anyhow::Result<()> {
        let api = X11Api::new()?;
        let tree = api.get_visible_windows()?;
        for input in tree {
            let _input = input as WindowId;
            // println!("Window {}:", input);
            // println!("  - class: {:?}", api.get_window_class(&input)?);
            // println!("  - name: {:?}", api.get_window_name(&input)?);
            // println!("  - name: {:?}", api.get_window_type(&input)?);
        }
        Ok(())
    }

    #[test]
    fn should_demonstrate_the_x11rb_capabilities() -> anyhow::Result<()> {
        let (conn, screen_num) = x11rb::connect(None)?;
        let screen = &conn.setup().roots[screen_num];
        let win = screen.root;
        let tree = conn.query_tree(win)?.reply()?;
        for win in tree.children {
            let attr = conn.get_window_attributes(win)?.reply()?;

            if let MapState::VIEWABLE = attr.map_state {
                let geometry = conn.get_geometry(win)?.reply()?;

                let class = conn
                    .get_property(
                        false,
                        win,
                        AtomEnum::WM_CLASS,
                        AtomEnum::STRING,
                        0,
                        u32::MAX,
                    )?
                    .reply()?;
                let _class = String::from_utf8(class.value)?;

                for prop in &["_NET_WM_ICON_NAME", "_NET_WM_NAME", "_NET_WM_VISIBLE_NAME"] {
                    let wm_name_atom = conn.intern_atom(true, prop.as_bytes())?.reply()?;
                    let utf8_atom = conn.intern_atom(true, b"UTF8_STRING")?.reply()?;
                    let name = conn
                        .get_property(false, win, wm_name_atom.atom, utf8_atom.atom, 0, u32::MAX)?
                        .reply()?;
                    let _name = String::from_utf8(name.value)?;
                    // assert!(name.len() >= 0);
                }

                assert!(geometry.width > 0);
                assert!(geometry.height > 0);
            }
        }

        Ok(())
    }
}
