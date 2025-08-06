use log::info;
use std::sync::{Arc, Mutex};
use vello::Scene;
use vello::kurbo::{Affine, Line, Point, Rect, Stroke};
use vello::peniko::Color;

#[derive(PartialEq, Copy, Clone)]
pub enum RowsMode {
    FromUp,
    FromDown,
}

#[derive(PartialEq, Copy, Clone)]
pub enum ColumnsMode {
    FromLeft,
    FromRight,
}

#[derive(PartialEq, Copy, Clone)]
pub struct RobotEditingState {
    pub deleting_rows_mode: RowsMode,
    pub deleting_columns_mode: ColumnsMode,
}

pub struct Cell {
    colored: bool,
    left: bool,
    right: bool,
    up: bool,
    down: bool,
}

pub struct Robot {
    width: usize,
    height: usize,
    cell_size: f64,
    o: f64, // offset_x
    i: f64, // offset_y
    vertical_borders: Vec<Vec<bool>>,
    horizontal_borders: Vec<Vec<bool>>,
    colored: Vec<Vec<bool>>,
    fill_color: Color,
    grid_color: Color,
    stroke_active: Stroke,
    storke_inactive: Stroke,
    cell_color: Color,
    robot_color: Color,
    robot_border_color: Color,
    x: usize,
    y: usize,
    scale: f64,
}

impl Robot {
    pub fn new(width: usize, height: usize, cell_size: f64) -> Self {
        #[cfg(unix)]
        tracy_full::zone!("Robot Initialization", tracy_full::color::Color::CYAN, true);
        Self {
            width,
            height,
            cell_size,
            vertical_borders: {
                let mut borders = vec![vec![false; height]; width + 1];
                for i in 0..height {
                    borders[0][i] = true;
                    borders[width][i] = true;
                }
                borders
            },
            horizontal_borders: {
                let mut borders = vec![vec![false; height + 1]; width];
                for i in 0..width {
                    borders[i][0] = true;
                    borders[i][height] = true;
                }
                borders
            },
            colored: vec![vec![false; height]; width],
            fill_color: Color::from_rgb8(39, 143, 40),
            grid_color: Color::from_rgb8(200, 200, 16),
            stroke_active: Stroke::new(6.0),
            storke_inactive: Stroke::new(2.0),
            cell_color: Color::from_rgb8(147, 112, 219),
            robot_color: Color::from_rgb8(255, 255, 255),
            robot_border_color: Color::from_rgb8(0, 0, 0),
            x: 0,
            y: 0,
            o: 100.0,
            i: 100.0,
            scale: 1.0,
        }
    }

    pub fn clear_field(&self, scene: &mut Scene) {
        #[cfg(unix)]
        tracy_full::zone!("Vello Clear Field", tracy_full::color::Color::CYAN, true);
        scene.fill(
            vello::peniko::Fill::NonZero,
            Affine::IDENTITY,
            self.fill_color,
            None,
            &Rect::from_origin_size(
                (self.o, self.i),
                (
                    (self.width as f64) * self.cell_size,
                    (self.height as f64) * self.cell_size,
                ),
            ),
        );
    }

    pub fn fill_cells(&self, scene: &mut Scene) {
        #[cfg(unix)]
        tracy_full::zone!("Vello Fill Cells");
        for x in 0..self.width {
            for y in 0..self.height {
                if self.colored[x][y] {
                    scene.fill(
                        vello::peniko::Fill::NonZero,
                        Affine::IDENTITY,
                        self.cell_color,
                        None,
                        &Rect::from_origin_size(
                            (
                                (x as f64) * self.cell_size + self.o,
                                (y as f64) * self.cell_size + self.i,
                            ),
                            (self.cell_size, self.cell_size),
                        ),
                    )
                }
            }
        }
    }

