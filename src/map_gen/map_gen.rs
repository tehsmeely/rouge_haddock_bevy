use super::cell_map::CellMap;
use crate::game::components::TileType;
use array2d::Array2D;
use log::info;

use rand::prelude::SliceRandom;
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Instant;

#[derive(Debug)]
enum AbSelector {
    A,
    B,
}
impl AbSelector {
    fn not(&self) -> Self {
        match self {
            AbSelector::A => AbSelector::B,
            AbSelector::B => AbSelector::A,
        }
    }
}
#[derive(Clone, Debug)]
pub struct AB<T: Clone> {
    a: T,
    b: T,
}

impl<T: Clone> AB<T> {
    fn new(v: T) -> Self {
        Self { a: v.clone(), b: v }
    }

    fn get(&self, selector: &AbSelector) -> &T {
        match selector {
            AbSelector::A => &self.a,
            AbSelector::B => &self.b,
        }
    }
    fn set(&mut self, selector: &AbSelector, v: T) {
        match selector {
            AbSelector::A => self.a = v,
            AbSelector::B => self.b = v,
        }
    }
}

#[derive(Debug)]
pub struct Grid {
    pub grid_size: (i32, i32),
    pub grid: Array2D<AB<TileType>>,
    current: AbSelector,
}

fn pos_is_valid(pos: (i32, i32), grid_size: (i32, i32)) -> bool {
    let x = pos.0 >= 0 && pos.0 < grid_size.0;
    let y = pos.1 >= 0 && pos.1 < grid_size.1;
    x && y
}

pub const ORTHOG_NEIGHBOURS: [(i32, i32); 4] = [(0, 1), (1, 0), (-1, 0), (0, -1)];
const NEIGHBOURS: [(i32, i32); 9] = [
    (1, 1),
    (0, 1),
    (-1, 1),
    (1, 0),
    (0, 0),
    (-1, 0),
    (1, -1),
    (0, -1),
    (-1, -1),
];

fn pos_as_usize(p: (i32, i32)) -> (usize, usize) {
    (p.0 as usize, p.1 as usize)
}

impl Grid {
    pub fn new(grid_size: (i32, i32)) -> Self {
        let mut rng = rand::thread_rng();
        let types = [TileType::WALL, TileType::WATER];
        let weights = |tile_type: &TileType| match tile_type {
            &TileType::WATER => 45,
            &TileType::WALL => 55,
        };
        let random_tile_type = || {
            let tt = types.choose_weighted(&mut rng, weights).unwrap().clone();
            AB::new(tt)
        };
        let grid = Array2D::filled_by_row_major(
            random_tile_type,
            grid_size.1 as usize,
            grid_size.0 as usize,
        );
        Self {
            grid_size,
            grid,
            current: AbSelector::A,
        }
    }

    pub fn update(&mut self) {
        for y in 0i32..self.grid_size.1 {
            for x in 0i32..self.grid_size.0 {
                let mut neighbour_count = 0;
                for (i, j) in NEIGHBOURS.iter() {
                    let pos = (x + i, y + j);
                    if pos_is_valid(pos, self.grid_size) {
                        match self.grid[pos_as_usize(pos)].get(&self.current) {
                            TileType::WATER => (),
                            TileType::WALL => neighbour_count += 1,
                        }
                    } else {
                        // Maybe remove this?
                        neighbour_count += 1;
                    }
                }
                let new_type = if neighbour_count > 4 || neighbour_count == 0 {
                    TileType::WALL
                } else {
                    TileType::WATER
                };
                self.grid[(x as usize, y as usize)].set(&self.current.not(), new_type);
            }
        }
        self.current = self.current.not();
    }

    fn draw_internal(&self, cost_map: &HashMap<(i32, i32), i32>) {
        let header = "-".repeat(self.grid_size.0 as usize + 2);
        println!("{}", header);
        for y in 0i32..self.grid_size.1 {
            print!("|");
            for x in 0i32..self.grid_size.0 {
                let print_str = cost_map
                    .get(&(x, y))
                    .map(|cost| {
                        if *cost < 10 {
                            format!("{}", cost)
                        } else {
                            String::from("!")
                        }
                    })
                    .unwrap_or_else(|| {
                        self.grid[(x as usize, y as usize)]
                            .get(&self.current)
                            .to_str()
                            .to_string()
                    });

                print!("{}", print_str)
            }
            println!("|");
        }
        println!("{}", header);
    }
    pub fn draw(&self) {
        self.draw_internal(&HashMap::new())
    }

