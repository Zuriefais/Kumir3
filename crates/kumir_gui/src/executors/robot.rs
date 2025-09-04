use crate::executors::Executor;
use egui::{Pos2, Vec2 as eguiVec2};
use kumir_runtime::FuncResult;
use log::info;
use vello::Scene;
use vello::kurbo::{Affine, Line, Point, Rect, Stroke, Vec2 as velloVec2};
use vello::peniko::{Color, color::palette::css};

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum RowsMode {
    FromUp,
    FromDown,
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ColumnsMode {
    FromLeft,
    FromRight,
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct RobotEditingState {
    pub deleting_rows_mode: RowsMode,
    pub deleting_columns_mode: ColumnsMode,
}

#[derive(Debug)]
pub enum Hovered {
    Cell {
        min: Point,
        max: Point,
        x: usize,
        y: usize,
    },
    VerticalBorder {
        min: Point,
        max: Point,
        x: usize,
        y: usize,
    },
    HorizontalBorder {
        min: Point,
        max: Point,
        x: usize,
        y: usize,
    },
    Robot {
        delta: velloVec2,
        dragging: bool,
    },
    None,
}

#[derive(Debug)]
struct FieldParameters {
    fill_color: Color,
    grid_color: Color,
    stroke_active: Stroke,
    storke_inactive: Stroke,
    cell_color: Color,
    robot_color: Color,
    robot_border_color: Color,
    hovered_color: Color,
}

#[derive(Debug)]
pub struct Robot {
    width: usize,
    height: usize,
    cell_size: f64,
    o: f64, // offset_x
    i: f64, // offset_y
    center_x: f64,
    center_y: f64,
    vertical_borders: Vec<Vec<bool>>,
    horizontal_borders: Vec<Vec<bool>>,
    colored: Vec<Vec<bool>>,
    x: usize,
    y: usize,
    scale: f64,
    hovered: Hovered,
    field_parameters: FieldParameters,

    pub editing_state: RobotEditingState,
}

impl Robot {
    pub fn new(width: usize, height: usize, cell_size: f64, center_x: f64, center_y: f64) -> Self {
        // #[cfg(unix)]
        // tracy_full::zone!("Robot Initialization", tracy_full::color::Color::CYAN, true);
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
            // fill_color: Color::from_rgb8(39, 143, 40),
            x: 0,
            y: 0,
            o: 0f64,
            i: 0f64,
            center_x: center_x,
            center_y: center_y,
            scale: 1.0,
            hovered: Hovered::None,
            editing_state: RobotEditingState {
                deleting_rows_mode: RowsMode::FromDown,
                deleting_columns_mode: ColumnsMode::FromRight,
            },
            field_parameters: FieldParameters {
                fill_color: css::DARK_GREEN,
                grid_color: Color::from_rgb8(200, 200, 16),
                stroke_active: Stroke::new(6.0),
                storke_inactive: Stroke::new(2.0),
                cell_color: Color::from_rgb8(147, 112, 219),
                robot_color: Color::from_rgb8(255, 255, 255),
                robot_border_color: Color::from_rgb8(0, 0, 0),
                hovered_color: css::BLUE_VIOLET,
            },
        }
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
                        self.field_parameters.cell_color,
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
        // #[cfg(unix)]
        // tracy_full::zone!("Vello Draw Grid", tracy_full::color::Color::CYAN, true);
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
                            &self.field_parameters.stroke_active,
                            Affine::IDENTITY,
                            self.field_parameters.grid_color,
                            None,
                            &horizontal_line,
                        ),
                        false => scene.stroke(
                            &self.field_parameters.storke_inactive,
                            Affine::IDENTITY,
                            self.field_parameters.grid_color,
                            None,
                            &horizontal_line,
                        ),
                    }
                }

                let vertical_line = Line::new(p1, Point::new(p1.x, p1.y + self.cell_size));
                if y < self.height {
                    match self.vertical_borders[x][y] {
                        true => scene.stroke(
                            &self.field_parameters.stroke_active,
                            Affine::IDENTITY,
                            self.field_parameters.grid_color,
                            None,
                            &vertical_line,
                        ),
                        false => scene.stroke(
                            &self.field_parameters.storke_inactive,
                            Affine::IDENTITY,
                            self.field_parameters.grid_color,
                            None,
                            &vertical_line,
                        ),
                    }
                }
            }
        }
    }

    pub fn draw_robot(&self, scene: &mut Scene) {
        // #[cfg(unix)]
        // tracy_full::zone!("Vello Draw Robot", tracy_full::color::Color::CYAN, true);
        let mut center_x = self.cell_size / 2.0 + self.cell_size * (self.x as f64) + self.o;
        let mut center_y = self.cell_size / 2.0 + self.cell_size * (self.y as f64) + self.i;

        match self.hovered {
            Hovered::Robot { delta, dragging: _ } => {
                center_x += delta.x / self.get_scale();
                center_y += delta.y / self.get_scale();
            }
            _ => (),
        }

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
            self.field_parameters.robot_color,
            None,
            &robot_shape,
        );

        scene.stroke(
            &Stroke::new(2.0),
            transform,
            self.field_parameters.robot_border_color,
            None,
            &robot_shape,
        );
    }

    pub fn draw_hovered(&self, scene: &mut Scene) {
        match self.hovered {
            Hovered::None => (),
            Hovered::Cell {
                min,
                max,
                x: _,
                y: _,
            }
            | Hovered::HorizontalBorder {
                min,
                max,
                x: _,
                y: _,
            }
            | Hovered::VerticalBorder {
                min,
                max,
                x: _,
                y: _,
            } => scene.fill(
                vello::peniko::Fill::NonZero,
                Affine::IDENTITY,
                self.field_parameters.hovered_color,
                None,
                &Rect::from_points(min, max),
            ),
            _ => (),
        }
    }

    pub fn add_row_from_up(&mut self) {
        // #[cfg(unix)]
        // tracy_full::zone!("Add Row From Up", tracy_full::color::Color::CYAN, true);
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
        // #[cfg(unix)]
        // tracy_full::zone!("Remove Row From Up", tracy_full::color::Color::CYAN, true);
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
        // #[cfg(unix)]
        // tracy_full::zone!("Add Row From Down", tracy_full::color::Color::CYAN, true);
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
        // #[cfg(unix)]
        // tracy_full::zone!("Remove Row From Down");
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
        // #[cfg(unix)]
        // tracy_full::zone!("Add Column From Left", tracy_full::color::Color::CYAN, true);
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
        // #[cfg(unix)]
        // tracy_full::zone!(
        //     "Remove Column From Left",
        //     tracy_full::color::Color::CYAN,
        //     true
        // );
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
        // #[cfg(unix)]
        // tracy_full::zone!(
        //     "Add Column From Right",
        //     tracy_full::color::Color::CYAN,
        //     true
        // );
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
        // #[cfg(unix)]
        // tracy_full::zone!(
        //     "Remove Column From Right",
        //     tracy_full::color::Color::CYAN,
        //     true
        // );
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
        // #[cfg(unix)]
        // tracy_full::zone!("Change Height", tracy_full::color::Color::CYAN, true);
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
        // #[cfg(unix)]
        // tracy_full::zone!("Change Width", tracy_full::color::Color::CYAN, true);
        let new_width = self.width as i64 + delta_width;
        if new_width >= 1 && (self.x as i64 - 1 >= 0 || !from_left) {
            self.width = new_width as usize;
            true
        } else {
            false
        }
    }

    // Robot API

    fn move_robot(&mut self, x: i64, y: i64) -> bool {
        // #[cfg(unix)]
        // tracy_full::zone!("Move Robot", tracy_full::color::Color::CYAN, true);
        let new_x = self.x as i64 + x;
        let new_y = self.y as i64 + y;
        if new_x >= self.width as i64 || new_x < 0 {
            return false;
        } else {
            self.x = new_x as usize;
        }
        if new_y >= self.height as i64 || new_y < 0 {
            return false;
        } else {
            self.y = new_y as usize;
        }

        true
    }

    pub fn move_right(&mut self) -> FuncResult<()> {
        if self.free_right().unwrap().unwrap() && self.move_robot(1, 0) {
            Ok(None)
        } else {
            Err("Robot is destroyed: there is a wall on the right".to_string())
        }
    }

    pub fn move_left(&mut self) -> FuncResult<()> {
        if self.free_left().unwrap().unwrap() && self.move_robot(-1, 0) {
            Ok(None)
        } else {
            Err("Robot is destroyed: there is a wall on the left".to_string())
        }
    }

    pub fn move_up(&mut self) -> FuncResult<()> {
        if self.free_above().unwrap().unwrap() && self.move_robot(0, -1) {
            Ok(None)
        } else {
            Err("Robot is destroyed: there is a wall above it".to_string())
        }
    }

    pub fn move_down(&mut self) -> FuncResult<()> {
        if self.free_below().unwrap().unwrap() && self.move_robot(0, 1) {
            Ok(None)
        } else {
            Err("Robot is destroyed: there is a wall below it".to_string())
        }
    }

    pub fn paint(&mut self) -> FuncResult<()> {
        self.colored[self.x][self.y] = true;
        Ok(None)
    }

    pub fn free_right(&self) -> FuncResult<bool> {
        Ok(Some(!self.vertical_borders[self.x + 1][self.y]))
    }

    pub fn free_left(&self) -> FuncResult<bool> {
        Ok(Some(!self.vertical_borders[self.x][self.y]))
    }

    pub fn free_above(&self) -> FuncResult<bool> {
        Ok(Some(!self.horizontal_borders[self.x][self.y]))
    }

    pub fn free_below(&self) -> FuncResult<bool> {
        Ok(Some(!self.horizontal_borders[self.x][self.y + 1]))
    }

    pub fn wall_right(&self) -> FuncResult<bool> {
        Ok(Some(self.vertical_borders[self.x + 1][self.y]))
    }

    pub fn wall_left(&self) -> FuncResult<bool> {
        Ok(Some(self.vertical_borders[self.x][self.y]))
    }

    pub fn wall_above(&self) -> FuncResult<bool> {
        Ok(Some(self.horizontal_borders[self.x][self.y]))
    }

    pub fn wall_below(&self) -> FuncResult<bool> {
        Ok(Some(self.horizontal_borders[self.x][self.y + 1]))
    }

    pub fn colored(&self) -> FuncResult<bool> {
        Ok(Some(self.colored[self.x][self.y]))
    }

    pub fn not_colored(&self) -> FuncResult<bool> {
        Ok(Some(!self.colored[self.x][self.y]))
    }
}

