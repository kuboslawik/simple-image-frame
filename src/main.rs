use clap::Parser;
use macroquad::prelude::*;
use std::thread::sleep;

mod image_loader;
use image_loader::ImageLoaderWorker;

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
    #[arg(short, long, required = true, num_args = 1..)]
    pictures_list: Vec<String>,

    /// Hide photo timestamp
    #[arg(long)]
    hide_timestamp: bool,
}

fn window_conf() -> Conf {
    Conf {
        window_title: "simple-image-frame".to_owned(),
        window_width: 1024,
        window_height: 768,
        fullscreen: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let args = Args::parse();

    if args.display_time < 1 {
        println!("Display time must be equal or greater than 1");
        return;
    }

    if args.full_time < 1 {
        println!("Full slideshow time must be greater than 0");
        return;
    }

    let slideshow_repeat = (args.full_time as f32
        / (args.pictures_list.len() as f32 * args.display_time as f32))
        .ceil() as usize;

    let image_loader = ImageLoaderWorker::build(
        3,
        args.pictures_list,
        slideshow_repeat,
        screen_width() as i32,
        screen_height() as i32,
    );
    image_loader.start_worker();

    let font =
        load_ttf_font_from_bytes(include_bytes!("../fonts/digital-7.ttf")).expect("Font not found");

    let mut current_texture: Option<Texture2D> = None;
    let mut old_texture: Option<Texture2D> = None;
    let mut current_exif_text = String::new();

    let mut timer = args.display_time as f32;
    let mut transition_alpha = 1.0;
    let mut is_transitioning = false;
    let fade_speed = 2.0;

    loop {
        let mut dt = get_frame_time().min(0.1);

        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        if dt < 0.0333 {
            sleep(std::time::Duration::from_secs_f32(0.0333 - dt));
            dt = 0.0333;
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

        if !current_exif_text.is_empty() && !args.hide_timestamp {
            let h = screen_height();
            let params = TextParams {
                font: Some(&font),
                font_size: 20,
                ..Default::default()
            };

            draw_text_ex(
                &current_exif_text,
                22.0,
                h - 18.0,
                TextParams {
                    color: BLACK,
                    ..params
                },
            );
            draw_text_ex(
                &current_exif_text,
                20.0,
                h - 20.0,
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
