extern crate rayon;
extern crate walkdir;
extern crate image;
extern crate indicatif;
extern crate regex;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate derivative;
extern crate atomic_counter;
extern crate colored;

use rayon::prelude::*;
use walkdir::WalkDir;
use image::DynamicImage;
use image::GenericImage;
use std::path::PathBuf;
use std::path::Path;
use std::error::Error;
use std::io::BufReader;
use image::ImageDecoder;
use atomic_counter::AtomicCounter;
use colored::*;
use std::io::Write;

const USAGE: &'static str = "
image-black

Usage:
  image-black any     <filter>.. <src_dir>
  image-black list    <filter>.. <src_dir>
  image-black count   <filter>.. <src_dir>
  image-black remove  <filter>.. <src_dir>
  image-black convert <filter>.. to <transform>.. <src_dir> <dst_dir>
  image-black convert <filter>.. into <transform>.. <src_dir>

Filters:
         channel    : rgb | rgba | gray
         format     : png | jpg
         filesize   : filesize>10.5M filesize==300K filesize<50B
         dim        : long>=500 short==400 width<640 height>128
  jpeg   quality    : q==100 q<90 q>=80
  aspect ratio (w/h): aspect>2

Transforms:
           channel: rgb | rgba | gray | graya
           format : png | jpg
           dim    : long=512 short=128
           quality: q=90
    aspect ratio  : aspect=2

";

fn exit_with_usage() {
    println!("{}", USAGE);
    std::process::exit(1);
}

struct Metadata {
    width: u32,
    height: u32,
    color: image::ColorType,
}

fn read_metadata<P: AsRef<Path>>(path: P) -> Result<Metadata, Box<Error>> {
    let path = path.as_ref();
    let r = BufReader::new(std::fs::File::open(path)?);
    let ext = path.extension()
        .and_then(|s| s.to_str())
        .map_or("<no ext>".to_string(), |s| s.to_ascii_lowercase());
    let (dim, color) = match ext.as_ref() {
        "png" => {
            let mut d = image::png::PNGDecoder::new(r);
            (d.dimensions()?, d.colortype()?)
        }
        "jpg" | "jpeg" => {
            let mut d = image::jpeg::JPEGDecoder::new(r);
            (d.dimensions()?, d.colortype()?)
        }
        format => {
            return Err(From::from(image::ImageError::UnsupportedError(format!(
                "Image format image/{:?} is not supported.",
                format
            ))));
        }
    };
    Ok(Metadata { width: dim.0, height: dim.1, color })
}

struct ImageInfo<'a> {
    path: &'a Path,
    meta: Option<Metadata>,
    image: Option<DynamicImage>,
}

enum Filter {
    PathFilter(bool, fn(&Path) -> bool),
    MetaFilter(bool, fn(&Metadata) -> bool),
    ContentFilter(bool, fn(&DynamicImage) -> bool),

    FilesizeFilter(bool, fn(&u64, &u64) -> bool, u64),
    MetaIntCmpFilter(bool, fn(&Metadata) -> u64, fn(&u64, &u64) -> bool, u64),
}

fn is_rgb(meta: &Metadata) -> bool { meta.color == image::ColorType::RGB(8) }

fn is_rgba(meta: &Metadata) -> bool { meta.color == image::ColorType::RGBA(8) }

fn is_gray(meta: &Metadata) -> bool { meta.color == image::ColorType::Gray(8) }

fn is_graya(meta: &Metadata) -> bool { meta.color == image::ColorType::GrayA(8) }

fn get_ext(path: &Path) -> String { path.extension().map(|x| x.to_str().unwrap()).unwrap_or("").to_ascii_lowercase() }

fn is_png(path: &Path) -> bool { get_ext(path) == "png" }

fn is_jpg(path: &Path) -> bool { get_ext(path) == "jpg" }

