mod algorithms;
mod types;
mod util;

use algorithms::{
    a_star_solution, bytes_to_image, fallback_image, generate_edges, maze_image, solution_image,
};

use types::{EdgeVec, Point, Pxl};
use util::{out_of_bounds, wall_between};

use image::{imageops, ImageOutputFormat, Rgba};
use imageproc::{definitions::Image, drawing::draw_filled_rect_mut, rect::Rect};

use std::{collections::HashSet, io::Cursor};

use pyo3::create_exception;
use pyo3::prelude::*;
use pyo3::{
    exceptions::{PyException, PyIOError, PyValueError},
    types::{PyBytes, PySequence, PyTuple},
};

create_exception!(maze, SolutionNotFound, PyException);

/// bundles elements representing a maze
#[pyclass(module = "maze")]
struct Maze {
    width: i32,
    height: i32,
    bg_colour: Pxl,
    solution_colour: Pxl,
    solution_moves: Option<(i32, Vec<String>)>,
    maze_image: Image<Pxl>,
    player_icon: Image<Pxl>,
    walls: HashSet<(Point, Point)>,
}

/// private methods (not exposed to the Python)
impl Maze {
    /// draws the solution path onto the maze image
    fn draw_solution(&mut self, py: Python, solution: &EdgeVec) {
        let img = std::mem::take(&mut self.maze_image);

        self.maze_image = py.allow_threads(|| solution_image(img, solution, self.solution_colour));
    }
}

/// public methods (exposed to the Python)
#[pymethods]
impl Maze {
    /// whether or not two points are blocked off by a wall
    #[pyo3(signature = (a, b, /))]
    fn has_wall_between(&self, a: Point, b: Point) -> bool {
        let (w, h) = (self.width, self.height);
        wall_between(&self.walls, a, b) || out_of_bounds(b, w, h) || out_of_bounds(a, w, h)
    }

    /// removes the player (if it exists) at an XY coodinate
    ///
    /// this essentially just pastes the background colour over those coordinates
    #[pyo3(signature = (xy, /))]
    fn undraw_at(&mut self, xy: Point) {
        let rect = Rect::at(xy.0 * 40, xy.1 * 40).of_size(37, 37);
        draw_filled_rect_mut(&mut self.maze_image, rect, self.bg_colour);
    }

    /// draws the player at a given XY coordinate
    #[pyo3(signature = (xy, /))]
    fn draw_player_at(&mut self, xy: Point) {
        let (x, y) = (i64::from(xy.0) * 40, i64::from(xy.1) * 40);
        imageops::overlay(&mut self.maze_image, &self.player_icon, x, y);
    }

    /// determines the solution to the maze, along with a set of "perfect moves"
    ///
    /// on the Discord bot, there is a button to move the furthest distance possible in a direction
    /// this will count the moves in a solution, with the above condition in mind
    ///
    /// this will store the solution in an internal field;
    /// to get the actual value, use `.get_solution()`
    #[pyo3(signature = (*, draw_path))]
    fn compute_solution(&mut self, py: Python, draw_path: bool) {
        let (n_moves, moves, solution) = a_star_solution(&self.walls, self.width, self.height);
        self.solution_moves = Some((n_moves, moves));

        if draw_path {
            self.draw_solution(py, &solution);
        }
    }

