extern crate wee_alloc;

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
mod utils;

extern crate js_sys;

use wasm_bindgen::prelude::*;

extern crate web_sys;

use web_sys::console;

pub struct Timer<'a> {
    name: &'a str,
}

impl<'a> Timer<'a> {
    pub fn new(name: &'a str) -> Timer<'a> {
        console::time_with_label(name);
        Timer { name }
    }
}

impl<'a> Drop for Timer<'a> {
    fn drop(&mut self) {
        console::time_end_with_label(self.name);
    }
} // A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

extern crate fixedbitset;
use fixedbitset::FixedBitSet;

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: FixedBitSet,
}

impl Default for Universe {
    fn default() -> Self {
        utils::set_panic_hook();
        Self {
            width: 64,
            height: 64,
            cells: FixedBitSet::with_capacity(4096),
        }
    }
}

#[allow(clippy::missing_const_for_fn)]
#[wasm_bindgen]
impl Universe {
    #[must_use]
    pub fn new_empty() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn new() -> Self {
        utils::set_panic_hook();
        let width: u32 = 64;
        let height: u32 = 64;

        let size = (width * height) as usize;
        let mut cells = FixedBitSet::with_capacity(size);
        for i in 0..size {
            cells.set(i, js_sys::Math::random() < 0.5);
        }
        Self {
            width,
            height,
            cells,
        }
    }

    pub fn tick(&mut self) {
        let _timer = Timer::new("Universe::tick");
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);

                next.set(
                    idx,
                    match (cell, live_neighbors) {
                        (true, x) if x < 2 => false,
                        (true, 2 | 3) | (false, 3) => true,
                        (true, x) if x > 3 => false,
                        (otherwise, _) => otherwise,
                    },
                );
            }
        }

        self.cells = next;
    }
    #[must_use]
    pub fn width(&self) -> u32 {
        self.width
    }
    #[must_use]
    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn set_width(&mut self, width: u32) {
        self.width = width;
        self.reset();
    }

    pub fn set_height(&mut self, height: u32) {
        self.height = height;
        self.reset();
    }
    #[must_use]
    pub fn cells(&self) -> *const usize {
        self.cells.as_slice().as_ptr()
    }

    pub fn set_cell(&mut self, row: u32, col: u32) {
        let idx = self.get_index(row, col);
        self.cells.set(idx, true);
    }

    pub fn toggle_cell(&mut self, row: u32, col: u32) {
        let idx = self.get_index(row, col);
        self.cells.set(idx, !self.cells[idx]);
    }
    #[must_use]
    pub fn get_cell(&self, row: u32, col: u32) -> bool {
        let idx = self.get_index(row, col);
        self.cells[idx]
    }
}

impl Universe {
    const fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().copied() {
            for delta_col in [self.width - 1, 0, 1].iter().copied() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (column + delta_col) % self.width;
                let idx = self.get_index(neighbor_row, neighbor_col);
                count += u8::from(self.cells[idx]);
            }
        }
        count
    }

    fn reset(&mut self) {
        let size = (self.width * self.height) as usize;
        self.cells = FixedBitSet::with_capacity(size);
    }
}
