use raylib::prelude::Image;


pub struct PreparedImage {
    pub image: Image,
    pub path: String,
    pub date: Option<String>,
}

impl PreparedImage {
    pub fn new(path: &str) -> Result<Self, String> {
   
        let image = Image::load_image(path).map_err(|e| e.to_string())?;

        Ok(Self {
            image,
            path: path.to_string(),
            date: None,
        })
    }
}