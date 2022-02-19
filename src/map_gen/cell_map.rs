use std::collections::HashMap;

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
