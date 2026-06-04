use clap::Parser;
use raylib::prelude::*;

mod image_loader;

/// Simple program to show images slideshow
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Time between photos
    #[arg(short, long, default_value_t = 3)]
    display_time: u32,

    /// Full time of the slideshow
    #[arg(short, long, default_value_t = 5400)]
    full_time: u32,

    /// List of the pictures
    #[arg(short, long, required = true, num_args = 1..)]
    pictures_list: Vec<String>,
}

fn main() {
    let args = Args::parse();

    let (mut rl, thread) = raylib::init()
        .size(0, 0)
        .fullscreen()
        .title("Simple Image Slideshow")
        .build();

    rl.set_target_fps(20);

    let target_w = rl.get_screen_width();
    let target_h = rl.get_screen_height();

    let image_loader =
        image_loader::ImageLoaderWorker::build(3, args.pictures_list, target_w, target_h);
    image_loader.start_worker();

    let font = rl
        .load_font(&thread, "./fonts/digital-7.ttf")
        .expect("ERROR: Font file digital-7.ttf not found in project fonts folder");

    let mut current_texture: Option<Texture2D> = None;
    let mut old_texture: Option<Texture2D> = None;
    let mut current_exif_text = String::new();

    let mut timer = args.display_time as f32;
    let mut transition_alpha = 1.0f32;
    let mut is_transitioning = false;
    let fade_speed = 2.0f32;
    let slideshow_start = std::time::Instant::now();

    while !rl.window_should_close() {
        let mut dt = rl.get_frame_time();
        if dt > 0.1 {
            dt = 0.1;
        }

        if slideshow_start.elapsed().as_secs() >= args.full_time as u64 {
            break;
        }

        if is_transitioning {
            transition_alpha += dt * fade_speed;
            if transition_alpha >= 1.0 {
                transition_alpha = 1.0;
                is_transitioning = false;
                timer = 0.0;
                old_texture = None;
            }
        } else {
            timer += dt;
        }

        if timer >= args.display_time as f32 && !is_transitioning {
            if let Some(prepared) = image_loader.get_next_image() {
                old_texture = current_texture.take();
                let tex = rl
                    .load_texture_from_image(&thread, &prepared.image)
                    .unwrap();
                current_texture = Some(tex);
                current_exif_text = prepared.date.clone().unwrap_or_default();
                transition_alpha = 0.0;
                is_transitioning = true;
            } else if image_loader.is_finished() {
                break;
            }
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);

        if is_transitioning {
            if let Some(ref tex) = old_texture {
                draw_tex(&mut d, tex, 1.0 - transition_alpha);
            }
            if let Some(ref tex) = current_texture {
                draw_tex(&mut d, tex, transition_alpha);
            }
        } else if let Some(ref tex) = current_texture {
            draw_tex(&mut d, tex, 1.0);
        }

        if !current_exif_text.is_empty() {
            let h = d.get_screen_height() as f32;
            let pos_shadow = Vector2::new(22.0, h - 38.0);
            let pos_text = Vector2::new(20.0, h - 40.0);

            d.draw_text_ex(
                &font,
                &current_exif_text,
                pos_shadow,
                20.0,
                2.0,
                Color::BLACK,
            );
            d.draw_text_ex(
                &font,
                &current_exif_text,
                pos_text,
                20.0,
                2.0,
                Color::RAYWHITE,
            );
        }
    }
}

fn draw_tex(d: &mut RaylibDrawHandle, tex: &Texture2D, alpha: f32) {
    let sw = d.get_screen_width() as f32;
    let sh = d.get_screen_height() as f32;
    let s = f32::min(
        (sw - 4.0) / tex.width() as f32,
        (sh - 4.0) / tex.height() as f32,
    );
    let dest = Rectangle::new(
        ((sw - tex.width() as f32 * s) / 2.0).round(),
        ((sh - tex.height() as f32 * s) / 2.0).round(),
        (tex.width() as f32 * s).round(),
        (tex.height() as f32 * s).round(),
    );
    let src = Rectangle::new(0.0, 0.0, tex.width() as f32, tex.height() as f32);
    d.draw_texture_pro(
        tex,
        src,
        dest,
        Vector2::new(0.0, 0.0),
        0.0,
        Color::WHITE.alpha(alpha),
    );
}
