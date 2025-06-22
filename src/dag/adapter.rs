use crate::screen::Screen;
use std::cmp::{Reverse, max};
use std::collections::{BinaryHeap, HashSet};

#[derive(Default)]
pub(super) struct Adapter {
    pub(super) enabled: bool,
    pub(super) inputs: Vec<HashSet<i32>>,
    pub(super) outputs: Vec<HashSet<i32>>,
    pub(super) height: i32,
    pub(super) y: i32,
    pub(super) rendering: Vec<Vec<char>>,
}

const BIG: i32 = 1 << 15;

#[derive(Default, Clone)]
struct Node {
    visited: bool,
    cost: i32,
    /// indices into `edges`
    edges: Vec<usize>,
}

#[derive(Default, Clone)]
struct Edge {
    /// source node index
    a: usize,
    /// destination node index
    b: usize,
    weight: i32,
    assigned: i32,
}

fn connect(idx: usize, a: usize, b: usize, w: i32, nodes: &mut [Node], edges: &mut [Edge]) {
    edges[idx].a = a;
    edges[idx].b = b;
    edges[idx].weight = w;
    nodes[a].edges.push(idx);
    nodes[b].edges.push(idx);
}

struct Coordinator {
    width: usize,
    height: usize,
}
impl Coordinator {
    const fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }

    const fn index(&self, x: usize, y: usize, layer: usize) -> usize {
        x + self.width * (y + self.height * layer)
    }

    const fn assigned(&self, x: usize, y: usize, l: usize, edges: &[Edge]) -> bool {
        edges[self.index(x, y, l)].assigned != 0
    }
}

impl Adapter {
    pub fn construct(&mut self) {
        let width = self.inputs.len();
        let connector_len = self.highest_connector_id(width);

        /* search height starting at 3, grow until a solution appears */
        let mut height: usize = 3;
        loop {
            /* build graph */
            let nodes_count = width * height * 2;
            let edges_count = width * height * 3;
            let mut nodes: Vec<Node> = vec![Node::default(); nodes_count];
            let mut edges: Vec<Edge> = vec![Edge::default(); edges_count];

            let coord = Coordinator::new(width, height);

            for y in 0..height {
                for x in 0..width {
                    /* vertical */
                    if y != height - 1 {
                        connect(
                            coord.index(x, y, 0),
                            coord.index(x, y, 0),
                            coord.index(x, y + 1, 0),
                            1,
                            &mut nodes,
                            &mut edges,
                        );
                    }
                    /* horizontal (middle layers only) */
                    if y >= 1 && y <= height - 3 && x != width - 1 {
                        connect(
                            coord.index(x, y, 1),
                            coord.index(x, y, 1),
                            coord.index(x + 1, y, 1),
                            1,
                            &mut nodes,
                            &mut edges,
                        );
                    }
                    /* corners */
                    let dy = height as i32 / 2 - y as i32;
                    connect(
                        coord.index(x, y, 2),
                        coord.index(x, y, 0),
                        coord.index(x, y, 1),
                        10 + dy * dy,
                        &mut nodes,
                        &mut edges,
                    );
                }
            }

            /* try to route every connector one-by-one */
            let mut solution_found = true;
            for connector in 1..=connector_len {
                /* reset Dijkstra state */
                for n in &mut nodes {
                    n.visited = false;
                    n.cost = BIG;
                }

                /* start/end sets */
                let mut start = HashSet::new();
                let mut end = HashSet::new();
                for x in 0..width {
                    if self.inputs[x].contains(&connector) {
                        start.insert(coord.index(x, 0, 0));
                    }
                    if self.outputs[x].contains(&connector) {
                        end.insert(coord.index(x, height - 1, 0));
                    }
                }

                /* priority queue */
                let mut pq: BinaryHeap<(Reverse<i32>, usize)> = BinaryHeap::new();
                for &s in &start {
                    pq.push((Reverse(0), s));
                }

                while let Some((Reverse(cost), node_index)) = pq.pop() {
                    if nodes[node_index].visited {
                        continue;
                    }
                    nodes[node_index].visited = true;
                    nodes[node_index].cost = cost;
                    for &edge_index in &nodes[node_index].edges {
                        if edges[edge_index].assigned != 0 {
                            continue;
                        }
                        let v = if edges[edge_index].a == node_index {
                            edges[edge_index].b
                        } else {
                            edges[edge_index].a
                        };
                        if nodes[v].visited {
                            continue;
                        }
                        pq.push((Reverse(cost + edges[edge_index].weight), v));
                    }
                }

                /* pick the cheapest target */
                let mut best = BIG;
                let mut cur = None;
                for &e in &end {
                    if nodes[e].cost < best {
                        best = nodes[e].cost;
                        cur = Some(e);
                    }
                }
                if cur.is_none() {
                    solution_found = false;
                    break;
                }
                let mut cur = cur.unwrap();

                /* back-trace & mark path */
                while !start.contains(&cur) {
                    /* find predecessor with cost = cur.cost - weight */
                    for &eidx in &nodes[cur].edges {
                        let (a, b, w) = (edges[eidx].a, edges[eidx].b, edges[eidx].weight);
                        let prev = if cur == a { b } else { a };
                        if nodes[prev].cost + w == nodes[cur].cost {
                            edges[eidx].assigned = connector;
                            cur = prev;
                            break;
                        }
                    }
                }

                /* penalise perpendicular crossings */
                for y in 0..height {
                    for x in 0..width {
                        let e0 = coord.index(x, y, 0);
                        let e1 = coord.index(x, y, 1);
                        if edges[e0].assigned != 0 {
                            edges[e1].weight = 20;
                        }
                        if edges[e1].assigned != 0 {
                            edges[e0].weight = 20;
                        }
                    }
                }
            }
            if height > 30 {
                solution_found = true;
            }
            if !solution_found {
                height += 1;
                continue;
            }

            /* build character raster */
            self.height = height as i32;
            self.rendering = vec![vec![' '; width]; height];
            for y in 0..height {
                for x in 0..width {
                    let v = &mut self.rendering[y][x];
                    if coord.assigned(x, y, 1, &edges) {
                        *v = '─';
                    }
                    if coord.assigned(x, y, 0, &edges) {
                        *v = '│';
                    }
                    if coord.assigned(x, y, 2, &edges) {
                        if coord.assigned(x, y, 0, &edges) {
                            *v = if coord.assigned(x, y, 1, &edges) {
                                '┌'
                            } else {
                                '┐'
                            };
                        } else {
                            *v = if coord.assigned(x, y, 1, &edges) {
                                '└'
                            } else {
                                '┘'
                            };
                        }
                    }
                }
            }
            break;
        }
    }

    /// highest connector id that appears
    fn highest_connector_id(&self, width: usize) -> i32 {
        let mut connector_len = 0;
        for x in 0..width {
            for &c in &self.inputs[x] {
                connector_len = max(connector_len, c);
            }
        }
        connector_len
    }

    pub(super) fn render(&self, screen: &mut Screen) {
        for dy in 0..self.height - 1 {
            for (x, ch) in self.rendering[dy as usize].iter().enumerate() {
                if *ch != ' ' {
                    let p = screen.pixel(x, (self.y + dy) as usize);
                    *p = match (dy, *p) {
                        (0, '─') => '┬',
                        (h, '─') if h == self.height - 2 => '▽',
                        (_, _) => *ch,
                    };
                }
            }
        }
    }
}
