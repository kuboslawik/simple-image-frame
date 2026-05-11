use clap::Parser;
use raylib::prelude::*;

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
    #[arg(short, long, default_values = &["/home/kuba/Obrazy/1.jpg", "/home/kuba/Obrazy/2.jpg", "/home/kuba/Obrazy/3.jpg", "/home/kuba/Obrazy/4.jpg", "/home/kuba/Obrazy/5.jpg",])]
    pictures_list: Vec<String>,
}

const SCREEN_W: i32 = 1024;
const SCREEN_H: i32 = 768;

fn main() {
    let args = Args::parse();

    let (mut rl, thread) = raylib::init()
        .size(SCREEN_W, SCREEN_H)
        .title("Hello, World")
        .build();

    rl.set_target_fps(60);

    let mut image_loader = image_loader::ImageLoaderWorker::build(3, args.pictures_list);
    image_loader.start_worker();

    let mut active_texture: Option<Texture2D> = None;
    let mut next_texture: Option<Texture2D> = None;
    let mut last_update = std::time::Instant::now();

    while !rl.window_should_close() {
        if active_texture.is_none() || last_update.elapsed().as_secs() >= args.display_time as u64 {
            if let Some(prepared) = image_loader.get_next_image() {
                let new_texture = rl
                    .load_texture_from_image(&thread, &prepared.image)
                    .unwrap();
                active_texture = Some(new_texture);
                last_update = std::time::Instant::now();
            }
        }

        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::WHITE);
        d.draw_text(&args.display_time.to_string(), 12, 12, 20, Color::BLACK);

        if let Some(ref tex) = active_texture {
            d.draw_texture(tex, 0, 0, Color::WHITE);
        }

        d.draw_text(
            &format!(
                "Next change in: {}s",
                args.display_time as i64 - last_update.elapsed().as_secs() as i64
            ),
            12,
            12,
            20,
            Color::RED,
        );
    }
}