    pub fn draw_grid(&self, scene: &mut Scene) {
        #[cfg(unix)]
        tracy_full::zone!("Vello Draw Grid", tracy_full::color::Color::CYAN, true);
        for x in 0..=self.width {
            for y in 0..=self.height {
                let p1 = Point::new(
                    x as f64 * self.cell_size + self.o,
                    y as f64 * self.cell_size + self.i,
                );

                let horizontal_line = Line::new(p1, Point::new(p1.x + self.cell_size, p1.y));
                if x < self.width {
                    match self.horizontal_borders[x][y] {
                        true => scene.stroke(
                            &self.stroke_active,
                            Affine::IDENTITY,
                            self.grid_color,
                            None,
                            &horizontal_line,
                        ),
                        false => scene.stroke(
                            &self.storke_inactive,
                            Affine::IDENTITY,
                            self.grid_color,
                            None,
                            &horizontal_line,
                        ),
                    }
                }

                let vertical_line = Line::new(p1, Point::new(p1.x, p1.y + self.cell_size));
                if y < self.height {
                    match self.vertical_borders[x][y] {
                        true => scene.stroke(
                            &self.stroke_active,
                            Affine::IDENTITY,
                            self.grid_color,
                            None,
                            &vertical_line,
                        ),
                        false => scene.stroke(
                            &self.storke_inactive,
                            Affine::IDENTITY,
                            self.grid_color,
                            None,
                            &vertical_line,
                        ),
                    }
                }
            }
        }
    }

    pub fn draw_robot(&self, scene: &mut Scene) {
        #[cfg(unix)]
        tracy_full::zone!("Vello Draw Robot", tracy_full::color::Color::CYAN, true);
        let stroke = Stroke::new(0.5);
        let center_x = self.cell_size / 2.0 + self.cell_size * (self.x as f64) + self.o;
        let center_y = self.cell_size / 2.0 + self.cell_size * (self.y as f64) + self.i;

        let robot_shape = Rect::from_center_size(
            (center_x, center_y),
            (self.cell_size / 3.0, self.cell_size / 3.0),
        );

        let transform = Affine::translate((center_x, center_y))
            * Affine::rotate(45f64.to_radians())
            * Affine::translate((-center_x, -center_y));

        scene.fill(
            vello::peniko::Fill::NonZero,
            transform,
            self.robot_color,
            None,
            &robot_shape,
        );
    }

    pub fn draw_field(&self, scene: &mut Scene) {
        #[cfg(unix)]
        tracy_full::zone!("Vello Draw Field", tracy_full::color::Color::CYAN, true);
        let mut new_scene = Scene::new();
        self.clear_field(&mut new_scene);
        self.fill_cells(&mut new_scene);
        self.draw_grid(&mut new_scene);
        self.draw_robot(&mut new_scene);

        let center_x = self.width as f64 / 2.0 * self.cell_size + self.o;
        let center_y = self.height as f64 / 2.0 * self.cell_size + self.i;
        let transform = Affine::translate((center_x, center_y)) * Affine::scale(self.scale);

        scene.append(&new_scene, Some(transform));
    }

    pub fn change_offset_x(&mut self, o: f64) {
        #[cfg(unix)]
        tracy_full::zone!("Change Offset X", tracy_full::color::Color::CYAN, true);
        self.o += o;
    }

    pub fn change_offset_y(&mut self, i: f64) {
        #[cfg(unix)]
        tracy_full::zone!("Change Offset Y", tracy_full::color::Color::CYAN, true);
        self.i += i;
    }

    pub fn change_offset(&mut self, o: f64, i: f64) {
        #[cfg(unix)]
        tracy_full::zone!("Change Offset", tracy_full::color::Color::CYAN, true);
        self.change_offset_x(o);
        self.change_offset_y(i);
    }

    pub fn change_scale(&mut self, delta_scale: f64) {
        #[cfg(unix)]
        tracy_full::zone!("Change Scale", tracy_full::color::Color::CYAN, true);
        if 0.1 < self.scale + delta_scale && self.scale + delta_scale < 10.0 {
            self.scale += delta_scale;
        }
    }

    pub fn get_scale(&self) -> f64 {
        self.scale
    }

