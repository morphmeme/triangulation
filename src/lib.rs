mod utils;

use wasm_bindgen::prelude::*;
use itertools::Itertools;
use std::alloc::System;
use std::ops::Rem;
use log::Level;
use log::debug;
use std::fmt;


// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_log::init_with_level(Level::Debug);
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
#[derive(Debug, Copy, Clone)]
pub struct Point (pub f32, pub f32);

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}

#[wasm_bindgen]
impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            0:x, 1:y
        }
    }

    pub fn x(&self) -> f32 {
        self.0
    }

    pub fn y(&self) -> f32 {
        self.1
    }
}

impl Point {
    pub fn double_area(a : &Point, b : &Point, c : &Point) -> f32 {
        (c.0 - b.0) * (a.1 - b.1) - (a.0 - b.0) * (c.1 - b.1)
    }

    pub fn left(a: &Point, b: &Point, maybe_left: &Point) -> bool {
        Self::double_area(a, b, maybe_left) > std::f32::EPSILON
    }

    pub fn left_on(a: &Point, b: &Point, maybe_left: &Point) -> bool {
        Self::double_area(a, b, maybe_left) >= -std::f32::EPSILON
    }

    pub fn collinear(a: &Point, b: &Point, c: &Point) -> bool {
        Self::double_area(a, b, c).abs() < std::f32::EPSILON
    }

    fn proper_intersect(a0: &Point, a1: &Point, b0: &Point, b1: &Point) -> bool {
        if Self::collinear(a0, a1, b0) ||
            Self::collinear(a0, a1, b1) ||
            Self::collinear(b0, b1, a0) ||
            Self::collinear(b0, b1, a1) {
            return false;
        }


        (Self::left(a0, a1, b0) ^ Self::left(a0, a1, b1))
        && (Self::left(b0, b1, a0) ^ Self::left(b0, b1, a1))
    }

    pub fn between(a: &Point, b: &Point, maybe_between: &Point) -> bool {
        if !Self::collinear(a, b, maybe_between) {
            return false;
        }

        if a.0 != b.0 {
            ((a.0 <= maybe_between.0) && (maybe_between.0 <= b.0)) || ((a.0 >= maybe_between.0) && (maybe_between.0 >= b.0))
        } else {
            ((a.1 <= maybe_between.1) && (maybe_between.1 <= b.1)) || ((a.1 >= maybe_between.1) && (maybe_between.1 >= b.1))
        }
    }

    pub fn intersect(a0: &Point, a1: &Point, b0: &Point, b1: &Point) -> bool {
        Self::proper_intersect(a0, a1, b0, b1) ||
            (Self::between(a0, a1, b0)
            || Self::between(a0, a1, b1)
            || Self::between(b0, b1, a0)
            || Self::between(b0, b1, a1))
    }
    pub fn polar_angle(a: &Point, centroid: &Point) -> f32 {
        (a.1 - centroid.1).atan2(a.0 - centroid.0)
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        (self.0 - other.0).abs() < std::f32::EPSILON
        && (self.1 - other.1).abs() < std::f32::EPSILON
    }
}

impl Eq for Point {}

#[wasm_bindgen]
#[derive(Debug)]
pub struct Polygon {
    points : Vec<Point>
}

#[wasm_bindgen]
impl Polygon {
    pub fn empty() -> Self {
        Self {
            points : vec![]
        }
    }

    pub fn from_slice(slice: &[f32]) -> Self {
        Self {
            points: slice.chunks(2).map(|p| Point(p[0], p[1])).collect()
        }
    }

    pub fn push(&mut self, a : Point) {
        self.points.push(a);
    }

    pub fn ccw_sort(&mut self) {
        let centroid = self.centroid();
        let angles = self.points.iter()
            .map(|p| (Point::polar_angle(&p, &centroid))).collect::<Vec<f32>>();
        self.points = self.points.iter().enumerate().sorted_by(|(i, a), (j, b)| {
            let a_angle = angles.get(*i).unwrap();
            let b_angle = angles.get(*j).unwrap();
            a_angle.partial_cmp(b_angle).unwrap()
        }).map(|(_, p)| *p).collect();
    }

