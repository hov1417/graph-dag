use crate::dag::{Edge, Layer, Node};
use crate::screen::Screen;
use std::cmp::{max, min};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Default)]
pub struct Context {
    labels: Vec<String>,
    id: HashMap<String, usize>,

    nodes: Vec<Node>,
    layers: Vec<Layer>,
}

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("The graph has a cycle")]
    CycleFound,
}

macro_rules! timeit {
    ($name:literal, $e:expr) => {{
        let start = std::time::Instant::now();
        let res = $e;
        let duration = start.elapsed();
        println!("{} took {:?}", $name, duration);
        res
    }};
}

impl Context {
    pub(super) fn add_node(&mut self, name: &str) {
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

    
    pub(super) fn add_vertex(&mut self, a: &str, b: &str) {
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

    pub(super) fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
    
    fn parse(&mut self, input: &str) {
        fn split<'a>(s: &'a str, pat: &str) -> Vec<&'a str> {
            s.split(pat).filter(|x| !x.is_empty()).collect()
        }

        for line in split(input, "\n") {
            let mut prev = None;
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            for part in split(line, "->") {
                let name = part.trim();
                if name.is_empty() {
                    continue;
                }
                self.add_node(name);
                if let Some(p) = prev {
                    self.add_vertex(p, name);
                }
                prev = Some(name);
            }
        }
    }

    pub(super) fn toposort(&mut self) -> Result<(), ProcessingError> {
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
                return Err(ProcessingError::CycleFound);
            }
        }
        Ok(())
    }

    pub(super) fn complete(&mut self) {
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

    pub(super) fn build_layers(&mut self) {
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
                let mut closure = HashSet::new();
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

            let mut parent_mean = vec![0f32; w];
            for (i, &n) in layer.nodes.iter().enumerate() {
                let sum: usize = self.nodes[n]
                    .upward
                    .iter()
                    .map(|&p| self.nodes[p].row)
                    .sum();
                parent_mean[i] = sum as f32 / (self.nodes[n].upward.len() as f32 + 0.01);
            }

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

    pub(super) fn resolve_crossings(&mut self) {
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

    pub(super) fn layout(&mut self) {
        for (i, node) in self.nodes.iter_mut().enumerate() {
            if node.is_connector {
                node.width = 1;
            } else {
                let chars = self.labels[i].chars().count() as i32;
                let mut width = chars;
                width = max(width, node.upward.len() as i32);
                width = max(width, node.downward.len() as i32);
                // add at least 2 spaces as margin
                while width - chars < 2 {
                    width += 1;
                }
                // width and chars should have same width, for centering
                if width % 2 != chars % 2 {
                    width += 1;
                }
                // additional 2 width for border
                node.width = width + 2;
            }
            node.height = 3;
        }

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

            let mut id_map: HashMap<(usize, usize), i32> = HashMap::new();
            let mut next_id = 1;
            let mut get_id = |map: &mut HashMap<_, _>, a, b| -> i32 {
                *map.entry((a, b)).or_insert_with(|| {
                    let id = next_id;
                    next_id += 1;
                    id
                })
            };

            let mut inputs = vec![HashSet::new(); width as usize];
            let mut outputs = vec![HashSet::new(); width as usize];

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

        let mut y_position = 0;
        for layer in &mut self.layers {
            for &n in &layer.nodes {
                self.nodes[n].y = y_position;
            }
            for e in &mut layer.edges {
                e.y = y_position + 2;
            }
            if layer.adapter.enabled {
                layer.adapter.y = y_position + 2;
                y_position += layer.adapter.height - 3;
            }
            y_position += 3;
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
        self.layout_nodes_do_not_touch()
    }
    fn layout_grow_nodes(&mut self) -> bool {
        for layer in &self.layers {
            for &edge in &layer.edges {
                let node_indexes = [edge.up, edge.down];
                for node_index in node_indexes {
                    let node = &mut self.nodes[node_index];
                    if node.x + node.width - 1 - 1 < edge.x && !node.is_connector {
                        let parity = node.width % 2;
                        node.width = edge.x + 1 + 1 - node.x;
                        if parity != node.width % 2 {
                            node.width += 1;
                        }
                        return false;
                    }
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

    pub(super) fn render(&self) -> String {
        /* total size */
        let mut w = 0;
        let mut h = 0;
        for n in &self.nodes {
            w = max(w, n.x + n.width);
            h = max(h, n.y + n.height);
        }

        let mut screen = Screen::new(w as usize, h as usize);

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
                screen.draw_text_in_box_center(
                    n.x as usize,
                    n.y as usize,
                    n.width as usize,
                    &self.labels[i],
                );
            }
        }

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
                screen.draw_pixel(e.x as usize, e.y as usize, up);
                screen.draw_pixel(e.x as usize, (e.y + 1) as usize, down);
            }
        }

        for layer in &self.layers {
            if layer.adapter.enabled {
                layer.adapter.render(&mut screen);
            }
        }

        screen.stringify()
    }

    pub fn process(input: &str) -> Result<String, ProcessingError> {
        // todo debug logging
        let mut ctx = Self::default();
        timeit!("parse", ctx.parse(input));
        if ctx.is_empty() {
            return Ok(String::new());
        }
        ctx.toposort()?;
        timeit!("complete", ctx.complete());
        timeit!("build_layers", ctx.build_layers());
        timeit!("resolve_crossings", ctx.resolve_crossings());
        timeit!("layout", ctx.layout());
        let res = timeit!("render", ctx.render());
        Ok(res)
    }
}
