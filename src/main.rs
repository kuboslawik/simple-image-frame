use clap::Parser;
use macroquad::prelude::*;
use std::time::Instant;

mod image_loader;
use image_loader::ImageLoaderWorker;

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

#[macroquad::main("simple-image-frame")]
async fn main() {
    let args = Args::parse();

    let target_w = screen_width() as i32;
    let target_h = screen_height() as i32;

    let image_loader = ImageLoaderWorker::build(3, args.pictures_list, target_w, target_h);
    image_loader.start_worker();

    let font =
        load_ttf_font_from_bytes(include_bytes!("../fonts/digital-7.ttf")).expect("Font not found");

    let mut current_texture: Option<Texture2D> = None;
    let mut old_texture: Option<Texture2D> = None;
    let mut current_exif_text = String::new();

    let mut timer = args.display_time as f32;
    let mut transition_alpha = 1.0f32;
    let mut is_transitioning = false;
    let fade_speed = 2.0f32;
    let slideshow_start = Instant::now();

    loop {
        let dt = get_frame_time().min(0.1);

        if slideshow_start.elapsed().as_secs() >= args.full_time as u64 {
            break;
        }

        if is_key_pressed(KeyCode::Escape) {
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

                let tex = Texture2D::from_rgba8(
                    prepared.width as u16,
                    prepared.height as u16,
                    &prepared.pixels,
                );

                current_texture = Some(tex);
                current_exif_text = prepared.date.clone().unwrap_or_default();
                transition_alpha = 0.0;
                is_transitioning = true;
            } else if image_loader.is_finished() {
                break;
            }
        }

        clear_background(BLACK);

        if is_transitioning {
            if let Some(ref tex) = old_texture {
                draw_tex(tex, 1.0 - transition_alpha);
            }
            if let Some(ref tex) = current_texture {
                draw_tex(tex, transition_alpha);
            }
        } else if let Some(ref tex) = current_texture {
            draw_tex(tex, 1.0);
        }

        if !current_exif_text.is_empty() {
            let h = screen_height();
            let params = TextParams {
                font: Some(&font),
                font_size: 20,
                ..Default::default()
            };

            draw_text_ex(
                &current_exif_text,
                22.0,
                h - 38.0,
                TextParams {
                    color: BLACK,
                    ..params
                },
            );
            draw_text_ex(
                &current_exif_text,
                20.0,
                h - 40.0,
                TextParams {
                    color: WHITE,
                    ..params
                },
            );
        }

        next_frame().await;
    }
}

fn draw_tex(tex: &Texture2D, alpha: f32) {
    let sw = screen_width();
    let sh = screen_height();
    let s = f32::min((sw - 4.0) / tex.width(), (sh - 4.0) / tex.height());

    let w = tex.width() * s;
    let h = tex.height() * s;
    let x = (sw - w) / 2.0;
    let y = (sh - h) / 2.0;

    draw_texture_ex(
        tex,
        x,
        y,
        Color::new(1.0, 1.0, 1.0, alpha),
        DrawTextureParams {
            dest_size: Some(vec2(w, h)),
            ..Default::default()
        },
    );
}
