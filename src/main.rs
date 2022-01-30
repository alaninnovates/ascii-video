use std::fs;
use std::process::Command;
use std::thread;
use std::time::Duration;
use image::{DynamicImage, GenericImageView};
use image::imageops::FilterType;

fn image_to_ascii(image: DynamicImage, resolution: u32) -> String {
    let pallet: [char; 7] = [' ', '.', '/', '*', '#', '$', '@'];
    let mut y = 0;
    let mut art = String::new();
    let resized_image = image.resize(image.width() / resolution, image.height() / resolution, FilterType::Nearest);
    for p in resized_image.pixels() {
        if y != p.1 {
            art.push_str("\n");
            y = p.1;
        }
        let r = p.2.0[0] as f32;
        let g = p.2.0[1] as f32;
        let b = p.2.0[2] as f32;
        let k = r * 0.3 + g * 0.59 + b * 0.11;

        let character = ((k / 255.0) * (pallet.len() - 1) as f32).round() as usize;
        art.push_str(&format!("{} ", pallet[character]));
    }
    return art;
}

fn cls() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}

fn main() {
    println!("Loading frames...");
    let mut ffmpeg_cmd = Command::new("ffmpeg")
        .args(["-i", "stick_vid.mp4"])
        .args(["-r", "60"])
        .arg("frames/out-%03d.jpg")
        .spawn()
        .expect("FFmpeg errored");
    ffmpeg_cmd.wait();
    println!("Done loading frames");
    let mut frame_paths: Vec<_> = fs::read_dir("./frames").unwrap()
        .map(|r| r.unwrap())
        .collect();
    frame_paths.sort_by_key(|entry| entry.file_name());
    let mut images_ascii = Vec::new();
    println!("Rendering frames...");
    for frame_path in frame_paths {
        let image = image::open(frame_path.path().as_path());
        images_ascii.push(image_to_ascii(image.unwrap(), 20));
        println!("Rendered: {}", frame_path.path().display());
    }
    for img in images_ascii {
        print!("{}", img);
        thread::sleep(Duration::from_millis(30));
        cls();
    }
}

#[cfg(test)]
mod tests {
    use crate::image_to_ascii;

    #[test]
    fn convert_img_to_ascii() {
        let img = image::open("./test_img.png").unwrap();
        print!("{}", image_to_ascii(img, 20));
    }
}