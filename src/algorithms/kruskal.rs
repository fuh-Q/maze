use crate::types::{EdgeSet, Point};
use crate::util::partial_neighbours;

use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
};

/// node container thing for the graph below, attaches it with a rank
#[derive(Copy, Clone, Debug, PartialEq)]
struct Node<T: Debug + Eq> {
    parent: T,
    rank: u32,
}

/// a disjoint-set data structure implementation with rank balancing
///
/// i think 🤡
struct Graph<T: Debug + Hash + Eq + Copy> {
    parents: HashMap<T, Node<T>>,
}

impl<T: Debug + Hash + Eq + Copy> Graph<T> {
    /// new instance from an iterable of nodes
    ///
    /// whatever type those nodes may be, they must implement the
    /// `Debug`, `Hash`, `Eq`, and `Copy` traits
    fn new<N: IntoIterator<Item = T>>(nodes: N) -> Self {
        let mut map = HashMap::new();
        for n in nodes {
            map.insert(n, Node { parent: n, rank: 0 });
        }

        Self { parents: map }
    }

    /// finds the parent of a node
    ///
    /// # Panics
    /// Will panic if the supplied node is not part of the graph
    fn find_and_cache_parent(&mut self, node: T) -> Node<T> {
        let find = match self.parents.get(&node) {
            Some(n) => *n,
            None => panic!("could not find node {node:?}"),
        };

        // immediately return if the node is its own parent
        if find.parent == node {
            return find;
        }

        // otherwise look for the parent recursively
        let mut result = self.find_and_cache_parent(find.parent);
        result.rank = find.rank;

        self.parents.insert(node, result);

        result
    }

    /// merges two subtrees into one, with rank balancing
    ///
    /// returns whether the two trees were successfully merged,
    /// a `false` return indicates that such a merge would cause a loop
    fn union_subtrees(&mut self, a: T, b: T) -> bool {
        let a_parent = &mut self.find_and_cache_parent(a);
        let b_parent = &mut self.find_and_cache_parent(b);
        if a_parent.parent == b_parent.parent {
            return false;
        }

        if a_parent.rank == b_parent.rank {
            a_parent.rank += 1;
        }

        if a_parent.rank >= b_parent.rank {
            self.parents.insert(b_parent.parent, *a_parent);
        } else {
            self.parents.insert(a_parent.parent, *b_parent);
        }

        true
    }
}

/// generates an MST with `width * height` nodes, using Kruskal's Algorithm
///
/// returns a tuple `(walls, paths)` of the maze
pub fn generate_edges(width: i32, height: i32) -> (EdgeSet, EdgeSet) {
    // flattened collection of every xy coordinate in the maze
    let nodes: Vec<Point> = (0..width)
        .flat_map(|x| (0..height).map(move |y| (x, y)))
        .collect();

    // using a set since we want these edges shuffled when we iterate
    let edge_count = ((width - 1) * height + (height - 1) * width) as usize;
    let mut edges = HashSet::with_capacity(edge_count);
    for node in nodes.iter().copied() {
        let neighbours = partial_neighbours(node, width, height);
        for nbour in neighbours {
            edges.insert((node, nbour));
        }
    }

    let mut graph: Graph<Point> = Graph::new(nodes);

    // let mut paths = HashSet::with_capacity(edges.len() / 2);
    let mut walls = HashSet::with_capacity(edges.len() / 2);
    for edge in edges.iter().copied() {
        let no_loop = graph.union_subtrees(edge.0, edge.1);
        // if no_loop {
        //     paths.insert(edge);
        // } else {
        //     walls.insert(edge);
        // }
        if !no_loop {
            walls.insert(edge);
        }
    }

    // (walls, paths)
    (walls, HashSet::new())
}
