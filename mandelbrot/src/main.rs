extern crate num;
extern crate image;
extern crate crossbeam;

use std::io::Write;


/// Try to determine if `c` is in the Manbelbrot set, usinnnnnnnnng at most `limit` iterations to decide.
/// 
/// If `c` is not a member, return `Some(i)`, where `i` is the number of iterations it took for `c` to leave the circle of
/// radius two centered on the origin. If `c` seems to be a member (most precisely, if we reached the iteration limit without
/// being able to prove that `c` is not a member), return `None`.
fn escape_time(c: num::Complex<f64>, limit: u32) -> Option<u32> {
    const THRESHOLD :f64 = 4.0;
    let mut z = num::Complex { re: 0.0, im: 0.0 };
    for i in 0..limit {
        z = z * z + c;
        if z.norm_sqr() > THRESHOLD {
            return Some(i);
        }
    }
    None
}


/// Parse the string `s` as a coordinate pair, like `"400x600"` or `"1.0,0.5"`.
/// 
/// Specifically, `s` should have the form <left><sep><right>, where <sep> is the character given by the `separator` argument,
/// and <left> and <right> are both strings that can be parsed by `T::from_str`.
/// 
/// If `s` has the proper form, return `Some<(x, y)>`. If it doesn't parse correctly, return `None`.
fn parse_pair<T: std::str::FromStr>(s: &str, separator: char) -> Option<(T, T)> {
    match s.find(separator) {
        None => None,
        Some(index) => {
            // The expressions &s[..index] and &s[index + 1..] are slices of the string, preceding and following the separator.
            // The type parameter T's associated from_str function takes each of these and tries to parse them as a value of type T, producing a tuple of results.
            match(T::from_str(&s[..index]), T::from_str(&s[index + 1..])) {
                (Ok(l), Ok(r)) => Some((l, r)),
                _ => None
            }
        }
    }
}
#[test]
fn test_parse_pair() {
    assert_eq!(parse_pair::<i32>("", ','), None);
    assert_eq!(parse_pair::<i32>("10,", ','), None);
    assert_eq!(parse_pair::<i32>(",10", ','), None);
    assert_eq!(parse_pair::<i32>("10,20", ','), Some((10, 20)));
    assert_eq!(parse_pair::<i32>("10,20xy", ','), None);
    assert_eq!(parse_pair::<f64>("0.5x", 'x'), None);
    assert_eq!(parse_pair::<f64>("0.5x1.5", 'x'), Some((0.5, 1.5)));
}


/// Parse a pair of floating-point numbers separated by a comma as a complex number.
fn parse_complex(s: &str) -> Option<num::Complex<f64>> {
    match parse_pair(s, ',') {
        Some((re, im)) => Some(num::Complex{ re, im }),
        None => None
    }
}
#[test]
fn test_parse_complex() {
    assert_eq!(parse_complex("1.25,-0.0625"), Some(num::Complex{ re: 1.25, im: -0.0625 }));
    assert_eq!(parse_complex(",-0.0625"), None);
}


/// Given the row and column of a pixel in the output image, return the corresponding point on the complex plane.
///
/// `bounds` is a pair giving the width and height of image in pixels.
/// `pixel` ia a (column, row) pair indicationg a particular pixel in that image,
/// The `upper_left` and `lower_right` parameters are points on the complex plane designating the area our image covers.
fn pixel_to_point(bounds: (usize, usize), pixel: (usize, usize), upper_left: num::Complex<f64>, lower_right: num::Complex<f64>) -> num::Complex<f64> 
{
    let (width, height) = (lower_right.re - upper_left.re, upper_left.im - lower_right.im);

    // Why subtraction here ? pixel.1 increases as we go down, but the imaginary component increases as we go up.
    num::Complex {
        re: upper_left.re + pixel.0 as f64 * width / bounds.0 as f64,
        im: upper_left.im - pixel.1 as f64 * height / bounds.1 as f64
    }
}
#[test]
fn test_pixel_to_point() {
    assert_eq!(pixel_to_point((100, 100), (25, 75), num::Complex{ re: -1.0, im: 1.0}, num::Complex{ re: 1.0, im: -1.0 }), num::Complex{ re: -0.5, im: -0.5 });
}


/// render a rectangle of the Mandelbrot set into a buffer of pixels.
/// 
/// The `bounds` argument gives the width and height of the buffer `pixels`, which holds one grayscale pixel per byte.
/// The `upper_left` and `lower_right` arguments specify points on the complex corresponding to the upper_left and lower_right corners of pixels buffer.
fn render(pixels: &mut[u8], bounds: (usize, usize), upper_left: num::Complex<f64>, lower_right: num::Complex<f64>)
{
    assert!(pixels.len() == bounds.0 * bounds.1);

    const WHITE :u32 = 0xff;
    for row in 0 .. bounds.1 {
        for column in 0 .. bounds.0 {
            let point = pixel_to_point(bounds, (column, row), upper_left, lower_right);
            pixels[row * bounds.0 + column] = match escape_time(point, WHITE) {
                None => 0,
                Some(count) => (WHITE - count) as u8
            }
        }
    }
}


/// Write the buffer `pixels` whose dimensions are given by `bounds`, to the file named `filename`.
fn write_image(filename: &str, pixels: &[u8], bounds: (usize, usize)) -> Result<(), std::io::Error>
{
    // The ?operator exist to make these checks convenient. Instead of spelling everything out, and writing:
    // let output = match std::fs::File::create(filename { Ok(f) => {f} Err(e);});
    let output = std::fs::File::create(filename)?;
    let encoder = image::png::PNGEncoder::new(output);
    encoder.encode(&pixels, bounds.0 as u32, bounds.1 as u32, image::ColorType::Gray(8))?;

    Ok(())
}



fn main() {
    const ARGS_NUM :usize = 5;
    let args: Vec<String> = std::env::args().collect();
    if args.len() != ARGS_NUM {
        writeln!(std::io::stderr(), "Usage: mandelbrot FILE PIXELS UPPERLEFT LOWERRIGHT").unwrap();
        writeln!(std::io::stderr(), "Example: {} mandelbrot.png 1000x750 -1.20,0.35 1.0,0.20", args[0]).unwrap();
        std::process::exit(1);
    }

    let bounds = parse_pair(&args[2], 'x').expect("error parsing image dimensions");
    let upper_left = parse_complex(&args[3]).expect("error parsing upper left corner point");
    let lower_right = parse_complex(&args[4]).expect("error parsing lower right corner point");

    let mut pixels = vec![0; bounds.0 * bounds.1];

    const THREADS :usize = 8;
    let rows_per_band = (bounds.1 / THREADS) + 1;
    {
        let bands: Vec<&mut [u8]> = pixels.chunks_mut(rows_per_band * bounds.0).collect();
        // The argument |spawner| { ... } is a Rust closure expression. A closure is a value that can be called as if it were a function.
        // Here, |spawner| is the argument list, and { ... } is the body of the function.
        crossbeam::scope(|spawner| {
            for (i, band) in bands.into_iter().enumerate() {  // The enumerate adapter producer tuples pairing each vector element with its index.
                let top = rows_per_band * i;
                let height = band.len() / bounds.0;
                let band_bounds = (bounds.0, height);
                let band_upper_left = pixel_to_point(bounds, (0, top), upper_left, lower_right);
                let band_lower_right = pixel_to_point(bounds, (bounds.0, top + height), upper_left, lower_right);

                spawner.spawn(move || { 
                    render(band, band_bounds, band_upper_left, band_lower_right);
                });
            }
        });
    }
    
    write_image(&args[1], &pixels, bounds).expect("error writing PNG file")
}
