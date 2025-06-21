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
impl Adapter {
    pub fn construct(&mut self) {
        let width = self.inputs.len();
        /* highest connector id that appears */
        let mut connector_len = 0;
        for x in 0..width {
            for &c in &self.inputs[x] {
                connector_len = max(connector_len, c);
            }
        }

        #[derive(Default, Clone)]
        struct Node {
            visited: bool,
            cost: i32,
            edges: Vec<usize>, // indices into `edges`
        }
        #[derive(Default, Clone)]
        struct Edge {
            a: usize, // node index
            b: usize, // node index
            weight: i32,
            assigned: i32,
        }

        /* search height starting at 3, grow until a solution appears */
        let big = 1 << 15;
        let mut height: usize = 3;
        loop {
            /* build graph */
            let nodes_count = width * height * 2;
            let edges_count = width * height * 3;
            let mut nodes: Vec<Node> = vec![Node::default(); nodes_count];
            let mut edges: Vec<Edge> = vec![Edge::default(); edges_count];

            let index =
                |x: usize, y: usize, layer: usize| -> usize { x + width * (y + height * layer) };

            let connect =
                |idx: usize, a: usize, b: usize, w: i32, nodes: &mut [Node], edges: &mut [Edge]| {
                    edges[idx].a = a;
                    edges[idx].b = b;
                    edges[idx].weight = w;
                    nodes[a].edges.push(idx);
                    nodes[b].edges.push(idx);
                };

            for y in 0..height {
                for x in 0..width {
                    /* vertical */
                    if y != height - 1 {
                        connect(
                            index(x, y, 0),
                            index(x, y, 0),
                            index(x, y + 1, 0),
                            1,
                            &mut nodes,
                            &mut edges,
                        );
                    }
                    /* horizontal (middle layers only) */
                    if y >= 1 && y <= height - 3 && x != width - 1 {
                        connect(
                            index(x, y, 1),
                            index(x, y, 1),
                            index(x + 1, y, 1),
                            1,
                            &mut nodes,
                            &mut edges,
                        );
                    }
                    /* corners */
                    let dy = height as i32 / 2 - y as i32;
                    connect(
                        index(x, y, 2),
                        index(x, y, 0),
                        index(x, y, 1),
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
                    n.cost = big;
                }

                /* start/end sets */
                let mut start = HashSet::new();
                let mut end = HashSet::new();
                for x in 0..width {
                    if self.inputs[x].contains(&connector) {
                        start.insert(index(x, 0, 0));
                    }
                    if self.outputs[x].contains(&connector) {
                        end.insert(index(x, height - 1, 0));
                    }
                }

                /* priority queue */
                let mut pq: BinaryHeap<(Reverse<i32>, usize)> = BinaryHeap::new();
                for &s in &start {
                    pq.push((Reverse(0), s));
                }

                while let Some((Reverse(cost), nidx)) = pq.pop() {
                    if nodes[nidx].visited {
                        continue;
                    }
                    nodes[nidx].visited = true;
                    nodes[nidx].cost = cost;
                    for &eidx in &nodes[nidx].edges {
                        if edges[eidx].assigned != 0 {
                            continue;
                        }
                        let v = if edges[eidx].a == nidx {
                            edges[eidx].b
                        } else {
                            edges[eidx].a
                        };
                        if nodes[v].visited {
                            continue;
                        }
                        pq.push((Reverse(cost + edges[eidx].weight), v));
                    }
                }

                /* pick cheapest target */
                let mut best = big;
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
                        let (a, b, w) = {
                            let e = &edges[eidx];
                            (e.a, e.b, e.weight)
                        };
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
                        let e0 = index(x, y, 0);
                        let e1 = index(x, y, 1);
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
            let assigned = |x: usize, y: usize, l: usize, edges: &[Edge]| -> bool {
                edges[index(x, y, l)].assigned != 0
            };
            for y in 0..height {
                for x in 0..width {
                    let v = &mut self.rendering[y][x];
                    if assigned(x, y, 1, &edges) {
                        *v = '─';
                    }
                    if assigned(x, y, 0, &edges) {
                        *v = '│';
                    }
                    if assigned(x, y, 2, &edges) {
                        if assigned(x, y, 0, &edges) {
                            *v = if assigned(x, y, 1, &edges) {
                                '┌'
                            } else {
                                '┐'
                            };
                        } else {
                            *v = if assigned(x, y, 1, &edges) {
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
