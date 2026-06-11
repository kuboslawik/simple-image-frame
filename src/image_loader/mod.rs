pub mod prepared_image;

use self::prepared_image::PreparedImage;

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
    current_repeat_count: usize,
    max_repeat_count: usize,
    finished: bool,
    target_width: i32,
    target_height: i32,
}

impl ImageLoaderWorker {
    pub fn build(
        cache_size: usize,
        paths: Vec<String>,
        max_repeat_count: usize,
        target_width: i32,
        target_height: i32,
    ) -> Self {
        let inner_structure = LoaderInnerStructure {
            cache_size,
            cache: Vec::with_capacity(cache_size),
            paths,
            current_index: 0,
            current_repeat_count: 0,
            max_repeat_count,
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
                let (path, tw, th) = {
                    let mut inner_structure = state.lock().unwrap();

                    if inner_structure.current_repeat_count >= inner_structure.max_repeat_count {
                        inner_structure.finished = true;
                        return;
                    }

                    if inner_structure.cache.len() >= inner_structure.cache_size {
                        drop(inner_structure);
                        thread::sleep(Duration::from_millis(100));
                        continue;
                    }

                    let p = inner_structure.paths[inner_structure.current_index].clone();
                    let tw = inner_structure.target_width;
                    let th = inner_structure.target_height;

                    inner_structure.current_index += 1;
                    if inner_structure.current_index >= inner_structure.paths.len() {
                        inner_structure.current_index = 0;
                        inner_structure.current_repeat_count += 1;
                    }

                    (p, tw, th)
                };

                match PreparedImage::new(&path, tw, th) {
                    Ok(img) => {
                        let mut inner_structure = state.lock().unwrap();
                        inner_structure.cache.push(img);
                    }
                    Err(e) => {
                        println!("Error loading image {}: {}", path, e);
                    }
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

    pub fn is_finished(&self) -> bool {
        let inner = self.state.lock().unwrap();
        inner.finished && inner.cache.is_empty()
    }
}
