mod utils;

use js_sys::Math;
use std::fmt;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const CELL_SIZE: u32 = 15;
const GRID_COLOR: &str = "#CCCCCC";
const DEAD_COLOR: &str = "#FFFFFF";
const ALIVE_COLOR: &str = "#000000";

#[wasm_bindgen]
pub struct Universe {
    canvas_id: String,
    context: web_sys::CanvasRenderingContext2d,
    height: u32,
    width: u32,
    cells: Vec<Cell>,
}

#[wasm_bindgen]
impl Universe {
    pub fn new(canvas_id: String, width: u32, height: u32) -> Self {
        let cells = random_cells(width, height);

        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id(&canvas_id).unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();
        let canvas_height = (CELL_SIZE + 1) * width + 1;
        let canvas_width = (CELL_SIZE + 1) * height + 1;
        canvas.set_height(canvas_height);
        canvas.set_width(canvas_width);

        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        Self {
            canvas_id,
            context,
            height,
            width,
            cells,
        }
    }

    pub fn random_mutate(&mut self) {
        self.cells = random_cells(self.width, self.height)
    }

    pub fn draw_grid(&self) {
        let ctx = &self.context;
        ctx.begin_path();

        let grid_color = JsValue::from_str(GRID_COLOR);
        ctx.set_stroke_style(&grid_color);

        // Vertical lines.
        for i in 0..self.width {
            ctx.move_to((i * (CELL_SIZE + 1) + 1).into(), 0 as f64);
            ctx.line_to(
                (i * (CELL_SIZE + 1) + 1).into(),
                ((CELL_SIZE + 1) * self.height + 1).into(),
            );
        }

        // Horizontal lines.
        for j in 0..self.height {
            ctx.move_to(0 as f64, (j * (CELL_SIZE + 1) + 1).into());
            ctx.line_to(
                ((CELL_SIZE + 1) * self.width + 1).into(),
                (j * (CELL_SIZE + 1) + 1).into(),
            );
        }

        ctx.stroke();
    }

    pub fn draw_cells(&self) {
        let ctx = &self.context;

        ctx.begin_path();

        // NOTE: We draw twice because cost of fill_style.
        // Alive cells.
        let alive_color = JsValue::from_str(ALIVE_COLOR);
        ctx.set_fill_style(&alive_color);
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                if self.cells[idx] != Cell::Alive {
                    continue;
                }

                ctx.fill_rect(
                    (col * (CELL_SIZE + 1) + 1).into(),
                    (row * (CELL_SIZE + 1) + 1).into(),
                    CELL_SIZE.into(),
                    CELL_SIZE.into(),
                );
            }
        }

        // Dead cells.
        let dead_color = JsValue::from_str(DEAD_COLOR);
        ctx.set_fill_style(&dead_color);
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                if self.cells[idx] != Cell::Dead {
                    continue;
                }

                ctx.fill_rect(
                    (col * (CELL_SIZE + 1) + 1).into(),
                    (row * (CELL_SIZE + 1) + 1).into(),
                    CELL_SIZE.into(),
                    CELL_SIZE.into(),
                );
            }
        }
    }

    pub fn render(&self) -> String {
        self.to_string()
    }

    pub fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    pub fn tick(&mut self) {
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);

                let next_cell = match (cell, live_neighbors) {
                    // Rule 1: Any live cell with fewer than two live neighbours
                    // dies, as if caused by underpopulation.
                    (Cell::Alive, x) if x < 2 => Cell::Dead,
                    // Rule 2: Any live cell with two or three live neighbours
                    // lives on to the next generation.
                    (Cell::Alive, 2 | 3) => Cell::Alive,
                    // Rule 3: Any live cell with more than three live
                    // neighbours dies, as if by overpopulation.
                    (Cell::Alive, x) if x > 3 => Cell::Dead,
                    // Rule 4: Any dead cell with exactly three live neighbours
                    // becomes a live cell, as if by reproduction.
                    (Cell::Dead, 3) => Cell::Alive,
                    // All other cells remain in the same state.
                    (otherwise, _) => otherwise,
                };

                next[idx] = next_cell;
            }
        }
        self.cells = next
    }

    pub fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let neighbour_row = (row + delta_row) % self.height;
                let neighbour_col = (column + delta_col) % self.width;
                let idx = self.get_index(neighbour_row, neighbour_col);

                if let Cell::Alive = self.cells[idx] {
                    count += 1;
                }
            }
        }
        count
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    /// Set the width of the universe.
    ///
    /// Resets all cells to the dead state.
    pub fn set_width(&mut self, width: u32) {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id(&self.canvas_id).unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();
        let canvas_width = (CELL_SIZE + 1) * width + 1;
        canvas.set_width(canvas_width);
        self.width = width;
        self.cells = (0..width * self.height).map(|_| Cell::Dead).collect();
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// Set the height of the universe.
    ///
    /// Resets all cells to the dead state.
    pub fn set_height(&mut self, height: u32) {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id(&self.canvas_id).unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();
        let canvas_height = (CELL_SIZE + 1) * height + 1;
        canvas.set_height(canvas_height);
        self.height = height;
        self.cells = (0..height * self.width).map(|_| Cell::Dead).collect();
    }

    pub fn toggle_cell(&mut self, row: u32, column: u32) {
        let idx = self.get_index(row, column);
        self.cells[idx].toggle();
    }

    pub fn set_alive_cell(&mut self, row: u32, column: u32) {
        let idx = self.get_index(row, column);
        self.cells[idx].set_alive();
    }

    pub fn cells(&self) -> *const Cell {
        self.cells.as_ptr()
    }
}

impl Universe {
    // Get the dead and alive values of the entire universe.
    pub fn get_cells(&self) -> &[Cell] {
        &self.cells
    }

    /// Set cells to be alive in a universe by passing the row and column
    /// of each cell as an array.
    pub fn set_cells(&mut self, cells: &[(u32, u32)]) {
        for (row, col) in cells.iter().cloned() {
            let idx = self.get_index(row, col);
            self.cells[idx] = Cell::Alive;
        }
    }
}

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let symbol = if cell == Cell::Dead { '◻' } else { '◼' };
                write!(f, "{}", symbol)?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

impl Cell {
    fn toggle(&mut self) {
        *self = match *self {
            Cell::Alive => Cell::Dead,
            Cell::Dead => Cell::Alive,
        }
    }

    fn set_alive(&mut self) {
        *self = Cell::Alive
    }
}

fn random_cells(width: u32, height: u32) -> Vec<Cell> {
    (0..width * height)
        .map(|_| {
            // random bool
            if Math::random() < 0.5 {
                Cell::Alive
            } else {
                Cell::Dead
            }
        })
        .collect::<Vec<_>>()
}
