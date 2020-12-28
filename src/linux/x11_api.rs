use crate::linux::Margin;
use crate::{ImageOnHeap, PlatformApi, Result, WindowId, WindowList};
use anyhow::Context;
use image::flat::{SampleLayout, View};
use image::{Bgra, ColorType, FlatSamples, GenericImageView};
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

        Ok(if name.is_empty() { Some(name) } else { None })
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
                if let MapState::Viewable = attr.map_state {
                    result.push(window as WindowId);
                }
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

    pub fn get_window_geometry(&self, window: &WindowId) -> Result<(i16, i16, u16, u16)> {
        let conn = &self.conn;
        let window = *window as Window;
        let geom = conn.get_geometry(window)?.reply()?;
        Ok((geom.x, geom.y, geom.width, geom.height))
    }
}

impl PlatformApi for X11Api {
    /// 1. it does check for the screenshot
    /// 2. it checks for transparent margins and configures the api
    ///     to cut them away in further screenshots
    fn calibrate(&mut self, window_id: WindowId) -> Result<()> {
        let image = self.capture_window_screenshot(window_id)?;
        let image: View<_, Bgra<u8>> = image.as_view()?;
        let (width, height) = image.dimensions();
        let half_width = width / 2;
        let half_height = height / 2;

        let mut margin = Margin::zero();
        // identify top margin
        for y in 0..half_height {
            let Bgra([_, _, _, a]) = image.get_pixel(half_width, y);
            if a == 0xff {
                // the end of the transparent area
                margin.top = y as u16;
                dbg!(margin.top);
                break;
            }
        }
        // identify bottom margin
        for y in (half_height..height).rev() {
            let Bgra([_, _, _, a]) = image.get_pixel(half_width, y);
            if a == 0xff {
                // the end of the transparent area
                margin.bottom = (height - y - 1) as u16;
                dbg!(margin.bottom);
                break;
            }
        }
        // identify left margin
        for x in 0..half_width {
            let Bgra([_, _, _, a]) = image.get_pixel(x, half_height);
            if a == 0xff {
                // the end of the transparent area
                margin.left = x as u16;
                dbg!(margin.left);
                break;
            }
        }
        // identify right margin
        for x in (half_width..width).rev() {
            let Bgra([_, _, _, a]) = image.get_pixel(x, half_height);
            if a == 0xff {
                // the end of the transparent area
                margin.right = (width - x - 1) as u16;
                dbg!(margin.right);
                break;
            }
        }
        self.margin = if margin.is_zero() { None } else { Some(margin) };

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

    fn capture_window_screenshot(&self, window_id: WindowId) -> Result<ImageOnHeap> {
        let (_, _, mut width, mut height) = self.get_window_geometry(&window_id)?;
        let (mut x, mut y) = (0_i16, 0_i16);
        if self.margin.is_some() {
            let margin = self.margin.as_ref().unwrap();
            width -= margin.left + margin.right;
            height -= margin.top + margin.bottom;
            x = margin.left as i16;
            y = margin.top as i16;
        }
        let image = self
            .conn
            // NOTE: x and y are not the absolute coordinates but relative to the windows dimensions, that is why 0, 0
            .get_image(
                ImageFormat::ZPixmap,
                window_id as Drawable,
                x,
                y,
                width,
                height,
                !0,
            )?
            .reply()
            .context(format!(
                "Cannot fetch the image data for window {}",
                window_id
            ))?;

        let raw_data = image.data;
        let color = ColorType::Bgra8;
        let channels = 4;
        let mut buffer = FlatSamples {
            samples: raw_data,
            layout: SampleLayout::row_major_packed(channels, width as u32, height as u32),
            color_hint: Some(color),
        };

        // NOTE: in this case the alpha channel is 0, but should be set to 0xff
        if image.depth == 24 {
            // the index into the alpha channel
            let mut i = 3;
            let len = buffer.samples.len();
            while i < len {
                let alpha = buffer.samples.get_mut(i).unwrap();
                if alpha == &0 {
                    *alpha = 0xff;
                } else {
                    // NOTE: the assumption here is, if one pixel is fine, then all might be fine :)
                    break;
                }

                // going one pixel further, still pointing to the alpha channel
                i += buffer.layout.width_stride;
            }
        }

        Ok(ImageOnHeap::new(buffer))
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
        let window = prop.value32().unwrap().next().unwrap();

        Ok(window as WindowId)
    }
}

#[cfg(feature = "test_against_real_display")]
#[cfg(test)]
mod test {
    use super::*;
    use image::flat::View;
    use image::{Bgra, GenericImageView};

    #[test]
    fn calibrate() -> Result<()> {
        let mut api = X11Api::new()?;
        let win = api.get_active_window()?;
        let image = api.capture_window_screenshot(win)?;
        let image: View<_, Bgra<u8>> = image.as_view().unwrap();
        let (width, height) = image.dimensions();

        api.calibrate(win)?;
        let image_calibrated = api.capture_window_screenshot(win)?;
        let image_calibrated: View<_, Bgra<u8>> = image_calibrated.as_view().unwrap();
        let (width_new, height_new) = image_calibrated.dimensions();

        assert!(height >= height_new);
        assert!(width >= width_new);

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
        let image = api.capture_window_screenshot(win)?;
        let image: View<_, Bgra<u8>> = image.as_view().unwrap();

        let Bgra([b, g, r, a]) = image.get_pixel(10, 25);
        assert_ne!(b, 0);
        assert_ne!(g, 0);
        assert_ne!(r, 0);
        assert_eq!(a, 0xff);

        // Note: visual validation is sometimes helpful:
        // let file = format!("/tmp/foo-bar-{}.tga", win);
        // save_buffer(
        //     file.clone(),
        //     &image.samples,
        //     image.layout.width,
        //     image.layout.height,
        //     image.color_hint.unwrap(),
        // )?;
        Ok(())
    }

    #[test]
    fn should_instantiate_a_new_api() -> Result<()> {
        let _api = X11Api::new()?;

        Ok(())
    }

    #[test]
    fn should_list_current_active_window() -> Result<()> {
        let api = X11Api::new()?;
        let windows = api.get_visible_windows()?;
        let window = api.get_active_window()?;

        assert!(!windows.is_empty());
        assert!(
            windows.contains(&window),
            "Active window was not found in visible list"
        );
        let name = api.get_window_name(&window)?;
        if let Some(name) = name {
            assert!(!name.is_empty());
            println!("Active window: {:?}", name);
        } else {
            eprintln!("The active window has no name :(");
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

            if let MapState::Viewable = attr.map_state {
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
