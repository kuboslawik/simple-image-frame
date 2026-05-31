use raylib::prelude::Image;
use std::io::{self, Cursor, Read, Seek};

pub struct PreparedImage {
    pub image: Image,
    pub path: String,
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

        //TODO: scale and resize

        image.resize(target_width, target_height);

        Ok(Self {
            image,
            path: path.to_string(),
            date,
        })
    }

    pub fn date_str(&self) -> &str {
        self.date.as_deref().unwrap_or("No data")
    }
}
