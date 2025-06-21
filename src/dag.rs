//! dag.rs  – ASCII DAG-to-text renderer (Rust port of Arthur Sonzogni’s code)

use std::cmp::{Reverse, max, min};
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap};

use crate::screen::Screen;
// keep the first file in `screen.rs`

/* ------------------------------------------------------------------------- */
/* -- data structures ------------------------------------------------------ */

#[derive(Default)]
struct Node {
    /* parsing */
    upward: BTreeSet<usize>,
    downward: BTreeSet<usize>,
    is_connector: bool,
    padding: i32,

    /* layering */
    layer: usize,
    row: usize,
    downward_closure: BTreeSet<usize>,
    upward_sorted: Vec<usize>,
    downward_sorted: Vec<usize>,

    /* rendering */
    width: i32,
    height: i32,
    x: i32,
    y: i32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Edge {
    up: usize,
    down: usize,
    x: i32,
    y: i32,
}

#[derive(Default)]
struct Adapter {
    enabled: bool,
    inputs: Vec<BTreeSet<i32>>,
    outputs: Vec<BTreeSet<i32>>,
    height: i32,
    y: i32,
    rendering: Vec<Vec<char>>,
}

#[derive(Default)]
struct Layer {
    nodes: Vec<usize>,
    edges: Vec<Edge>,
    adapter: Adapter,
}

#[derive(Default)]
pub struct Context {
    /* ids & labels */
    labels: Vec<String>,
    id: HashMap<String, usize>,

    nodes: Vec<Node>,
    layers: Vec<Layer>,
}

/* ------------------------------------------------------------------------- */
/* -- helpers -------------------------------------------------------------- */
fn split<'a>(s: &'a str, pat: &str) -> Vec<&'a str> {
    s.split(pat).filter(|x| !x.is_empty()).collect()
}

/* ------------------------------------------------------------------------- */
/* -- context methods ------------------------------------------------------ */

impl Context {
    /* ------------- construction ------------- */
    fn add_node(&mut self, name: &str) {
        if self.id.contains_key(name) {
            return;
        }
        let idx = self.nodes.len();
        self.nodes.push(Node {
            padding: 1,
            ..Default::default()
        });
        self.id.insert(name.into(), idx);
        self.labels.push(name.into());
    }
    fn add_vertex(&mut self, a: &str, b: &str) {
        let ia = self.id[a];
        let ib = self.id[b];
        self.nodes[ia].downward.insert(ib);
        self.nodes[ib].upward.insert(ia);
    }
    fn add_connector(&mut self, a: usize, b: usize) {
        let c = self.nodes.len();
        self.nodes.push(Node {
            is_connector: true,
            padding: 0,
            layer: self.nodes[a].layer + 1,
            ..Default::default()
        });
        self.labels.push("connector".into());

        self.nodes[a].downward.remove(&b);
        self.nodes[b].upward.remove(&a);

        self.nodes[a].downward.insert(c);
        self.nodes[c].upward.insert(a);

        self.nodes[c].downward.insert(b);
        self.nodes[b].upward.insert(c);
    }

    fn parse(&mut self, input: &str) {
        for line in split(input, "\n") {
            let mut prev = None;
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            for part in split(line, "->") {
                let name = part.trim();
                self.add_node(name);
                if let Some(p) = prev {
                    self.add_vertex(p, name);
                }
                prev = Some(name);
            }
        }
    }

    fn toposort(&mut self) -> bool {
        let mut changed = true;
        let mut iter = 0;
        while changed {
            changed = false;
            for a in 0..self.nodes.len() {
                let downward = self.nodes[a].downward.clone();
                for &b in &downward {
                    if self.nodes[b].layer <= self.nodes[a].layer {
                        self.nodes[b].layer = self.nodes[a].layer + 1;
                        changed = true;
                    }
                }
            }
            iter += 1;
            if iter > self.nodes.len() * self.nodes.len() {
                return false;
            }
        }
        true
    }

    fn complete(&mut self) {
        loop {
            let mut again = false;
            for a in 0..self.nodes.len() {
                let layer_a = self.nodes[a].layer;
                let downs: Vec<usize> = self.nodes[a].downward.clone().into_iter().collect();
                for b in downs {
                    if layer_a + 1 != self.nodes[b].layer {
                        self.add_connector(a, b);
                        again = true;
                        break;
                    }
                }
            }
            if !again {
                break;
            }
        }
    }

