#![no_std]
extern crate wee_alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
mod utils;

extern crate js_sys;

use wasm_bindgen::prelude::*;

extern crate web_sys;

// use web_sys::console;

const WIDTH: u32 = 128;
const HEIGHT: u32 = 128;
const SIZE: u32 = WIDTH * HEIGHT;

// pub struct Timer<'a> {
//     name: &'a str,
// }

// impl<'a> Timer<'a> {
//     pub fn new(name: &'a str) -> Timer<'a> {
//         console::time_with_label(name);
//         Timer { name }
//     }
// }

// impl<'a> Drop for Timer<'a> {
//     fn drop(&mut self) {
//         console::time_end_with_label(self.name);
//     }
// } // A macro to provide `println!(..)`-style syntax for `console.log` logging.
// macro_rules! log {
//     ( $( $t:tt )* ) => {
//         web_sys::console::log_1(&format!( $( $t )* ).into());
//     }
// }

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: [u8; SIZE as usize / 8],
    cells_tmp: [u8; SIZE as usize / 8],
}

static mut UNIV: Universe = Universe {
    width: WIDTH,
    height: HEIGHT,
    cells: [0; SIZE as usize / 8],
    cells_tmp: [0; SIZE as usize / 8],
};

#[allow(clippy::missing_const_for_fn)]
#[wasm_bindgen]
impl Universe {
    #[must_use]
    pub unsafe fn new_empty() {
        Self::reset();
    }

    #[must_use]
    pub unsafe fn new() {
        // utils::set_panic_hook();
        let mut cells = [0; SIZE as usize / 8];
        for i in 0..SIZE {
            let idx = i / 8;
            let bit = i % 8;

            cells[idx as usize] = match js_sys::Math::random() < 0.5 {
                true => cells[idx as usize] | 2u8.pow(bit),
                false => cells[idx as usize] & (u8::MAX - 2u8.pow(bit)),
            }
        }
        UNIV.cells = cells.clone();
        UNIV.cells_tmp = cells;
    }

    pub unsafe fn tick() {
        // let _timer = Timer::new("Universe::tick");
        UNIV.cells_tmp = UNIV.cells.clone();
        // log!("{:?}", UNIV.cells);
        for row in 0..UNIV.height {
            for col in 0..UNIV.width {
                let cell = Self::get_cell(row, col);
                let live_neighbors = Self::live_neighbor_count(row, col);

                match (cell, live_neighbors) {
                    (true, x) if (x < 2) | (x > 3) => Self::unset_tmp_cell(row, col),
                    (false, 3) | (true, _) => Self::set_tmp_cell(row, col),
                    (false, _) => Self::unset_tmp_cell(row, col),
                };
            }
        }
        // log!("t{:?}", UNIV.cells_tmp);
        UNIV.cells = UNIV.cells_tmp.clone();
        // log!("a{:?}", UNIV.cells);
    }
    #[must_use]
    pub unsafe fn width() -> u32 {
        UNIV.width
    }
    #[must_use]
    pub unsafe fn height() -> u32 {
        UNIV.height
    }

    #[must_use]
    pub unsafe fn cells() -> *const u8 {
        UNIV.cells.as_slice().as_ptr()
    }

    pub unsafe fn set_cell(row: u32, col: u32) {
        let i = Self::get_index(row, col);
        let idx = i / 8;
        let bit = i % 8;

        UNIV.cells[idx as usize] = UNIV.cells[idx as usize] | 2u8.pow(bit as u32);
        UNIV.cells_tmp[idx as usize] = UNIV.cells[idx as usize] | 2u8.pow(bit as u32);
    }

    pub unsafe fn unset_cell(row: u32, col: u32) {
        let i = Self::get_index(row, col);
        let idx = i / 8;
        let bit = i % 8;

        UNIV.cells[idx as usize] = UNIV.cells[idx as usize] & (u8::MAX - 2u8.pow(bit as u32));
        UNIV.cells_tmp[idx as usize] = UNIV.cells[idx as usize] & (u8::MAX - 2u8.pow(bit as u32));
    }

    pub unsafe fn toggle_cell(row: u32, col: u32) {
        let i = Self::get_index(row, col);
        let idx = i / 8;
        let bit = i % 8;
        UNIV.cells[idx as usize] = match !((UNIV.cells[idx] & 2u8.pow(bit as u32)) > 0) {
            true => UNIV.cells[idx as usize] | 2u8.pow(bit as u32),
            false => UNIV.cells[idx as usize] & (u8::MAX - 2u8.pow(bit as u32)),
        };
        UNIV.cells_tmp = UNIV.cells.clone();
    }
    #[must_use]
    pub unsafe fn get_cell(row: u32, col: u32) -> bool {
        let i = Self::get_index(row, col);
        let idx = i / 8;
        let bit = i % 8;
        UNIV.cells[idx] & 2u8.pow(bit as u32) > 0
    }
}

impl Universe {
    const unsafe fn get_index(row: u32, column: u32) -> usize {
        (row * UNIV.width + column) as usize
    }

    unsafe fn set_tmp_cell(row: u32, col: u32) {
        let i = Self::get_index(row, col);
        let idx = i / 8;
        let bit = i % 8;

        // log!("{row},{col}=setting  {idx}*8+{bit}={i} ");
        UNIV.cells_tmp[idx as usize] = UNIV.cells_tmp[idx as usize] | 2u8.pow(bit as u32);
    }

    unsafe fn unset_tmp_cell(row: u32, col: u32) {
        let i = Self::get_index(row, col);
        let idx = i / 8;
        let bit = i % 8;

        // log!("{row},{col}=unsetting  {idx}*8+{bit}={i} ");
        UNIV.cells_tmp[idx as usize] =
            UNIV.cells_tmp[idx as usize] & (u8::MAX - 2u8.pow(bit as u32));
    }

    unsafe fn live_neighbor_count(row: u32, column: u32) -> u8 {
        let mut count = 0;

        let north = if row == 0 { UNIV.height - 1 } else { row - 1 };

        let south = if row == UNIV.height - 1 { 0 } else { row + 1 };

        let west = if column == 0 {
            UNIV.width - 1
        } else {
            column - 1
        };

        let east = if column == UNIV.width - 1 {
            0
        } else {
            column + 1
        };

        let nw = Self::get_cell(north, west);
        count += nw as u8;

        let n = Self::get_cell(north, column);
        count += n as u8;

        let ne = Self::get_cell(north, east);
        count += ne as u8;

        let w = Self::get_cell(row, west);
        count += w as u8;

        let e = Self::get_cell(row, east);
        count += e as u8;

        let sw = Self::get_cell(south, west);
        count += sw as u8;

        let s = Self::get_cell(south, column);
        count += s as u8;

        let se = Self::get_cell(south, east);
        count += se as u8;

        count
    }

    unsafe fn reset() {
        UNIV.cells = [0; SIZE as usize / 8];
        UNIV.cells_tmp = [0; SIZE as usize / 8];
    }
}
