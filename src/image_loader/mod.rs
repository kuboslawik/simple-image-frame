pub mod prepared_image;

//Raylib
use prepared_image::PreparedImage;

// Threading
use std::sync::{Arc, Mutex};
use std::{thread, time::Duration};

pub struct ImageLoaderWorker {
    state: Arc<Mutex<LoaderInnerStructure>>,
}

struct LoaderInnerStructure {
    cache_size: usize,
    cache: Vec<PreparedImage>,
    paths: Vec<String>,
    current_index: usize,
    finished: bool,
    target_width: i32,
    target_height: i32,
}

impl ImageLoaderWorker {
    pub fn build(
        cache_size: usize,
        paths: Vec<String>,
        target_width: i32,
        target_height: i32,
    ) -> Self {
        let inner_structure = LoaderInnerStructure {
            cache_size,
            cache: Vec::with_capacity(cache_size),
            paths,
            current_index: 0,
            finished: false,
            target_width,
            target_height,
        };
        Self {
            state: Arc::new(Mutex::new(inner_structure)),
        }
    }

    pub fn start_worker(&self) {
        let state = Arc::clone(&self.state);

        thread::spawn(move || {
            loop {
                let (path, should_load, tw, th) = {
                    let mut inner_structure = state.lock().unwrap();
                    if inner_structure.current_index >= inner_structure.paths.len() {
                        inner_structure.finished = true;
                        return;
                    } else if inner_structure.cache.len() < inner_structure.cache_size
                        && !inner_structure.paths.is_empty()
                    {
                        (
                            Some(inner_structure.paths[inner_structure.current_index].clone()),
                            true,
                            inner_structure.target_width,
                            inner_structure.target_height,
                        )
                    } else {
                        (None, false, 0, 0)
                    }
                };

                if should_load && path.is_some() {
                    let p = path.unwrap();
                    match PreparedImage::new(&p, tw, th) {
                        Ok(img) => {
                            let mut inner_structure = state.lock().unwrap();
                            inner_structure.cache.push(img);
                            inner_structure.current_index = (inner_structure.current_index + 1);
                        }
                        Err(e) => {
                            println!("Error loading image {}: {}", p, e);
                            let mut inner_structure = state.lock().unwrap();
                            inner_structure.current_index = (inner_structure.current_index + 1);
                        }
                    }
                } else {
                    thread::sleep(Duration::from_millis(100));
                }
            }
        });
    }

    pub fn get_next_image(&self) -> Option<PreparedImage> {
        let mut inner_structure = self.state.lock().unwrap();
        if !inner_structure.cache.is_empty() {
            Some(inner_structure.cache.remove(0))
        } else {
            None
        }
    }
}
