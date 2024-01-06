use std::collections::HashSet;

use image::Rgba;

/// XY coordinate
pub type Point = (i32, i32);

/// `(node1, node2)` the two nodes the edge links
pub type EdgeSet = HashSet<(Point, Point)>;

/// pretty much the same as `EdgeSet` but a Vec instead
pub type EdgeVec = Vec<(Point, Point)>;

/// just so that i don't need to manually change this every time
pub type Pxl = Rgba<u8>;
