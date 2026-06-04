use raylib::prelude::Image;
use raylib::prelude::Rectangle;
use std::f32;
use std::io::{Cursor, Read};

pub struct PreparedImage {
    pub image: Image,
    pub date: Option<String>,
}

unsafe impl Send for PreparedImage {}
unsafe impl Sync for PreparedImage {}

impl PreparedImage {
    pub fn new(path: &str, target_width: i32, target_height: i32) -> Result<Self, String> {
        let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;

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

        let file_extension = std::path::Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| format!(".{}", s))
            .unwrap_or_else(|| ".png".to_string());

        let mut image =
            Image::load_image_from_mem(&file_extension, &buffer).map_err(|e| e.to_string())?;

        match orientation {
            Some(x) => match x {
                3 => {
                    image.rotate_cw();
                    image.rotate_cw();
                }
                6 => {
                    image.rotate_cw();
                }
                8 => {
                    image.rotate_ccw();
                }
                _ => {}
            },
            None => {}
        }

        let scale: f32 = f32::min(
            target_width as f32 / image.width() as f32,
            target_height as f32 / image.height() as f32,
        );

        if scale <= 1.0 {
            let new_width = (image.width as f32 * scale) as i32;
            let new_height = (image.height as f32 * scale) as i32;

            image.resize(new_width, new_height);
        }

        //Cropping 2 pixels due to FKMS bug on rasperry pi zero 2w

        let crop_rectangle = Rectangle {
            x: 1.0,
            y: 1.0,
            width: target_width as f32 - 2.0,
            height: target_height as f32 - 2.0,
        };

        image.crop(crop_rectangle);

        Ok(Self { image, date })
    }
}