    /* ------------- build layers & ordering ------------- */
    fn build_layers(&mut self) {
        let last_layer = self.nodes.iter().map(|n| n.layer).max().unwrap_or(0);
        self.layers.resize_with(last_layer + 1, Default::default);
        for (i, n) in self.nodes.iter().enumerate() {
            self.layers[n.layer].nodes.push(i);
        }
        self.optimize_row_order();

        let rows = self.nodes.iter().map(|n| n.row).collect::<Vec<_>>();
        /* sort adj lists */
        for node in &mut self.nodes {
            node.upward_sorted = node.upward.iter().copied().collect();
            node.downward_sorted = node.downward.iter().copied().collect();
            node.upward_sorted.sort_by_key(|&i| rows[i]);
            node.downward_sorted.sort_by_key(|&i| rows[i]);
        }
        /* fill edges */
        for layer in &mut self.layers {
            for &up in &layer.nodes {
                for &down in &self.nodes[up].downward_sorted {
                    layer.edges.push(Edge {
                        up,
                        down,
                        x: 0,
                        y: 0,
                    });
                }
            }
        }
    }

    fn optimize_row_order(&mut self) {
        /* downward closure, from next-to-last layer up */
        for y in (0..self.layers.len().saturating_sub(1)).rev() {
            for &up in &self.layers[y].nodes {
                let mut closure = BTreeSet::new();
                for &d in &self.nodes[up].downward {
                    closure.insert(d);
                    closure.extend(self.nodes[d].downward_closure.iter().copied());
                }
                self.nodes[up].downward_closure = closure;
            }
        }

        for layer in &mut self.layers {
            let w = layer.nodes.len();
            if w <= 1 {
                continue;
            }

            /* parent barycentres */
            let mut parent_mean = vec![0f32; w];
            for (i, &n) in layer.nodes.iter().enumerate() {
                let sum: usize = self.nodes[n]
                    .upward
                    .iter()
                    .map(|&p| self.nodes[p].row)
                    .sum();
                parent_mean[i] = sum as f32 / (self.nodes[n].upward.len() as f32 + 0.01);
            }

            /* distance matrix */
            let big = self.nodes.len() * 2;
            let mut dist = vec![vec![big; w]; w];
            for a in 0..w {
                for b in 0..w {
                    let na = &self.nodes[layer.nodes[a]];
                    let nb = &self.nodes[layer.nodes[b]];
                    let mut best = big;
                    for &c in &na.downward_closure {
                        if nb.downward_closure.contains(&c) {
                            best = min(best, self.nodes[c].layer - na.layer);
                        }
                    }
                    dist[a][b] = best;
                }
            }

            /* heuristic permutation search (swap-improve) */
            let mut perm: Vec<usize> = (0..w).collect();
            let score = |perm: &[usize]| -> f32 {
                let mut s = 0f32;
                for i in 0..w - 1 {
                    s += dist[perm[i]][perm[i + 1]] as f32;
                }
                for i in 0..w {
                    let d = i as f32 - parent_mean[perm[i]];
                    s += d * d * 15.0;
                }
                s
            };
            let mut current = score(&perm);
            loop {
                let mut improved = false;
                for a in 0..w {
                    for b in a + 1..w {
                        perm.swap(a, b);
                        let ns = score(&perm);
                        if ns < current {
                            current = ns;
                            improved = true;
                        } else {
                            perm.swap(a, b);
                        }
                    }
                }
                if !improved {
                    break;
                }
            }

            /* apply order */
            let new_nodes: Vec<usize> = perm.into_iter().map(|i| layer.nodes[i]).collect();
            layer.nodes = new_nodes;

            /* row field */
            for (i, &n) in layer.nodes.iter().enumerate() {
                self.nodes[n].row = i;
            }
        }
    }

    /* ------------- crossing detection ------------- */
    fn resolve_crossings(&mut self) {
        for layer in &mut self.layers {
            let mut up = layer.edges.clone();
            let mut down = layer.edges.clone();
            up.sort_by_key(|e| (self.nodes[e.up].row, self.nodes[e.down].row));
            down.sort_by_key(|e| (self.nodes[e.down].row, self.nodes[e.up].row));
            if up != down {
                layer.edges.clear();
                layer.adapter.enabled = true;
            }
        }
    }