    pub fn nb_triangulations(&self) -> usize {
        let mut table = vec![vec![0; self.points.len()]; self.points.len()];
        let mut calculated = vec![vec![0; self.points.len()]; self.points.len()];

        for i in 0..self.points.len() {
            for k in i+1..self.points.len() {
                self.nb_tri_helper(i, k, &mut table, &mut calculated);
            }
        }

        table[0][1]
    }
}

impl Polygon {
    pub fn new(points : Vec<Point>) -> Self {
        Self { points }
    }

    pub fn double_area(&self) -> f32 {
        self.points
            .iter()
            .skip(1)
            .tuple_windows::<(_, _,)>()
            .map(|(a, b)| {
                Point::double_area(&self.points[0], a, b)
            })
            .sum()
    }

    pub fn diagonalie(&self, a : usize, b: usize) -> bool {
        //TODO handle unwraps;
        let a_vertex = self.points.get(a).unwrap();
        let b_vertex = self.points.get(b).unwrap();

        for (c0, c1) in self.points.iter().tuple_windows::<(_, _,)>() {
            if (c0 != a_vertex) && (c1 != a_vertex) && (c0 != b_vertex) && (c1 != b_vertex)
                && Point::intersect(a_vertex, b_vertex, c0, c1) {
                return false;
            }
        }
        return true;
    }

    pub fn in_cone(&self, a: usize, b: usize) -> bool {
        let mut a_it = self.points.iter().cycle().skip(a);
        let mut a_rev_it = self.points.iter().rev().cycle().skip(self.points.len() - a);
        // TODO handle unwraps
        let a_prev_vertex = a_rev_it.next().unwrap();
        let a_vertex = a_it.next().unwrap();
        let a_next_vertex = a_it.next().unwrap();
        let b_vertex = self.points.get(b).unwrap();

        if Point::left_on(a_vertex, a_next_vertex, a_prev_vertex) {

            return Point::left(a_vertex, b_vertex, a_prev_vertex)
                && Point::left(b_vertex, a_vertex, a_next_vertex);
        }
        return !(Point::left_on(a_vertex, b_vertex, a_next_vertex)
            && Point::left_on(b_vertex, a_vertex, a_prev_vertex))
    }

    pub fn diagonal(&self, a: usize, b: usize) -> bool {
        return self.in_cone(a, b) && self.in_cone(b, a) && self.diagonalie(a, b);
    }

    // TODO just convert polygon to ccw
    pub fn left_polygon(&self, a: usize, b: usize) -> Vec<usize> {
        self.points.iter()
            .enumerate()
            .cycle()
            .skip(b+1)
            .take_while(|(idx, p)| idx != &a)
            .map(|(idx, p)| idx)
            .collect()
    }

    // TODO could do one iteration with point arithmetic
    pub fn centroid(&self) -> Point {
        let x_mean = self.points.iter().fold(0.0, |acc, p| acc + p.0) / (self.points.len() as f32);
        let y_mean = self.points.iter().fold(0.0, |acc, p| acc + p.1) / (self.points.len() as f32);
        Point::new(x_mean, y_mean)
    }

    pub fn edge(&self, a: usize, b: usize) -> bool {
        let diff = (b as i32 - a as i32).abs();
        diff == 1 || diff == (self.points.len() - 1) as i32
    }

    fn nb_tri_helper(&self, i : usize, k: usize, table : &mut Vec<Vec<usize>>, calculated : &mut Vec<Vec<usize>>) -> usize {
        if self.left_polygon(i, k).len() == 0 {
            calculated[i][k] = 1;
            table[i][k] = 1;
        }
        if calculated[i][k] == 1 {
            return table[i][k];
        }
        for j in self.left_polygon(i, k) {
            if (self.edge(i,j) || self.diagonal(i, j))
                && (self.edge(j, k) || self.diagonal(j, k)) {
                table[i][k] += self.nb_tri_helper(i, j, table, calculated) * self.nb_tri_helper(j, k, table, calculated);
            }
        }
        calculated[i][k] = 1;
        table[i][k]
    }
}
