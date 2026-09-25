//! Omarchy's desktop background, painted behind the chat while the app
//! follows the desktop's palette.
//!
//! The theme catalog (fastframe-theme) already watches Omarchy's `current/`
//! directory, so the app calls [`DesktopBackground::refresh`] whenever the
//! catalog reloads. The image is only decoded again when the file behind
//! `current/background` actually changed.

use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::{Arc, mpsc},
    time::SystemTime,
};

const MAX_BYTES: u64 = 64 * 1024 * 1024;
const MAX_SIDE: u32 = 2560;

#[derive(Clone, Debug, PartialEq)]
struct Key {
    path: PathBuf,
    modified: Option<SystemTime>,
    len: u64,
}

type Loaded = (Key, Option<Arc<egui::ColorImage>>);

#[derive(Default)]
pub struct DesktopBackground {
    key: Option<Key>,
    image: Option<Arc<egui::ColorImage>>,
    generation: u64,
    receiver: Option<mpsc::Receiver<Loaded>>,
}

impl DesktopBackground {
    /// Loads the background again if it changed. With `follows` false the
    /// background is dropped, since it belongs to the desktop's palette.
    pub fn refresh(&mut self, follows: bool, wake: impl Fn() + Send + 'static) {
        let key = if follows { current_key() } else { None };
        let Some(key) = key else {
            self.receiver = None;
            if self.key.take().is_some() || self.image.take().is_some() {
                self.generation += 1;
            }
            return;
        };
        if self.key.as_ref() == Some(&key) {
            return;
        }
        let (sender, receiver) = mpsc::channel();
        self.receiver = Some(receiver);
        std::thread::spawn(move || {
            let image = match decode(&key) {
                Ok(image) => Some(Arc::new(image)),
                Err(error) => {
                    log::warn!("unable to load the Omarchy background: {error}");
                    None
                }
            };
            if sender.send((key, image)).is_ok() {
                wake();
            }
        });
    }

    /// Takes a finished load. Call it every frame.
    pub fn poll(&mut self) {
        let Some(receiver) = &self.receiver else {
            return;
        };
        match receiver.try_recv() {
            Ok((key, image)) => {
                self.receiver = None;
                self.key = Some(key);
                self.image = image;
                self.generation += 1;
            }
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => self.receiver = None,
        }
    }

    /// The current image, with a counter that changes whenever it is
    /// replaced or removed.
    pub fn image(&self) -> (u64, Option<&Arc<egui::ColorImage>>) {
        (self.generation, self.image.as_ref())
    }
}

fn current_key() -> Option<Key> {
    let home = std::env::var_os("HOME")?;
    key_for(&Path::new(&home).join(".local/state/omarchy/current/background"))
}

/// Identifies the file without decoding it, so reloads caused by unrelated
/// theme files keep the texture already on screen.
fn key_for(link: &Path) -> Option<Key> {
    let path = fs::canonicalize(link).ok()?;
    let metadata = fs::metadata(&path).ok()?;
    metadata.is_file().then(|| Key {
        modified: metadata.modified().ok(),
        len: metadata.len(),
        path,
    })
}

/// Scaled down so it fits in one texture.
fn decode(key: &Key) -> io::Result<egui::ColorImage> {
    if key.len > MAX_BYTES {
        return Err(io::Error::other("Omarchy background is too large"));
    }
    let image = image::open(&key.path).map_err(io::Error::other)?;
    let image = if image.width().max(image.height()) > MAX_SIDE {
        image.resize(MAX_SIDE, MAX_SIDE, image::imageops::FilterType::Triangle)
    } else {
        image
    };
    let rgba = image.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    Ok(egui::ColorImage::from_rgba_unmultiplied(
        size,
        rgba.as_raw(),
    ))
}
