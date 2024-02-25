use crate::types::{EdgeSet, EdgeVec, Point};
use crate::util::{all_neighbours, out_of_bounds, wall_between};

use std::{
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
};

/// bundles metadata with a node required by the A* algorithm
#[derive(Copy, Clone, Debug)]
struct AStarNode {
    xy: Point,
    parent: Point,
    f_cost: i32,
    g_cost: i32,
    // no need to store h_cost
}

impl Eq for AStarNode {}
impl PartialEq for AStarNode {
    fn eq(&self, other: &Self) -> bool {
        self.xy == other.xy
    }
}

impl Hash for AStarNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.xy.hash(state);
    }
}

#[rustfmt::skip]
fn match_diff(diff: (i32, i32), max: bool, amt: i32) -> String {
    match diff {
        (0, -1) => if max { "⇈ Max up (+1)".to_string() } else { format!("↑ {amt} up (+{amt})") },
        (0, 1) => if max { "⇊ Max down (+1)".to_string() } else { format!("↓ {amt} down (+{amt})") },
        (-1, 0) => if max { "⇇ Max left (+1)".to_string() } else { format!("⇽ {amt} left (+{amt})") },
        (1, 0) => if max { "⇉ Max right (+1)".to_string() } else { format!("⇾ {amt} right (+{amt})") },

        _ => unreachable!("the above branches cover all possibilities")
    }
}

/// part of the function below, finds the length to the end of a corridor past a turning point
fn remaining_length(
    width: i32,
    height: i32,
    before: Point,
    old_diff: (i32, i32),
    walls: &EdgeSet,
) -> i32 {
    let mut distance_from_before = 1;

    loop {
        let node1 = (
            before.0 + old_diff.0 * distance_from_before,
            before.1 + old_diff.1 * distance_from_before,
        );

        let node2 = (node1.0 - old_diff.0, node1.1 - old_diff.1);
        if out_of_bounds(node1, width, height) || wall_between(walls, node1, node2) {
            break;
        }

        distance_from_before += 1;
    }

    distance_from_before
}

/// counts the moves for a "perfect run"
///
/// on the Discord bot, there is a button to move the furthest distance possible in a direction
/// this will count the moves in a solution, with the above condition in mind
///
/// this function is quite long, so it's been split into two parts
fn get_moves(
    width: i32,
    height: i32,
    path: &EdgeVec,
    walls: &EdgeSet,
) -> (MoveCount, UserFriendlyDirections) {
    let mut n_moves = 0;
    let mut perfect_run = vec![];
    let (_, first_af) = path.iter().copied().next().unwrap(); // path is never empty
    let mut prev_diff = (0 - first_af.0, 0 - first_af.1);
    let mut prev_turn_point = (0, 0);

    for (before, current) in path.iter().copied() {
        let diff = (current.0 - before.0, current.1 - before.1);
        if prev_diff == diff {
            continue;
        }

        let old_diff = prev_diff;
        prev_diff = diff;

        let diff_to_prev = (
            i32::abs_diff(prev_turn_point.0, before.0),
            i32::abs_diff(prev_turn_point.1, before.1),
        );

        // basically whichever x or y coordinate had changed
        let to_use = if diff_to_prev.0 == 0 {
            diff_to_prev.1
        } else {
            diff_to_prev.0
        } as i32;

        prev_turn_point = before;
        let distance_from_before = remaining_length(width, height, before, old_diff, walls);

        if to_use > 0 && distance_from_before >= to_use {
            perfect_run.push(match_diff(old_diff, false, to_use));
            n_moves += to_use;
            continue;
        } else if distance_from_before >= to_use {
            continue;
        }

        n_moves += distance_from_before;
        perfect_run.push(match_diff(old_diff, true, 1));
        if distance_from_before == 1 {
            continue;
        }

        perfect_run.push(match_diff(
            (-old_diff.0, -old_diff.1),
            false,
            distance_from_before - 1,
        ));
    }

    n_moves += 1;
    perfect_run.push(match_diff(
        prev_diff,
        // maze coordinates are zero-indexed, so width and height are adjusting accordingly
        prev_turn_point != (width - 2, height - 1) && prev_turn_point != (width - 1, height - 2),
        1,
    ));

    (n_moves, perfect_run)
}

/// we store the parent of each neighbour in that neighbour's data,
/// so now we just follow the chain of parents back from end to start
fn trace_path(min: i32, mut current: AStarNode, closed: &HashMap<Point, AStarNode>) -> EdgeVec {
    let mut path = Vec::with_capacity(min as usize);
    loop {
        let parent = *closed.get(&current.parent).unwrap();
        let before_xy = current.xy;
        current = parent;

        path.push((current.xy, before_xy));
        if current.xy == (0, 0) {
            break;
        }
    }

    path
}

/// part of the function below
fn a_star_for_neighbours(
    neighbours: &Vec<Point>,
    best: AStarNode,
    walls: &EdgeSet,
    end: Point,
    open: &mut HashSet<AStarNode>,
    closed: &HashMap<Point, AStarNode>,
) {
    let f_predicate = |&n: &&(i32, i32)| {
        !walls.contains(&(best.xy, *n))
            && !walls.contains(&(*n, best.xy))
            && !closed.contains_key(&n)
    };

    neighbours.iter().filter(f_predicate).for_each(|n| {
        let h_cost = end.0 - n.0 + end.1 - n.1;
        let g_cost = n.0 + n.1;
        let node = AStarNode {
            xy: *n,
            parent: best.xy,
            f_cost: g_cost + h_cost,
            g_cost,
        };

        if node.g_cost < best.g_cost || !open.contains(&node) {
            open.insert(node);
        }
    });
}

type MoveCount = i32;
type UserFriendlyDirections = Vec<String>;

/// uses the A* algorithm to compute a maze's solution
///
/// this was quite a long function, so it's been split into multiple parts
///
/// <https://www.youtube.com/watch?v=-L-WgKMFuhE> great video btw, a pure no-bullshit runthrough of A*
pub fn a_star_solution(
    walls: &EdgeSet,
    width: i32,
    height: i32,
) -> (MoveCount, UserFriendlyDirections, EdgeVec) {
    let min = width + height - 2; // theoretical minimum amount of moves it takes to finish a maze of a given size
    let mut open: HashSet<AStarNode> = HashSet::with_capacity(min as usize);
    let mut closed: HashMap<Point, AStarNode> = HashMap::with_capacity(min as usize);

    let start_node = AStarNode {
        xy: (0, 0),
        parent: (0, 0),
        g_cost: 0,
        f_cost: min,
    };

    open.insert(start_node);

    let end = (width - 1, height - 1);
    let last_node = loop {
        let best = open
            .iter()
            .min_by(|a, b| i32::cmp(&a.f_cost, &b.f_cost))
            .copied()
            .unwrap_or(start_node);

        open.remove(&best);
        closed.insert(best.xy, best);
        if best.xy == end {
            break best;
        }

        let neighbours = all_neighbours(best.xy, width, height);
        a_star_for_neighbours(&neighbours, best, walls, end, &mut open, &closed);
    };

    let path = trace_path(min, last_node, &closed);
    let (n_moves, moves) = get_moves(width, height, &path.iter().rev().copied().collect(), walls);

    (n_moves, moves, path)
}
