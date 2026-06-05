use image::GenericImageView;
use std::io::Cursor;

pub struct PreparedImage {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub date: Option<String>,
}

impl PreparedImage {
    pub fn new(path: &str, target_width: i32, target_height: i32) -> Result<Self, String> {
        let buffer = std::fs::read(path).map_err(|e| e.to_string())?;

        let mut bufreader = Cursor::new(&buffer);
        let exifreader = exif::Reader::new();
        let exif_data = exifreader.read_from_container(&mut bufreader).ok();

        let date = exif_data.as_ref().and_then(|exif| {
            exif.get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY)
                .map(|f| f.display_value().with_unit(exif).to_string())
        });

        let orientation = exif_data.as_ref().and_then(|exif| {
            exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY)
                .and_then(|f| f.value.get_uint(0))
        });

        let mut img = image::load_from_memory(&buffer).map_err(|e| e.to_string())?;

        match orientation {
            Some(x) => match x {
                3 => img = img.rotate180(),
                6 => img = img.rotate90(),
                8 => img = img.rotate270(),
                _ => {}
            },
            None => {}
        }

        let scale: f32 = f32::min(
            target_width as f32 / img.width() as f32,
            target_height as f32 / img.height() as f32,
        );

        if scale <= 1.0 {
            let new_width = (img.width() as f32 * scale) as u32;
            let new_height = (img.height() as f32 * scale) as u32;
            img = img.resize(new_width, new_height, image::imageops::FilterType::Triangle);
        }

        // Cropping 2 pixels due to FKMS bug on rasperry pi zero 2w
        let (w, h) = img.dimensions();
        let cropped = img.crop_imm(1, 1, w.saturating_sub(2), h.saturating_sub(2));
        let (final_w, final_h) = cropped.dimensions();
        let pixels = cropped.to_rgba8().into_raw();

        Ok(Self {
            pixels,
            width: final_w,
            height: final_h,
            date,
        })
    }
}
