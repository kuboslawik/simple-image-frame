pub mod prepared_image;

use prepared_image::PreparedImage;

pub struct ImageLoaderWorker {
    pub cache_size: usize,
    pub cache: Vec<PreparedImage>,
    pub ready: bool
}

impl ImageLoaderWorker {
    pub fn build(v_cache_size: usize) -> Self {
        Self{
            cache_size: v_cache_size,
            cache: Vec::with_capacity(v_cache_size),
            ready: true
        }
    }
    
    pub fn load_image(&mut self, path: &str) {
        match prepared_image::PreparedImage::new(path) {
            Ok(p) => self.cache.push(p),
            Err(e) => println!("Error: {}", e),
        }
    }

    pub fn print_cache_size(&self) -> String {
        format!("Cache size is {}", self.cache_size)
    }    
}