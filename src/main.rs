use raylib::prelude::*;
use clap::Parser;

mod image_loader;

/// Simple program to show images slideshow
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Time between photos
    #[arg(short, long, default_value_t = 20)]
    display_time: u32,

    /// Full time of the slideshow
    #[arg(short, long, default_value_t = 5400)]
    full_time: u32,

    /// List of the pictures
    #[arg(short, long, default_values = &["/home/kuba/Obrazy/1.jpg", "/home/kuba/Obrazy/2.jpg"])]
    pictures_list: Vec<String>
}

const SCREEN_W :i32 = 1024;
const SCREEN_H :i32 = 768;

fn main() {

    let args = Args::parse();

    let (mut rl, thread) = raylib::init()
        .size(SCREEN_W, SCREEN_H)
        .title("Hello, World")
        .build();

    let mut image_loader = image_loader::ImageLoaderWorker::build(3);
    
    for path in &args.pictures_list {
        image_loader.load_image(path);
    }

    for (i, img) in image_loader.cache.iter().enumerate() {
        println!("Picture {}: {} | Taken at: {}", i, img.path, img.date_str());
    }

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        let cache_size_string= String::from(image_loader.cache_size.to_string());

        d.clear_background(Color::WHITE);
        d.draw_text(&args.display_time.to_string(), 12, 12, 20, Color::BLACK);
        d.draw_text(&cache_size_string, 21, 21, 20, Color::BLACK);
    }
}