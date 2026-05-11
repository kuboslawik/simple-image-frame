use raylib::prelude::Image;

pub struct PreparedImage {
    pub image: Image,
    pub path: String,
    pub date: Option<String>,
}

unsafe impl Send for PreparedImage {}
unsafe impl Sync for PreparedImage {}

impl PreparedImage {
    pub fn new(path: &str) -> Result<Self, String> {
        let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
        let mut bufreader = std::io::BufReader::new(&file);
        let exifreader = exif::Reader::new();
        let exif_data = exifreader.read_from_container(&mut bufreader).ok();

        let date = exif_data.as_ref().and_then(|exif| {
            exif.get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY)
                .map(|f| f.display_value().with_unit(exif).to_string())
        });

        let image = Image::load_image(path).map_err(|e| e.to_string())?;

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
