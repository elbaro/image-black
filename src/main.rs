extern crate walkdir;
extern crate image;
extern crate clap;
use clap::{Arg,App};

use walkdir::WalkDir;
use image::DynamicImage;
use image::DynamicImage::*;
use image::ConvertBuffer;

fn main() {
    let config = App::new("image black")
        .version("alpha")
        .about("convert image color")
        .arg(
            Arg::with_name("DIR")
                .required(true)
                .index(1)
                .help("converts all images in this directory (recursive)")
        )
        .arg(
            Arg::with_name("color")
                .short("c")
                .long("color")
                .value_name("COLOR_TYPE")
                .takes_value(true)
                .required(true)
                .help("target color type. gray | graya | rgb | rgba")
        )
        .arg(
            Arg::with_name("test")
                .short("t")
                .long("test")
                .help("test mode: doesn't write files")
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("verbose mode")
        )
        .get_matches();

    let c = config.value_of("color").unwrap();
    let verbose = config.is_present("verbose");
    let test = config.is_present("test");

    let (color, convert) :(_,Box<Fn(DynamicImage)->DynamicImage>) =
        match c.to_ascii_lowercase().as_str() {
        "g" | "gray" | "grey" | "grayscale" | "greyscale" => {
            (
                image::ColorType::Gray(8),
                Box::new(|img: DynamicImage|
                    ImageLuma8(
                        match img {
                            ImageLuma8(img) => img.convert(),
                            ImageLumaA8(img) => img.convert(),
                            ImageRgb8(img) => img.convert(),
                            ImageRgba8(img) => img.convert(),
                        }
                    ))
            )
        }
        "ga" | "graya" | "greya" | "grayalpha" | "greyalpha" => {
            (
                image::ColorType::GrayA(8),
                Box::new(|img: DynamicImage|
                    ImageLumaA8(
                        match img {
                            ImageLuma8(img) => img.convert(),
                            ImageLumaA8(img) => img.convert(),
                            ImageRgb8(img) => img.convert(),
                            ImageRgba8(img) => img.convert(),
                        }
                    ))
            )
        }
        "rgb" => {
            (
                image::ColorType::RGB(8),
                Box::new(|img: DynamicImage|
                    ImageRgb8(
                        match img {
                            ImageLuma8(img) => img.convert(),
                            ImageLumaA8(img) => img.convert(),
                            ImageRgb8(img) => img.convert(),
                            ImageRgba8(img) => img.convert(),
                        }
                    ))
            )
        }
        "rgba" => {
            (
                image::ColorType::RGBA(8),
                Box::new(|img: DynamicImage|
                    ImageRgba8(
                        match img {
                            ImageLuma8(img) => img.convert(),
                            ImageLumaA8(img) => img.convert(),
                            ImageRgb8(img) => img.convert(),
                            ImageRgba8(img) => img.convert(),
                        }
                    ))
            )
        }
        _ => {
            panic!("unknown color type: {}", c);
        }
    };

    let mut failed = 0;
    let mut intact = 0;
    let mut converted = 0;
    if test {
        println!("[test mode - no files will be modified]");
    }
    let dir = config.value_of("DIR").unwrap_or("");
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Ok(img) = image::open(entry.path()) {
                if verbose { println!("{:?} : {:?}", img.color(), entry.path()); }
                if img.color() == color {
                    intact += 1;
                } else {
                    if test {
                        converted += 1;
                        continue;
                    }
                    let img = convert(img);
                    let save = match img {
                        ImageLuma8(img) => img.save(entry.path()),
                        ImageLumaA8(img) => img.save(entry.path()),
                        ImageRgb8(img) => img.save(entry.path()),
                        ImageRgba8(img) => img.save(entry.path()),
                    };
                    if save.is_ok() {
                        converted += 1;
                    } else {
                        eprintln!("failed to save: {:?}", entry.path());
                        failed += 1;
                    }
                }
            } else {
                failed += 1;
            }
        }
    }
    println!("not changed: {:8} pics", intact);
    println!("converted  : {:8} pics", converted);
    println!("failed     : {:8} files (not image / corrupted / cannot save)", failed);
}
