use crate::types::{EdgeSet, EdgeVec, Pxl};

use image::{imageops, GenericImage, Pixel, Rgba, RgbaImage};
use imageproc::{definitions::Image, drawing::draw_filled_rect_mut, rect::Rect};

use pyo3::types::PyBytes;
use rayon::prelude::*;

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use std::cell::UnsafeCell;

/// path/wall pixel gaps in generated images
const CELL: i32 = 20;
const WALL_THICKNESS: i32 = 3;
const SHIFT: i32 = 21;

// fallback "player icon" in case the images in the assets folder don't work
pub const HALF_WHITE: Pxl = Rgba([255, 255, 255, 100]);
pub const HALF_BLACK: Pxl = Rgba([0, 0, 0, 127]);

/// wraps an image and allows its mutability to be shared across threads
/// in our case, each thread is drawing on non-overlapping pixels
/// so we don't care about the race condition that this creates
struct SharedImage<P: Pixel + Sync, I: GenericImage<Pixel = P>> {
    wrapped: UnsafeCell<I>,
}

impl<P: Pixel + Sync, I: GenericImage<Pixel = P>> SharedImage<P, I> {
    /// makes a new instance, wrapping the passed image in an `UnsafeCell`
    fn new(img: I) -> Self {
        Self {
            wrapped: UnsafeCell::from(img),
        }
    }

    /// # Safety
    /// Gives the caller access to multiple mutable references of the same value.
    /// If used in a concurrent context, this will create a data race
    #[allow(clippy::mut_from_ref)] // it's the entire point of this type
    fn get_image_mut(&self) -> &mut I {
        unsafe { &mut *self.wrapped.get() }
    }

    /// destroys this object and returns the wrapped image
    fn into_inner(self) -> I {
        self.wrapped.into_inner()
    }
}

unsafe impl<P: Pixel + Sync, I: GenericImage<Pixel = P>> Sync for SharedImage<P, I> {}

/// generates the maze image using its wall edges
pub fn maze_image(
    walls: &EdgeSet,
    bg_colour: Pxl,
    wall_colour: Pxl,
    end_icon: &Image<Pxl>,
    width: i32,
    height: i32,
) -> Image<Pxl> {
    // subtract 1 from width and height as the coordinates are zero-indexed
    let (w, h) = ((width - 1) * CELL * 2 + 37, (height - 1) * CELL * 2 + 37);
    let mut img = RgbaImage::from_pixel(w as u32, h as u32, bg_colour);

    let (x, y) = ((i64::from(width) - 1) * 40, (i64::from(height) - 1) * 40);
    imageops::overlay(&mut img, end_icon, x, y); // draws the end marker at the bottom-right corner

    let shared = SharedImage::new(img);
    walls.par_iter().for_each(|(node1, node2)| {
        let (x, y) = (((node1.0 + 1) * CELL * 2), ((node1.1 + 1) * CELL * 2));
        let rect = if node1.0 == node2.0 {
            Rect::at(x - 43, y - WALL_THICKNESS).of_size(43, WALL_THICKNESS as u32)
        } else {
            Rect::at(x - WALL_THICKNESS, y - 43).of_size(WALL_THICKNESS as u32, 43)
        };

        let img = shared.get_image_mut();
        draw_filled_rect_mut(img, rect, wall_colour);
    });

    shared.into_inner()
}

/// very similar to the function above, but still different enough to where a single macro
/// can't cover both functions without tons of function-specific casing... and indents
/// or maybe that's just a skill issue on my part
pub fn solution_image(
    original: Image<Pxl>,
    solution: &EdgeVec,
    solution_line_colour: Pxl,
) -> Image<Pxl> {
    let shared = SharedImage::new(original);

    solution.par_iter().for_each(|(node1, node2)| {
        let (x, y) = ((((node1.0 + 1) * CELL) * 2), (((node1.1 + 1) * CELL) * 2));
        let rect = if node1.0 == node2.0 {
            let coords = if node1.1 < node2.1 {
                (x - WALL_THICKNESS - SHIFT, y - WALL_THICKNESS - SHIFT)
            } else {
                (x - WALL_THICKNESS - SHIFT, y - 43 - SHIFT)
            };

            Rect::at(coords.0, coords.1).of_size(6, 46)
        } else {
            let coords = if node1.0 < node2.0 {
                (x - WALL_THICKNESS - SHIFT, y - WALL_THICKNESS - SHIFT)
            } else {
                (x - 43 - SHIFT, y - WALL_THICKNESS - SHIFT)
            };

            Rect::at(coords.0, coords.1).of_size(46, 6)
        };

        let img = shared.get_image_mut();
        draw_filled_rect_mut(img, rect, solution_line_colour);
    });

    shared.into_inner()
}

/// if the supplied player icon is unusable/not given
pub fn fallback_image(name: &str, bg_colour: Pxl) -> Image<Pxl> {
    // summing 4 RGBA u8 values will most likely overflow
    let bg_sum: u16 = bg_colour.0.iter().map(|n_u8| u16::from(*n_u8)).sum();
    let path = if bg_sum > 382 { "black" } else { "white" };
    let fallback_colour = if bg_sum > 382 { HALF_BLACK } else { HALF_WHITE };

    match image::open(format!("assets/{name}-{path}.png")) {
        Ok(img) => img.into_rgba8(),
        Err(_) => RgbaImage::from_pixel(37, 37, fallback_colour),
    }
}

/// takes a `bytes` object from Python, and converts it to an `image::ImageBuffer`
pub fn bytes_to_image(bytes: &PyBytes, image_name: &str) -> PyResult<Image<Pxl>> {
    match image::load_from_memory_with_format(bytes.as_bytes(), image::ImageFormat::Png) {
        Ok(img) => Ok(img.into_rgba8()),
        Err(e) => Err(PyValueError::new_err(format!("{image_name} image: {e}"))),
    }
}
