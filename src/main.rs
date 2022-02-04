use std::io::{ErrorKind, Read};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use image::{DynamicImage, GenericImageView, RgbImage};
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

// this is all we need, even though there are definitely more properties
#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct FfProbeData {
    pub streams: Vec<Stream>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Stream {
    pub width: Option<i64>,
    pub height: Option<i64>,
}

fn get_video_info(video_name: &str) -> FfProbeData {
    let ffprobe_cmd = Command::new("ffprobe")
        .args(["-v", "quiet"])
        .args(["-print_format", "json"])
        .arg("-show_streams")
        .arg("-show_format")
        .arg(video_name)
        .output()
        .unwrap();
    if !ffprobe_cmd.status.success() {
        println!("Failed to parse video metadata")
    }
    serde_json::from_slice::<FfProbeData>(&ffprobe_cmd.stdout).unwrap()
}

fn main() {
    let video_name = "./stick_vid.mp4";
    let video_info = get_video_info(video_name).streams;
    println!("Gathering frames from ffmpeg...");
    let frame_cmd = Command::new("ffmpeg")
        .args(["-i", video_name])
        .args(["-c:v", "png"])
        .args(["-vf", "format=rgb24"])
        .args(["-vcodec", "rawvideo"])
        .args(["-r", "60"])
        .args(["-f", "image2pipe", "-"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdout = frame_cmd.stdout.unwrap();
    let stderr = frame_cmd.stderr.unwrap();

    println!("Starting video loop...");
    let video_width = video_info[0].width.unwrap();
    let video_height = video_info[0].height.unwrap();
    loop {
        let mut buffer = vec![0u8; (video_width * video_height * 3) as usize];

        if let Err(err) = stdout.read_exact(&mut buffer) {
            if err.kind() == ErrorKind::UnexpectedEof {
                break;
            }
            println!("{}", err);
            break;
        }
        let image = RgbImage::from_raw(
            video_width as u32, video_height as u32,
            buffer,
        ).expect("Buffer to be the correct size");
        let rgb_image = DynamicImage::ImageRgb8(image);
        print!("{}", image_to_ascii(rgb_image, 20));
        thread::sleep(Duration::from_millis(10));
        cls();
    }
}