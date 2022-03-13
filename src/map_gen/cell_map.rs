use rand::prelude::SliceRandom;
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;

#[derive(Debug)]
pub struct CellMap<V>(pub HashMap<(i32, i32), V>);

impl<V> CellMap<V> {
    pub fn new(m: HashMap<(i32, i32), V>) -> Self {
        Self(m)
    }

    pub fn cell_count(&self) -> usize {
        self.0.len()
    }

    pub fn offset(self, offset: (i32, i32)) -> Self {
        let new_map: HashMap<(i32, i32), V> = self
            .0
            .into_iter()
            .map(|((x, y), v)| ((x + offset.0, y + offset.1), v))
            .collect();
        Self(new_map)
    }

    pub fn normalise(self) -> Self {
        //Adjust a hashmap of cell positions so they align with 0 on x and y
        let x_offset = self.0.keys().map(|(x, _)| x).min().cloned().unwrap_or(0);
        let y_offset = self.0.keys().map(|(_, y)| y).min().cloned().unwrap_or(0);
        if x_offset != 0 && y_offset != 0 {
            self.offset((-x_offset, -y_offset))
        } else {
            self
        }
    }

    pub fn rect_size(&self) -> (usize, usize) {
        let x_min = self.0.keys().map(|(x, _)| x).min().cloned().unwrap_or(0);
        let y_min = self.0.keys().map(|(_, y)| y).min().cloned().unwrap_or(0);
        let x_max = self.0.keys().map(|(x, _)| x).max().cloned().unwrap_or(0);
        let y_max = self.0.keys().map(|(_, y)| y).max().cloned().unwrap_or(0);
        ((x_max - x_min) as usize, (y_max - y_min) as usize)
    }

    pub fn get_all_cells(&self) -> Vec<(i32, i32)> {
        self.0.keys().cloned().collect()
    }

    pub fn contains(&self, cell: &(i32, i32)) -> bool {
        self.0.contains_key(cell)
    }
}

impl<V> CellMap<V>
where
    V: Ord,
{
    pub fn start_point(&self) -> Option<(i32, i32)> {
        let min_val = self.0.values().min()?;
        for (key, v) in self.0.iter() {
            if v == min_val {
                return Some(key.clone());
            }
        }
        None
    }
    pub fn end_point(&self) -> Option<(i32, i32)> {
        let max_val = self.0.values().max()?;
        for (key, v) in self.0.iter() {
            if v == max_val {
                return Some(key.clone());
            }
        }
        None
    }
}

impl CellMap<i32> {
    pub fn recalculate(&self, start_point: (i32, i32)) -> Self {
        let mut check_cells: VecDeque<((i32, i32), i32)> = VecDeque::new();
        let mut new_self: HashMap<(i32, i32), i32> = HashMap::new();

        fn get_neighbours((x, y): (i32, i32)) -> Vec<(i32, i32)> {
            super::map_gen::ORTHOG_NEIGHBOURS
                .into_iter()
                .map(|(i, j)| (x + i, y + j))
                .collect()
        }

        check_cells.push_back((start_point, 0));

        while let Some((cell, cost)) = check_cells.pop_front() {
            new_self.insert(cell, cost);
            for n in get_neighbours(cell).into_iter() {
                if self.0.contains(&n) {
                    let new_cost = cost + 1;
                    let should_walk = match new_self.get(&n) {
                        Some(prev_cost) => new_cost < *prev_cost,
                        None => true,
                    };
                    if should_walk {
                        check_cells.push_back((n, new_cost));
                    }
                }
            }
        }
        Self(new_self)
    }

    pub fn distribute_points_by_cost(&self, n: usize) -> Vec<(i32, i32)> {
        // Find min, max, and mid cost
        // Fetch all cells where min < cost < max (i.e drop min/max)
        // pick with weigh: 1/ distance from mid
        // TODO: Error handle here.
        let min_cost = self.0.values().min().cloned().unwrap();
        let max_cost = self.0.values().max().cloned().unwrap();
        let mid_cost = min_cost + (max_cost - min_cost) / 2;
        let positions: Vec<(i32, i32)> = self
            .0
            .iter()
            .filter_map(|(k, v)| {
                if *v > min_cost && *v < max_cost {
                    Some(k.clone())
                } else {
                    None
                }
            })
            .collect();

        let weights = |pos: &(i32, i32)| match self.0.get(pos) {
            Some(val) => mid_cost - (mid_cost - val).abs(),
            None => 0,
        };
        let mut rng = rand::thread_rng();
        positions
            .choose_multiple_weighted(&mut rng, n, weights)
            .unwrap()
            .cloned()
            .collect()
    }
}