    fn find_start(&mut self) -> Option<(i32, i32)> {
        // Traverse from bottom up, starting with the middle column, until we find an open cell
        let x0 = self.grid_size.0 / 2;
        let y0 = self.grid_size.1 - 1;

        for i in 0..self.grid_size.0 {
            for j in 0..self.grid_size.1 {
                let x = (x0 as isize + to_half_signed(i as isize)) as usize;
                let y = y0 as usize - (j as usize);
                if self.grid[(x, y)].get(&self.current) == &TileType::WATER {
                    return Some((x as i32, y as i32));
                }
            }
        }
        info!("Did not find start!");
        None
    }

    fn pos_is_valid_and(&self, pos: (i32, i32), and: TileType) -> bool {
        if pos_is_valid(pos, self.grid_size) {
            self.grid[(pos.0 as usize, pos.1 as usize)].get(&self.current) == &and
        } else {
            false
        }
    }

    fn map_and_cull(&mut self, start_point: (i32, i32)) -> HashMap<(i32, i32), i32> {
        let mut check_cells: VecDeque<((i32, i32), i32)> = VecDeque::new();
        let mut distance_map: HashMap<(i32, i32), i32> = HashMap::new();

        fn get_neighbours((x, y): (i32, i32)) -> Vec<(i32, i32)> {
            ORTHOG_NEIGHBOURS
                .into_iter()
                .map(|(i, j)| (x + i, y + j))
                .collect()
        }

        check_cells.push_back((start_point, 0));

        while let Some((cell, cost)) = check_cells.pop_front() {
            distance_map.insert(cell, cost);
            for n in get_neighbours(cell).into_iter() {
                if self.pos_is_valid_and(n, TileType::WATER) {
                    let new_cost = cost + 1;
                    let should_walk = match distance_map.get(&n) {
                        Some(prev_cost) => new_cost < *prev_cost,
                        None => true,
                    };
                    if should_walk {
                        check_cells.push_back((n, new_cost));
                    }
                }
            }
        }

        println!("Walked: {:?}", distance_map);
        self.draw_internal(&distance_map);

        // Cull any Water cells not in Distance Map
        let reachable_cells: HashSet<(i32, i32)> = distance_map.keys().cloned().collect();
        for i in 0..self.grid_size.0 {
            for j in 0..self.grid_size.1 {
                if self.grid[pos_as_usize((i, j))].get(&self.current) == &TileType::WATER && !reachable_cells.contains(&(i, j)) {
                    self.grid[pos_as_usize((i, j))].set(&self.current, TileType::WALL);
                }
            }
        }

        self.draw_internal(&distance_map);

        distance_map
    }
}

fn to_half_signed(i: isize) -> isize {
    let sign = if i % 2 == 0 { 1 } else { -1 };
    (i / 2) * sign
}

pub fn get_cell_map(min_size: usize, max_tries: i32) -> CellMap<i32> {
    for _i in 0..max_tries {
        let start = Instant::now();
        let map = run_single(min_size);
        let duration = start.elapsed();
        info!("Possibly invalid map generated in : {:?}", duration);
        if let Some(valid_map) = map {
            return valid_map;
        }
    }
    panic!(
        "Unable to generate big enough cell map within [max_tries]({})",
        max_tries
    );
}

fn run_single(min_size: usize) -> Option<CellMap<i32>> {
    let mut grid = Grid::new((20, 20));
    grid.draw();
    for _i in 0..6 {
        grid.update();
    }
    grid.draw();
    let start = grid.find_start()?;
    //println!("Start: {:?}", start);
    let cell_map = CellMap::new(grid.map_and_cull(start));
    if cell_map.cell_count() < min_size {
        return None;
    }
    //println!("cell_map: {:?}", cell_map);
    let normalised_cell_map = cell_map.normalise();
    //println!("normalise_cell_map: {:?}", normalised_cell_map);
    Some(normalised_cell_map)
}
