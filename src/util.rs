use crate::types::{EdgeSet, Point};

/// gets the neighbours for this node one to the right and one down
#[rustfmt::skip]
pub fn partial_neighbours(node: Point, width: i32, height: i32) -> Vec<Point> {
    let mut adjacent = vec![];

    if node.0 + 1 < width { adjacent.push((node.0 + 1, node.1)) }
    if node.1 + 1 < height { adjacent.push((node.0, node.1 + 1)) }

    adjacent
}

/// gets all adjacent neighbours up, left, down, and right
#[rustfmt::skip]
pub fn all_neighbours(node: Point, width: i32, height: i32) -> Vec<Point> {
    let mut adjacent = partial_neighbours(node, width, height);

    if node.0 > 0 { adjacent.push((node.0 - 1, node.1)) }
    if node.1 > 0 { adjacent.push((node.0, node.1 - 1)) }

    adjacent
}

/// mouthful
pub const fn out_of_bounds(node: Point, width: i32, height: i32) -> bool {
    node.0 < 0 || node.1 < 0 || node.0 >= width || node.1 >= height
}

/// mouthful #2
pub fn wall_between(walls: &EdgeSet, a: Point, b: Point) -> bool {
    walls.contains(&(a, b)) || walls.contains(&(b, a))
}