    /* ------------- size & x-positions ------------- */
    fn layout(&mut self) {
        /* initial widths */
        for (i, node) in self.nodes.iter_mut().enumerate() {
            if node.is_connector {
                node.width = 1;
            } else {
                let mut w = self.labels[i].chars().count() as i32;
                w = max(w, node.upward.len() as i32);
                w = max(w, node.downward.len() as i32);
                node.width = w + 2;
            }
            node.height = 3;
        }

        /* iterative constraints */
        for _ in 0..1000 {
            if self.layout_nodes_do_not_touch()
                && self.layout_edges_do_not_touch()
                && self.layout_grow_nodes()
                && self.layout_shift_edges()
                && self.layout_shift_connector_nodes()
            {
                break;
            }
        }

        /* adapters input/output sets */
        for y in 0..self.layers.len() - 1 {
            let up = &self.layers[y];
            let down = &self.layers[y + 1];
            if !up.adapter.enabled {
                continue;
            }

            let mut width = 0;
            for &n in &up.nodes {
                width = max(width, self.nodes[n].x + self.nodes[n].width);
            }
            for &n in &down.nodes {
                width = max(width, self.nodes[n].x + self.nodes[n].width);
            }

            let mut id_map: BTreeMap<(usize, usize), i32> = BTreeMap::new();
            let mut next_id = 1;
            let mut get_id = |map: &mut BTreeMap<_, _>, a, b| -> i32 {
                *map.entry((a, b)).or_insert_with(|| {
                    let id = next_id;
                    next_id += 1;
                    id
                })
            };

            let mut inputs = vec![BTreeSet::new(); width as usize];
            let mut outputs = vec![BTreeSet::new(); width as usize];

            for &a in &up.nodes {
                let n = &self.nodes[a];
                for x in n.x + n.padding..n.x + n.width - n.padding {
                    for &b in &n.downward {
                        inputs[x as usize].insert(get_id(&mut id_map, a, b));
                    }
                }
            }
            for &b in &down.nodes {
                let n = &self.nodes[b];
                for x in n.x + n.padding..n.x + n.width - n.padding {
                    for &a in &n.upward {
                        outputs[x as usize].insert(get_id(&mut id_map, a, b));
                    }
                }
            }

            let adapter = &mut self.layers[y].adapter;
            adapter.inputs = inputs;
            adapter.outputs = outputs;
            adapter.construct();
        }

        /* y positions */
        let mut ycur = 0;
        for layer in &mut self.layers {
            for &n in &layer.nodes {
                self.nodes[n].y = ycur;
            }
            for e in &mut layer.edges {
                e.y = ycur + 2;
            }
            if layer.adapter.enabled {
                layer.adapter.y = ycur + 2;
                ycur += layer.adapter.height - 3;
            }
            ycur += 3;
        }
    }

    /* ---- layout sub-steps (return false if they changed something) ---- */
    fn layout_nodes_do_not_touch(&mut self) -> bool {
        let mut stable = true;
        for layer in &mut self.layers {
            let mut x = 0;
            for &n in &layer.nodes {
                if self.nodes[n].x < x {
                    self.nodes[n].x = x;
                    stable = false;
                }
                x = self.nodes[n].x + self.nodes[n].width;
            }
        }
        stable
    }
    fn layout_edges_do_not_touch(&mut self) -> bool {
        /* identical to nodes step */
        self.layout_nodes_do_not_touch()
    }
    fn layout_grow_nodes(&mut self) -> bool {
        for layer in &self.layers {
            for &edge in &layer.edges {
                let node_up = &mut self.nodes[edge.up];
                if node_up.x + node_up.width - 1 - 1 < edge.x && !node_up.is_connector {
                    node_up.width = edge.x + 1 + 1 - node_up.x;
                    return false;
                }

                let node_down = &mut self.nodes[edge.down];
                if node_down.x + node_down.width - 1 - 1 < edge.x && !node_down.is_connector {
                    node_down.width = edge.x + 1 + 1 - node_down.x;
                    return false;
                }
            }
        }
        true
    }
    fn layout_shift_edges(&mut self) -> bool {
        for layer in &mut self.layers {
            for e in &mut layer.edges {
                let minx = max(
                    self.nodes[e.up].x + self.nodes[e.up].padding,
                    self.nodes[e.down].x + self.nodes[e.down].padding,
                );
                if e.x < minx {
                    e.x = minx;
                    return false;
                }
            }
        }
        true
    }
    fn layout_shift_connector_nodes(&mut self) -> bool {
        for i in 0..self.nodes.len() {
            if !self.nodes[i].is_connector {
                continue;
            }
            let layer = self.nodes[i].layer;
            let mut minx = 0;
            for e in &self.layers[layer - 1].edges {
                if e.down == i {
                    minx = max(minx, e.x);
                }
            }
            for e in &self.layers[layer].edges {
                if e.up == i {
                    minx = max(minx, e.x);
                }
            }
            if self.nodes[i].x < minx {
                self.nodes[i].x = minx;
                return false;
            }
        }
        true
    }