fn parse_filter(args: &[String]) -> Result<Vec<Filter>, Box<Error>> {
    println!("==========");

    Ok(args.iter().map(|arg| {
        println!("filter: {}", arg);

        let not = arg.starts_with("!");
        let arg: &str = {
            if not { &arg[1..] } else { arg }
        };

        let arg = arg.to_ascii_lowercase();

        match arg.as_ref() {
            "rgb" => Filter::MetaFilter(not, is_rgb),
            "rgba" => Filter::MetaFilter(not, is_rgba),
            "gray" | "grey" => Filter::MetaFilter(not, is_gray),
            "graya" | "greya" => Filter::MetaFilter(not, is_graya),
            "png" => Filter::PathFilter(not, is_png),
            "jpg" => Filter::PathFilter(not, is_jpg),
            _ => {
                lazy_static! {
                    static ref RE: regex::Regex = regex::Regex::new(r"(?P<name>[[:lower:]]+)(?P<op>[><=]+)(?P<num>[[:digit:]]+(.[[:digit:]]+)?)(?P<unit>[bkm]?)").unwrap();
                }
                let capture = RE.captures(&arg).expect(&format!("wrong arg format: {}", arg));
                let name = capture.name("name").expect(&format!("wrong arg: {}", arg)).as_str();
                let op = capture.name("op").unwrap().as_str();

                let cmp = match op {
                    ">" => u64::gt,
                    ">=" => u64::ge,
                    "<" => u64::lt,
                    "<=" => u64::le,
                    "==" => u64::eq,
                    _ => panic!(format!("unknown operator: {}", arg))
                };

                let filter = match name {
                    "filesize" => {
                        let num: f64 = capture.name("num").expect("no number for filesize").as_str().parse().expect("fail to parse filesize");
                        let num: u64 = (num * (match capture.name("unit").expect("no unit for filesize").as_str() {
                            "b" => 1,
                            "k" => 1 << 10,
                            "m" => 1 << 20,
                            _ => panic!("unknown filesize unit")
                        } as f64)) as u64;

                        Filter::FilesizeFilter(not, cmp, num)
                    }
                    _ => {
                        let num: u64 = capture.name("num").expect("no number for filesize").as_str().parse().expect("fail to parse filesize");
                        let getter: fn(&Metadata) -> u64 = match name {
                            "width" => |meta: &Metadata| meta.width as u64,
                            "height" => |meta: &Metadata| meta.height as u64,
                            "long" => |meta: &Metadata| meta.width.max(meta.height) as u64,
                            "short" => |meta: &Metadata| meta.width.min(meta.height) as u64,
                            _ => {
                                panic!(format!("unknown arg: {}", arg));
                            }
                        };

                        Filter::MetaIntCmpFilter(not, getter, cmp, num)
                    }
                };
                filter
            }
        }
    }).collect())
}

#[derive(Debug)]
enum Padding {
    Constant(u8),
    _Mirror,
}

impl Default for Padding {
    fn default() -> Padding { Padding::Constant(0) }
}

//#[derive(Default, Debug)]
#[derive(Default, Derivative)]
#[derivative(Debug)]
struct Transform {
    format: Option<String>,
    color: Option<image::ColorType>,
    _jpeg_q: Option<u32>,
    width: Option<u32>,
    height: Option<u32>,
    long: Option<u32>,
    short: Option<u32>,
    both: Option<(u32, u32)>,
    pad: Option<Padding>,
    #[derivative(Debug = "ignore")]
    sampling: Option<image::FilterType>,
}

