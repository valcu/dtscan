//! This is the R wrapper of https://github.com/randogoth/xenobalanus

use extendr_api::prelude::*;
use delaunator::{triangulate, Point as DelaunatorPoint};
use geo::Coord;
use rayon::prelude::*;
use std::cmp::{min, max};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};


// === Core types ===

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f32,
    pub y: f32
}

impl Point {
    pub fn distance(&self, point: Point) -> f32 {
        ((point.x - self.x).powi(2) + (point.y - self.y).powi(2)).sqrt()
    }
}

impl From<Point> for Coord<f32> {
    fn from(point: Point) -> Self {
        Coord { x: point.x, y: point.y }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Edge(pub usize, pub usize);

#[derive(Debug, Default, Clone)]
pub struct TriangleData {
    pub index: usize,
    pub area: Option<f32>,
    pub terminal_edge: Option<Edge>,
    pub vertices: Vec<usize>
}

impl TriangleData {
    pub fn get_edges(&self) -> Vec<Edge> {
        let mut edges = Vec::new();
        if self.vertices.len() >= 3 {
            for i in 0..self.vertices.len() {
                let v1 = self.vertices[i];
                let v2 = if i + 1 < self.vertices.len() {
                    self.vertices[i + 1]
                } else {
                    self.vertices[0]
                };
                edges.push(if v1 < v2 { Edge(v1, v2) } else { Edge(v2, v1) });
            }
        }
        edges
    }
}

#[derive(Debug, Clone)]
pub struct GeometryData {
    pub triangles: Vec<TriangleData>,
    pub edge_to_triangles: HashMap<Edge, Vec<usize>>,
    pub edge_lengths: HashMap<Edge, f32>,
    pub vertex_connections: HashMap<usize, HashSet<usize>>,
}

impl GeometryData {
    fn new() -> Self {
        GeometryData {
            triangles: Vec::new(),
            edge_to_triangles: HashMap::new(),
            edge_lengths: HashMap::new(),
            vertex_connections: HashMap::new(),
        }
    }

    fn add_triangle(&mut self, index: usize, points: &[Point], tri_idx: &[usize], types: usize) {
        let a = points[tri_idx[0]];
        let b = points[tri_idx[1]];
        let c = points[tri_idx[2]];
        let mut vertices = vec![tri_idx[0], tri_idx[1], tri_idx[2]];
        vertices.sort_unstable();

        let mut edges = [
            (Edge(min(tri_idx[0], tri_idx[1]), max(tri_idx[0], tri_idx[1])), a.distance(b)),
            (Edge(min(tri_idx[1], tri_idx[2]), max(tri_idx[1], tri_idx[2])), b.distance(c)),
            (Edge(min(tri_idx[2], tri_idx[0]), max(tri_idx[2], tri_idx[0])), c.distance(a)),
        ].to_vec();

        edges.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let terminal_edge = edges.first().map(|(e, _)| *e);

        let area = if types == 0 || types == 2 {
            Some((a.x * (b.y - c.y) + b.x * (c.y - a.y) + c.x * (a.y - b.y)).abs() / 2.0)
        } else {
            None
        };

        if types == 0 || types == 1 {
            for &(edge, length) in &edges {
                self.vertex_connections.entry(edge.0).or_default().insert(edge.1);
                self.vertex_connections.entry(edge.1).or_default().insert(edge.0);
                self.edge_lengths.insert(edge, length);
                self.edge_to_triangles.entry(edge).or_default().push(index);
            }
        } else {
            for &(edge, length) in &edges {
                self.edge_lengths.insert(edge, length);
                self.edge_to_triangles.entry(edge).or_default().push(index);
            }
        }

        if index >= self.triangles.len() {
            self.triangles.resize(index + 1, TriangleData::default());
        }

        if types == 0 || types == 2 {
            self.triangles[index] = TriangleData {
                index,
                area,
                terminal_edge,
                vertices
            };
        }
    }
}

#[extendr]
#[derive(Debug, Clone )]
pub struct dtscan {
    geometry_data: GeometryData,
    points: Vec<Point>,
    triangulation: Vec<usize>,
}


impl dtscan {
    fn new() -> Self {
        dtscan {
            geometry_data: GeometryData::new(),
            points: Vec::new(),
            triangulation: Vec::new(),
        }
    }

    fn delaunay(&mut self) {
        let pts: Vec<DelaunatorPoint> = self.points.iter()
            .map(|p| DelaunatorPoint { x: p.x as f64, y: p.y as f64 })
            .collect();
        self.triangulation = triangulate(&pts).triangles;
    }

    fn preprocess(&mut self, types: usize, parallel: bool) {
        if parallel {
            let gd = Arc::new(Mutex::new(GeometryData::new()));
            self.triangulation.par_chunks(3).enumerate().for_each(|(i, tri)| {
                let mut g = gd.lock().unwrap();
                g.add_triangle(i, &self.points, tri, types);
            });
            self.geometry_data = Arc::try_unwrap(gd).unwrap().into_inner().unwrap();
        } else {
            self.triangulation.chunks(3).enumerate().for_each(|(i, tri)| {
                self.geometry_data.add_triangle(i, &self.points, tri, types);
            });
        }
    }

    fn dtscan(&self, min_pts: usize, max_closeness: f32) -> Vec<Vec<usize>> {
        let mut clusters = Vec::new();
        let mut visited = HashSet::new();

        for (&v, neighbors) in &self.geometry_data.vertex_connections {
            if visited.contains(&v) {
                continue;
            }
            if neighbors.len() >= min_pts && neighbors.iter().all(|&n| {
                self.geometry_data.edge_lengths.get(&Edge(min(v, n), max(v, n)))
                    .map_or(false, |&l| l <= max_closeness)
            }) {
                let mut cluster = Vec::new();
                let mut to_expand = vec![v];

                while let Some(curr) = to_expand.pop() {
                    if !visited.insert(curr) {
                        continue;
                    }
                    cluster.push(curr);
                    if let Some(neigh) = self.geometry_data.vertex_connections.get(&curr) {
                        for &n in neigh {
                            if self.geometry_data.edge_lengths.get(&Edge(min(curr, n), max(curr, n)))
                                .map_or(false, |&l| l <= max_closeness)
                            {
                                to_expand.push(n);
                            }
                        }
                    }
                }
                if !cluster.is_empty() {
                    clusters.push(cluster);
                }
            }
        }
        clusters
    }
}


#[extendr]
fn new_dtscan() -> Robj {
    dtscan::new().into_robj()
}

#[extendr]
fn dtscan_delaunay(obj: &mut dtscan) {
    obj.delaunay();
}

#[extendr]
fn dtscan_preprocess(obj: &mut dtscan, types: usize, parallel: bool) {
    obj.preprocess(types, parallel);
}

#[extendr]
fn dtscan_run(obj: &dtscan, min_pts: usize, max_closeness: f32) -> List {
    obj.dtscan(min_pts, max_closeness)
        .into_iter()
        .map(|v| Robj::from(v))
        .collect()
}

#[extendr]
fn dtscan_set_points(obj: &mut dtscan, x: Robj, y: Robj) {
    let x: Vec<f64> = x.try_into().unwrap();
    let y: Vec<f64> = y.try_into().unwrap();
    obj.points = x.into_iter().zip(y).map(|(x, y)| Point { x: x as f32, y: y as f32 }).collect();
}


extendr_module! {
    mod dtscan;
    fn new_dtscan;
    fn dtscan_delaunay;
    fn dtscan_preprocess;
    fn dtscan_run;
    fn dtscan_set_points;  
}