    pub fn add_row_from_up(&mut self) {
        #[cfg(unix)]
        tracy_full::zone!("Add Row From Up", tracy_full::color::Color::CYAN, true);
        for i in 0..=self.width {
            self.vertical_borders[i].insert(0, i == 0 || i == self.width);
        }
        for i in 0..self.width {
            self.horizontal_borders[i].insert(0, true);
            self.horizontal_borders[i][1] = false;
        }
        for i in 0..self.width {
            self.colored[i].insert(0, false);
        }
        self.height += 1;
        self.y += 1;
        self.i -= self.cell_size / 2.0;
    }

    pub fn remove_row_from_up(&mut self) {
        #[cfg(unix)]
        tracy_full::zone!("Remove Row From Up", tracy_full::color::Color::CYAN, true);
        if self.change_height(-1, true) {
            for i in 0..=self.width {
                self.vertical_borders[i].remove(0);
            }
            for i in 0..self.width {
                self.horizontal_borders[i].remove(0);
                self.horizontal_borders[i][0] = true;
            }
            for i in 0..self.width {
                self.colored[i].remove(0);
            }
            self.y -= 1;
            self.i += self.cell_size / 2.0;
        }
    }

    pub fn add_row_from_down(&mut self) {
        #[cfg(unix)]
        tracy_full::zone!("Add Row From Down", tracy_full::color::Color::CYAN, true);
        for i in 0..=self.width {
            self.vertical_borders[i].push(i == 0 || i == self.width);
        }
        for i in 0..self.width {
            self.horizontal_borders[i][self.height] = false;
            self.horizontal_borders[i].push(true);
        }
        for i in 0..self.width {
            self.colored[i].push(false);
        }
        self.height += 1;
        self.i -= self.cell_size / 2.0;
    }

    pub fn remove_row_from_down(&mut self) {
        #[cfg(unix)]
        tracy_full::zone!("Remove Row From Down");
        if self.change_height(-1, false) {
            for i in 0..=self.width {
                self.vertical_borders[i].pop();
            }
            for i in 0..self.width {
                self.horizontal_borders[i].pop();
                self.horizontal_borders[i][self.height] = true;
            }
            for i in 0..self.width {
                self.colored[i].pop();
            }
            self.i += self.cell_size / 2.0;
        }
    }

    pub fn add_column_from_left(&mut self) {
        #[cfg(unix)]
        tracy_full::zone!("Add Column From Left", tracy_full::color::Color::CYAN, true);
        let vertical_borders = vec![true; self.height];
        self.vertical_borders[0] = vec![false; self.height];
        self.vertical_borders.insert(0, vertical_borders);
        let mut horizontal_borders = vec![false; self.height + 1];
        horizontal_borders[0] = true;
        horizontal_borders[self.height] = true;
        self.horizontal_borders.insert(0, horizontal_borders);
        let colored = vec![false; self.height];
        self.colored.insert(0, colored);
        self.width += 1;
        self.x += 1;
        self.o -= self.cell_size / 2.0;
    }

    pub fn remove_column_from_left(&mut self) {
        #[cfg(unix)]
        tracy_full::zone!(
            "Remove Column From Left",
            tracy_full::color::Color::CYAN,
            true
        );
        if self.change_width(-1, true) {
            self.vertical_borders.remove(0);
            self.vertical_borders[0] = vec![true; self.height];
            self.horizontal_borders.remove(0);
            self.colored.remove(0);
            self.x -= 1;
            self.o += self.cell_size / 2.0;
        }
    }

    pub fn add_column_from_right(&mut self) {
        #[cfg(unix)]
        tracy_full::zone!(
            "Add Column From Right",
            tracy_full::color::Color::CYAN,
            true
        );
        let vertical_borders = vec![true; self.height];
        self.vertical_borders[self.width] = vec![false; self.height];
        self.vertical_borders.push(vertical_borders);
        let mut horizontal_borders = vec![false; self.height + 1];
        horizontal_borders[0] = true;
        horizontal_borders[self.height] = true;
        self.horizontal_borders.push(horizontal_borders);
        let colored = vec![false; self.height];
        self.colored.push(colored);
        self.width += 1;
        self.o -= self.cell_size / 2.0;
    }