    /// returns the maze's solution if one has already been determined, otherwise raise `SolutionNotFound`
    ///
    /// the solution is essentially a tuple containing two items
    /// the first is a `u32` of how many moves a "perfect run" would take
    /// the second is a string of newline-separated human-readable directions (e.g "2 right", "3 left")
    ///
    /// this call clones a Rust object and converts it to Python,
    /// which introduces a significant amount of overhead (use it sparingly!)
    fn get_solution_expensively<'py>(&mut self, py: Python<'py>) -> PyResult<&'py PyAny> {
        if self.solution_moves.is_none() {
            const MSG: &str = "make sure to call `.compute_solution()` first";
            return Err(SolutionNotFound::new_err(MSG));
        }

        let solution = self.solution_moves.clone().unwrap();

        let collections = py.import("collections")?;
        let tuple_fields = ["move_count", "directions"].into_py(py);
        let type_args = PyTuple::new(py, ["Solution".into_py(py), tuple_fields]);

        collections
            .getattr("namedtuple")?
            .call1(type_args)? // instantiates the namedtuple type
            .call1(solution) // instantiates an instance of said type
    }

    /// clones the maze image into a `io.BytesIO` buffer in Python
    ///
    /// this call clones a Rust object and converts it to Python,
    /// which introduces a significant amount of overhead (use it sparingly!)
    fn get_image_expensively<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let mut buf = Cursor::new(vec![]);
        match self.maze_image.write_to(&mut buf, ImageOutputFormat::Png) {
            Ok(()) => (),
            Err(e) => return Err(PyIOError::new_err(format!("could not write image: {e}"))),
        }

        let io = py.import("io")?;
        let builtins = py.import("builtins")?;

        let data = PyTuple::new(py, [buf.into_inner()]);
        let arr = builtins.getattr("bytearray")?.call1(data)?;

        let init_bytes = PyTuple::new(py, [arr]);
        io.getattr("BytesIO")?.call1(init_bytes)
    }

    /// moves the player as far as they can go in a particular direction, and return that position
    ///
    /// this will also re-draw the player on the maze
    #[pyo3(signature = (current, direction, /))]
    fn move_max(&mut self, mut current: Point, direction: (i32, i32)) -> Point {
        let old = current;
        loop {
            // the next node one over in the direction to look
            let n = (current.0 + direction.0, current.1 + direction.1);
            if out_of_bounds(n, self.width, self.height) || self.has_wall_between(current, n) {
                break;
            }

            current = n;
        }

        self.undraw_at(old);
        self.draw_player_at(current);
        current
    }
}

/// takes a Python tuple of either RGB or RGBA values, and shoves it into `image::Rgba`
macro_rules! into_rgba {
    ($name:tt) => {
        let len = $name.len().unwrap_or(0); // if a list/tuple has been passed, this will be `Some`
        if len != 3 && len != 4 {
            return Err(PyValueError::new_err(format!(
                "colour parameter expected RGB or RGBA collection; got value {}",
                $name.repr()?
            )));
        }

        let mut arr = [255u8; 4];
        for (idx, i) in $name.extract::<Vec<u8>>()?.iter().enumerate() {
            arr[idx] = *i;
        }

        let $name = Rgba(arr);
    };
}

/// new maze of a given width and height
#[pyfunction]
#[pyo3(signature = (*, width, height, bg_colour, wall_colour, solution_colour, player = None, endzone = None))]
#[allow(clippy::too_many_arguments)] // they're all keyword-only in Python
fn generate_maze<'py>(
    py: Python<'py>,
    width: i32,
    height: i32,
    bg_colour: &'py PySequence,
    wall_colour: &'py PySequence,
    solution_colour: &'py PySequence,
    player: Option<&'py PyBytes>,
    endzone: Option<&'py PyBytes>,
) -> PyResult<Maze> {
    into_rgba!(bg_colour);
    into_rgba!(wall_colour);
    into_rgba!(solution_colour);

    let (width, height) = (width, height);
    let (walls, _) = generate_edges(width, height);
    let player_icon = match player {
        None => fallback_image("player", bg_colour),
        Some(img) => bytes_to_image(img, "player")?,
    };

    let end_icon = match endzone {
        None => fallback_image("endzone", bg_colour),
        Some(img) => bytes_to_image(img, "endzone")?,
    };

    // screw the GIL
    let maze_image =
        py.allow_threads(|| maze_image(&walls, bg_colour, wall_colour, &end_icon, width, height));

    Ok(Maze {
        walls,
        maze_image,
        width,
        height,
        bg_colour,
        player_icon,
        solution_colour,
        solution_moves: None,
    })
}

const ALL: [&str; 8] = [
    "__version__",
    "Maze",
    "generate_maze",
    "SolutionNotFound",
    "UP",
    "DOWN",
    "LEFT",
    "RIGHT",
];

#[pymodule]
fn maze(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(generate_maze, m)?)?;
    m.add_class::<Maze>()?;

    m.add("SolutionNotFound", py.get_type::<SolutionNotFound>())?;

    m.add("UP", (0, -1))?;
    m.add("DOWN", (0, 1))?;
    m.add("LEFT", (-1, 0))?;
    m.add("RIGHT", (1, 0))?;

    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("__all__", ALL)?;

    Ok(())
}