fn parse_transform(args: &[String]) -> Transform {
    let mut t: Transform = Default::default();

    println!("==========");
    args.iter().for_each(|arg| {
        println!("transform: {}", arg);

        let arg = arg.to_ascii_lowercase();

        match arg.as_ref() {
            "rgb" => { t.color = Some(image::ColorType::RGB(8)) }
            "rgba" => { t.color = Some(image::ColorType::RGBA(8)) }
            "gray" | "grey" => { t.color = Some(image::ColorType::Gray(8)) }
            "graya" | "greya" => { t.color = Some(image::ColorType::GrayA(8)) }
            "png" => { t.format = Some("png".to_string()) }
            "jpg" => { t.format = Some("jpg".to_string()) }
            "nearest" => { t.sampling = Some(image::FilterType::Nearest) }
            "bilinear" => { t.sampling = Some(image::FilterType::Triangle) }
            "bicubic" => { t.sampling = Some(image::FilterType::CatmullRom) } // good for upsample
            "gaussian" => { t.sampling = Some(image::FilterType::Gaussian) }
            "lanczos" => { t.sampling = Some(image::FilterType::Lanczos3) } // good for downsample

            _ => {
                lazy_static! {
                    static ref RE: regex::Regex = regex::Regex::new(r"(?P<name>[[:lower:]]+)=(?P<num>[[:digit:]]+(.[[:digit:]]+)?)").unwrap();
                }
                let capture = RE.captures(&arg).expect(&format!("unknown arg: {}", arg));
                let name = capture.name("name").expect(&format!("wrong arg: {}", arg)).as_str();
                let num: u32 = capture.name("num").expect("no number for filesize").as_str().parse().expect("fail to parse filesize");

                match name {
                    "width" => { t.width = Some(num) }
                    "height" => { t.height = Some(num) }
                    "long" => { t.long = Some(num) }
                    "short" => { t.short = Some(num) }
                    _ => panic!(format!("unknown arg: {}", arg))
                }
            }
        }
    });

    return t;
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        exit_with_usage();
    }

    let mode = &args[1];

    if mode == "convert"
        && args.iter().find(|&s| s == "to").is_none()
        && args.iter().find(|&s| s == "into").is_none() {
        println!("convert mode requires 'to' or 'into' keyword");
        std::process::exit(1);
    }

    let src_dir: &str =
        if mode == "convert" && args.iter().position(|ref s| s == &"to").is_some() {
            &args[args.len() - 2]
        } else {
            &args[args.len() - 1]
        };

    println!("walking dir .. ({})", src_dir);

    let files: Vec<PathBuf> = {
        let mut v = Vec::new();
        for entry in WalkDir::new(src_dir).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                v.push(entry.path().to_owned());
            }
        }
        v
    };

    let stat_total = files.len();
    let stat_fail = atomic_counter::RelaxedCounter::new(0);
    let stat_matched = atomic_counter::RelaxedCounter::new(0);
    println!("{} files found", format!("{}", stat_total).bright_green().bold());


    let mut convert_sep = 0;
    let filters = {
        if mode == "convert" {
            let i = 1 + std::cmp::max(
                args.iter().skip(1).position(|s| s == "to").unwrap_or(0),
                args.iter().skip(1).position(|s| s == "into").unwrap_or(0),
            );
            convert_sep = i;
            parse_filter(&args[2..i])
        } else {
            parse_filter(&args[2..args.len() - 1])
        }
    }.expect("fail to parse filters");

    let require_meta = filters.iter().any(|filter| {
        match filter {
            Filter::MetaFilter(_, _) => true,
            Filter::MetaIntCmpFilter(_, _, _, _) => true,
            _ => false
        }
    });

    let require_content = mode == "convert" ||
        filters.iter().any(|filter| {
            match filter {
                Filter::ContentFilter(_, _) => true,
                _ => false
            }
    });

    let bar = indicatif::ProgressBar::new(stat_total as u64);
    bar.set_style(indicatif::ProgressStyle::default_bar().template("{elapsed}/{eta} {wide_bar} {pos}/{len}"));


    let filter_fn = |info: &ImageInfo| {
        bar.inc(1);
        let m = filters.iter().all(|filter| {
            match filter {
                Filter::PathFilter(not, f) => not ^ f(&info.path),
                Filter::MetaFilter(not, f) => not ^ f(info.meta.as_ref().unwrap()),
                Filter::ContentFilter(not, f) => not ^ f(info.image.as_ref().unwrap()),
                Filter::FilesizeFilter(not, cmp, num) => not ^ cmp(&std::fs::metadata(&info.path).unwrap().len(), &num),
                Filter::MetaIntCmpFilter(not, f, cmp, num) => not ^ cmp(&f(info.meta.as_ref().unwrap()), &num),
            }
        });
        if m { stat_matched.inc(); }
        m
    };


    let logfile_path = {
        let mut p = std::env::temp_dir();
        p.push("image-black.log");
        p
    };
    let logfile_mutex = std::sync::Mutex::new(std::fs::File::create(&logfile_path).expect("fail to create an error log"));

    let it = files.par_iter().filter_map(|p| -> Option<ImageInfo> {
        Some(
            if require_content {
                let img = match image::open(p) {
                    Ok(img) => img,
                    Err(e) => {
                        writeln!(logfile_mutex.lock().unwrap(), "[error] {} {}", p.display(), e).unwrap();
                        stat_fail.inc();
                        return None
                    }
                };
                let dim = img.dimensions();
                let color = img.color();
                ImageInfo {
                    path: p,
                    meta: Some(Metadata{width:dim.0, height:dim.1, color:color}),
                    image: Some(img),
                }
            } else if require_meta {
                ImageInfo {
                    path: p,
                    meta: match read_metadata(p) {
                        Ok(meta) => Some(meta),
                        Err(e) => {
                            writeln!(logfile_mutex.lock().unwrap(), "[error] {} {}", p.display(), e).unwrap();
                            stat_fail.inc();
                            return None
                        },
                    },
                    image: None,
                }
            } else {
                ImageInfo {
                    path: p,
                    meta: None,
                    image: None
                }
            }
        )
    });

    match mode.as_ref() {
        "any" => {
            match it.find_any(filter_fn) {
                Some(info) => {
                    bar.println("found");
                    bar.println(format!("=> {:?}", info.path));
                }
                None => {
                    bar.println("none found.");
                }
            };
        }
        "list" => {
            it.for_each(|info| {
                if filter_fn(&info) {
                    bar.println(info.path.to_str().unwrap())
                }
            })
        }
        "count" => {
            it.filter(filter_fn).count();
        }
        "remove" => {
            it.filter(filter_fn).for_each(|info| {
                std::fs::remove_file(info.path).unwrap();
            });
        }
        "convert" => {
            let dst_dir = Path::new(&args[args.len() - 1]);

            let t =
                if args.iter().position(|ref s| s == &"to").is_some() {
                    parse_transform(&args[convert_sep + 1..args.len() - 2])
                } else {
                    parse_transform(&args[convert_sep + 1..args.len() - 1])
                };

            it.filter(filter_fn).for_each(|info| {
                let mut img = info.image.as_ref().unwrap().clone();
                let mut dst: PathBuf = dst_dir.join(info.path.strip_prefix(src_dir).unwrap());


                if let Some(_pad) = t.pad.as_ref() {
                    unimplemented!();
                }

                if let Some(format) = t.format.as_ref() { // change .png to .jpg
                    dst = dst.with_extension(format);
                }

                if let Some(color) = t.color {
                    img = match color {
                        image::ColorType::RGB(8) => DynamicImage::ImageRgb8(img.to_rgb()),
                        image::ColorType::RGBA(8) => DynamicImage::ImageRgba8(img.to_rgba()),
                        image::ColorType::Gray(8) => DynamicImage::ImageLuma8(img.to_luma()),
                        image::ColorType::GrayA(8) => DynamicImage::ImageLumaA8(img.to_luma_alpha()),
                        _ => {
                            unreachable!()
                        }
                    };
                }

                if t.long.is_some() || t.short.is_some() || t.both.is_some() {
                    // case1: both (w,h) are specified && force resize
                    // case1: both (w,h) are specified && keep aspect ratio with padding

                    // case2: one of (w,h) is specified && the other is auto-calc

                    // case3: one of (w,h) is specified
                    let meta = info.meta.as_ref().unwrap();
                    let dim =
                        if let Some(long) = t.long {
                            if meta.width > meta.height {
                                (long, (meta.height as f64 * (long as f64) / (meta.width as f64)) as u32)
                            } else {
                                (((meta.width as f64) * (long as f64) / meta.height as f64) as u32, long)
                            }
                        } else if let Some(short) = t.short {
                            if meta.width < meta.height {
                                (short, (meta.height as f64 * (short as f64) / (meta.width as f64)) as u32)
                            } else {
                                (((meta.width as f64) * (short as f64) / meta.height as f64) as u32, short)
                            }
                        } else if let Some(_both) = t.both {
                            unimplemented!()
                        } else {
                            unreachable!()
                        };
                    img = img.resize(dim.0, dim.1, t.sampling.unwrap_or(image::FilterType::Lanczos3));
                }
                std::fs::create_dir_all(dst.parent().unwrap()).unwrap();
                img.save(dst).unwrap();
            });
        }
        _ => {
            panic!("unknown mode");
        }
    }
    bar.finish();

    println!("{} files matched.", format!("{}",stat_matched.get()).bright_green().bold());

    let fail = stat_fail.get();
    if fail > 0 {
        println!("{} files failed to read", format!("{}", fail).bright_red().bold());
        println!("see the error log in {}", logfile_path.display());
    }
}