    pub fn remove_column_from_right(&mut self) {
        #[cfg(unix)]
        tracy_full::zone!(
            "Remove Column From Right",
            tracy_full::color::Color::CYAN,
            true
        );
        if self.change_width(-1, false) {
            self.vertical_borders.pop();
            self.vertical_borders[self.width] = vec![true; self.height];
            self.horizontal_borders.pop();
            self.colored.pop();
            self.o += self.cell_size / 2.0;
        }
    }

    pub fn get_height(&self) -> usize {
        self.height
    }

    pub fn change_height(&mut self, delta_height: i64, from_up: bool) -> bool {
        #[cfg(unix)]
        tracy_full::zone!("Change Height", tracy_full::color::Color::CYAN, true);
        let new_height = self.height as i64 + delta_height;
        if new_height >= 1 && (self.y as i64 - 1 >= 0 || !from_up) {
            self.height = new_height as usize;
            true
        } else {
            false
        }
    }

    pub fn get_width(&self) -> usize {
        self.width
    }

    pub fn change_width(&mut self, delta_width: i64, from_left: bool) -> bool {
        #[cfg(unix)]
        tracy_full::zone!("Change Width", tracy_full::color::Color::CYAN, true);
        let new_width = self.width as i64 + delta_width;
        if new_width >= 1 && (self.x as i64 - 1 >= 0 || !from_left) {
            self.width = new_width as usize;
            true
        } else {
            false
        }
    }

    fn move_robot(&mut self, x: i64, y: i64) {
        #[cfg(unix)]
        tracy_full::zone!("Move Robot", tracy_full::color::Color::CYAN, true);
        let new_x = self.x as i64 + x;
        let new_y = self.y as i64 + y;
        if new_x >= self.width as i64 {
            self.x = self.width - 1;
        } else if new_x < 0 {
            self.x = 0;
        } else {
            self.x = new_x as usize;
        }
        if new_y >= self.height as i64 {
            self.y = self.height - 1;
        } else if new_y < 0 {
            self.y = 0;
        } else {
            self.y = new_y as usize;
        }
    }

    pub fn move_up(&mut self) {
        #[cfg(unix)]
        tracy_full::zone!("Move Up", tracy_full::color::Color::CYAN, true);
        self.move_robot(0, -1);
    }

    pub fn move_down(&mut self) {
        #[cfg(unix)]
        tracy_full::zone!("Move Down", tracy_full::color::Color::CYAN, true);
        self.move_robot(0, 1);
    }

    pub fn move_right(&mut self) {
        #[cfg(unix)]
        tracy_full::zone!("Move Right", tracy_full::color::Color::CYAN, true);
        self.move_robot(1, 0);
        info!("moved right");
    }

    pub fn move_left(&mut self) {
        #[cfg(unix)]
        tracy_full::zone!("Move Left", tracy_full::color::Color::CYAN, true);
        self.move_robot(-1, 0);
    }

    pub fn color(&mut self) {
        #[cfg(unix)]
        tracy_full::zone!("Color Cell", tracy_full::color::Color::CYAN, true);
        self.colored[self.x][self.y] = true;
    }

    pub fn is_colored(&self) -> bool {
        #[cfg(unix)]
        tracy_full::zone!("Check Colored", tracy_full::color::Color::CYAN, true);
        self.colored[self.x][self.y]
    }

    pub fn from_above(&self) -> bool {
        #[cfg(unix)]
        tracy_full::zone!("Check Above", tracy_full::color::Color::CYAN, true);
        self.horizontal_borders[self.x][self.y]
    }

    pub fn from_below(&self) -> bool {
        #[cfg(unix)]
        tracy_full::zone!("Check Below", tracy_full::color::Color::CYAN, true);
        self.horizontal_borders[self.x][self.y + 1]
    }

    pub fn from_left(&self) -> bool {
        #[cfg(unix)]
        tracy_full::zone!("Check Left", tracy_full::color::Color::CYAN, true);
        self.vertical_borders[self.x][self.y]
    }

    pub fn from_right(&self) -> bool {
        #[cfg(unix)]
        tracy_full::zone!("Check Right", tracy_full::color::Color::CYAN, true);
        self.vertical_borders[self.x + 1][self.y]
    }
}