impl Executor for Robot {
    fn clear_field(&self, scene: &mut Scene) {
        // #[cfg(unix)]
        // tracy_full::zone!("Vello Clear Field", tracy_full::color::Color::CYAN, true);
        scene.fill(
            vello::peniko::Fill::NonZero,
            Affine::IDENTITY,
            self.field_parameters.fill_color,
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

    fn draw_field(&mut self, scene: &mut Scene) {
        // #[cfg(unix)]
        // tracy_full::zone!("Vello Draw Field", tracy_full::color::Color::CYAN, true);
        let mut new_scene = Scene::new();
        self.clear_field(&mut new_scene);
        self.fill_cells(&mut new_scene);
        self.draw_grid(&mut new_scene);
        self.draw_hovered(&mut new_scene);
        self.draw_robot(&mut new_scene);

        // info!("center_x: {} center_y: {}", center_x, center_y);
        let transform = Affine::translate((self.center_x, self.center_y))
            * Affine::scale(self.scale)
            * Affine::translate((-self.center_x, -self.center_y));

        scene.append(&new_scene, Some(transform));
    }

    fn base_color(&self) -> Color {
        self.field_parameters.fill_color
    }

    fn change_scale(&mut self, delta_scale: f64) {
        // #[cfg(unix)]
        // tracy_full::zone!("Change Scale", tracy_full::color::Color::CYAN, true);
        if 0.3 < self.scale + delta_scale && self.scale + delta_scale < 3.0 {
            self.scale += delta_scale;
        }
    }

    fn get_scale(&self) -> f64 {
        self.scale
    }

    fn hovered(&mut self, pos: Pos2, pixels_per_point: f32) {
        let border_in_cell = 1.0 / 15.0;

        let x = pos.x as f64 * pixels_per_point as f64;
        let y = pos.y as f64 * pixels_per_point as f64;

        let width = self.cell_size * self.get_width() as f64 * self.scale;
        let height = self.cell_size * self.get_height() as f64 * self.scale;
        let cell_size = self.cell_size * self.scale;

        let offset_x = self.center_x - width / 2.0;
        let offset_y = self.center_y - height / 2.0;
        let x_pos = ((x - offset_x) / cell_size).floor();
        let y_pos = ((y - offset_y) / cell_size).floor();
        let x_full = offset_x + x_pos * cell_size;
        let y_full = offset_y + y_pos * cell_size;
        let pos_min = Point::new(
            x_full + border_in_cell * cell_size,
            y_full + border_in_cell * cell_size,
        );
        let pos_max = Point::new(
            x_full + (1.0 - border_in_cell) * cell_size,
            y_full + (1.0 - border_in_cell) * cell_size,
        );

        // dp - drawn position
        let dp_x_full = self.o + x_pos * self.cell_size;
        let dp_y_full = self.i + y_pos * self.cell_size;
        let dp_min = Point::new(
            dp_x_full + self.cell_size * border_in_cell,
            dp_y_full + self.cell_size * border_in_cell,
        );
        let dp_max = Point::new(
            dp_x_full + self.cell_size * (1.0 - border_in_cell),
            dp_y_full + self.cell_size * (1.0 - border_in_cell),
        );

        // robot hover checking
        let robot_min = Point::new(
            offset_x + self.x as f64 * cell_size + cell_size / 3.0,
            offset_y + self.y as f64 * cell_size + cell_size / 3.0,
        );
        let robot_max = Point::new(
            offset_x + (self.x + 1) as f64 * cell_size - cell_size / 3.0,
            offset_y + (self.y + 1) as f64 * cell_size - cell_size / 3.0,
        );

        let dragging_robot = match self.hovered {
            Hovered::Robot { delta: _, dragging } => dragging,
            _ => false,
        };

        if 0f64 <= x_pos
            && x_pos < self.get_width() as f64
            && 0f64 <= y_pos
            && y_pos < self.get_height() as f64
        {
            if !dragging_robot {
                self.hovered = {
                    let border_in_cell = self.cell_size * border_in_cell;
                    if robot_min.x < x && x < robot_max.x && robot_min.y < y && y < robot_max.y {
                        Hovered::Robot {
                            delta: velloVec2::new(0.0, 0.0),
                            dragging: false,
                        }
                    } else if pos_min.x < x && x < pos_max.x && pos_min.y < y && y < pos_max.y {
                        Hovered::Cell {
                            min: dp_min,
                            max: dp_max,
                            x: x_pos as usize,
                            y: y_pos as usize,
                        }
                    } else if x < pos_min.x {
                        Hovered::VerticalBorder {
                            min: Point::new(dp_x_full - border_in_cell, dp_y_full),
                            max: Point::new(dp_x_full + border_in_cell, dp_y_full + self.cell_size),
                            x: x_pos as usize,
                            y: y_pos as usize,
                        }
                    } else if y < pos_min.y {
                        Hovered::HorizontalBorder {
                            min: Point::new(dp_x_full, dp_y_full - border_in_cell),
                            max: Point::new(dp_x_full + self.cell_size, dp_y_full + border_in_cell),
                            x: x_pos as usize,
                            y: y_pos as usize,
                        }
                    } else if x > pos_max.x {
                        Hovered::VerticalBorder {
                            min: Point::new(dp_x_full + self.cell_size - border_in_cell, dp_y_full),
                            max: Point::new(
                                dp_x_full + self.cell_size + border_in_cell,
                                dp_y_full + self.cell_size,
                            ),
                            x: x_pos as usize + 1,
                            y: y_pos as usize,
                        }
                    } else if y > pos_max.y {
                        Hovered::HorizontalBorder {
                            min: Point::new(dp_x_full, dp_y_full + self.cell_size - border_in_cell),
                            max: Point::new(
                                dp_x_full + self.cell_size,
                                dp_y_full + self.cell_size + border_in_cell,
                            ),
                            x: x_pos as usize,
                            y: y_pos as usize + 1,
                        }
                    } else {
                        Hovered::None
                    }
                }
            }
        } else if !dragging_robot {
            self.hovered = Hovered::None;
        }

        info!("Hovered: {:?}", self.hovered)
    }

    fn clicked(&mut self) {
        info!("Clicked");
        match self.hovered {
            Hovered::Cell {
                min: _,
                max: _,
                x,
                y,
            } => self.colored[x][y] = !self.colored[x][y],
            Hovered::HorizontalBorder {
                min: _,
                max: _,
                x,
                y,
            } => self.horizontal_borders[x][y] = !self.horizontal_borders[x][y],
            Hovered::VerticalBorder {
                min: _,
                max: _,
                x,
                y,
            } => self.vertical_borders[x][y] = !self.vertical_borders[x][y],
            _ => (),
        }
    }

    fn drag_started(&mut self) {
        match self.hovered {
            Hovered::Robot { delta, dragging: _ } => {
                self.hovered = Hovered::Robot {
                    delta: delta,
                    dragging: true,
                };
            }
            _ => (),
        }
    }

    fn drag(&mut self, drag_delta: eguiVec2) {
        match self.hovered {
            Hovered::Robot { delta, dragging: _ } => {
                self.hovered = Hovered::Robot {
                    delta: velloVec2::new(
                        delta.x + drag_delta.x as f64,
                        delta.y + drag_delta.y as f64,
                    ),
                    dragging: true,
                };
            }
            _ => (),
        }
    }

    fn drag_stop(&mut self) {
        match self.hovered {
            Hovered::Robot { delta, dragging: _ } => {
                let cells_x = (delta.x / (self.cell_size * self.get_scale())).round();
                let cells_y = (delta.y / (self.cell_size * self.get_scale())).round();

                if (self.x as f64) < -cells_x {
                    self.x = 0;
                } else if self.x as f64 + cells_x >= self.get_width() as f64 {
                    self.x = self.get_width() - 1;
                } else {
                    if cells_x > 0f64 {
                        self.x += cells_x as usize;
                    } else {
                        self.x -= -cells_x as usize;
                    }
                }

                if (self.y as f64) < -cells_y {
                    self.y = 0;
                } else if self.y as f64 + cells_y >= self.get_height() as f64 {
                    self.x = self.get_height() - 1;
                } else {
                    if cells_y > 0f64 {
                        self.y += cells_y as usize;
                    } else {
                        self.y -= -cells_y as usize;
                    }
                }

                self.hovered = Hovered::None;
            }
            _ => (),
        }
    }

    fn update_transform(&mut self, width: f64, height: f64) {
        self.center_x = width / 2.0;
        self.center_y = height / 2.0;
        self.o = self.center_x - self.cell_size * self.get_width() as f64 / 2.0;
        self.i = self.center_y - self.cell_size * self.get_height() as f64 / 2.0;
        println!(
            "Updates centers and offsets: x: {} y: {}",
            self.center_x, self.center_y
        );
    }
}