    /* ------------- adapter build & render ------------- */
    fn render(&self) -> String {
        /* total size */
        let mut w = 0;
        let mut h = 0;
        for n in &self.nodes {
            w = max(w, n.x + n.width);
            h = max(h, n.y + n.height);
        }

        let mut screen = Screen::new(w as usize, h as usize);

        /* draw nodes */
        for (i, n) in self.nodes.iter().enumerate() {
            if n.is_connector {
                if n.width == 1 {
                    screen.draw_vertical_line(n.y as usize, (n.y + 2) as usize, n.x as usize, '│');
                } else {
                    screen.draw_box(
                        n.x as usize,
                        n.y as usize,
                        n.width as usize,
                        n.height as usize,
                    );
                }
            } else {
                screen.draw_box(
                    n.x as usize,
                    n.y as usize,
                    n.width as usize,
                    n.height as usize,
                );
                screen.draw_text((n.x + 1) as usize, (n.y + 1) as usize, &self.labels[i]);
            }
        }

        /* draw edges */
        for layer in &self.layers {
            for e in &layer.edges {
                let up = if self.nodes[e.up].is_connector {
                    '│'
                } else {
                    '┬'
                };
                let down = if self.nodes[e.down].is_connector {
                    '│'
                } else {
                    '▽'
                };
                screen.draw_pixel(e.x as usize, (e.y) as usize, up);
                screen.draw_pixel(e.x as usize, (e.y + 1) as usize, down);
            }
        }

        /* adapters */
        for layer in &self.layers {
            if layer.adapter.enabled {
                layer.adapter.render(&mut screen);
            }
        }

        screen.stringify()
    }

    /* ------------- public entry ------------- */
    pub fn process(input: &str) -> String {
        macro_rules! timeit {
            ($name:literal, $e:expr) => {{
                let start = std::time::Instant::now();
                let res = $e;
                let duration = start.elapsed();
                println!("{} took {:?}", $name, duration);
                res
            }};
        }
        let mut ctx = Self::default();
        timeit!("parse", ctx.parse(input));
        if ctx.nodes.is_empty() {
            return String::new();
        }
        if !ctx.toposort() {
            // TODO, error
            return "There are cycles".into();
        }
        timeit!("complete", ctx.complete());
        timeit!("build_layers", ctx.build_layers());
        timeit!("resolve_crossings", ctx.resolve_crossings());
        timeit!("layout", ctx.layout());
        let res = timeit!("render", ctx.render());
        res
    }
}

/* ------------------------------------------------------------------------- */
/* -- adapter impl --------------------------------------------------------- */

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

        /* local graph types ------------------------------------------------ */
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

        /* search height starting at 3, grow until a solution appears ------- */
        let big = 1 << 15;
        let mut height: usize = 3;
        loop {
            /* build graph -------------------------------------------------- */
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

            /* try to route every connector one-by-one ---------------------- */
            let mut solution_found = true;
            for connector in 1..=connector_len {
                /* reset Dijkstra state */
                for n in &mut nodes {
                    n.visited = false;
                    n.cost = big;
                }

                /* start/end sets */
                let mut start = BTreeSet::new();
                let mut end = BTreeSet::new();
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

            /* build character raster -------------------------------------- */
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
    fn render(&self, screen: &mut Screen) {
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

/* ------------------------------------------------------------------------- */
/* -- convenience wrapper -------------------------------------------------- */

pub fn dag_to_text(s: &str) -> String {
    Context::process(s)
}
