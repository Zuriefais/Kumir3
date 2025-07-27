use vello::Scene;
use vello::kurbo::{Affine, Line, Point, Rect, Stroke};
use vello::peniko::Color;

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
}

impl Robot {
    pub fn new(
        // scene: &mut Scene,
        width: usize,
        height: usize,
        cell_size: f64,
    ) -> Self {
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
        }
    }

    // Drawing methods

    pub fn clear_field(&self, scene: &mut Scene) {
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

    // pub fn draw_grid(&self, scene: &mut Scene) {
    //     let stroke = Stroke::new(0.5);
    //     let color = Color::from_rgb8(255, 255, 255);
    //     let transform = Affine::IDENTITY;
    //     let width = (self.width as f64) * self.cell_size + self.i;
    //     let height = (self.height as f64) * self.cell_size + self.o;

    //     // Draw vertical lines
    //     for x in (0..=(width as i32)).step_by(self.cell_size as usize) {
    //         let line = Line::new(
    //             Point::new(x as f64 + self.o, self.i),
    //             Point::new(x as f64 + self.o, height as f64 + self.i),
    //         );
    //         scene.stroke(&stroke, transform, color, None, &line);
    //     }

    //     // Draw horizontal lines
    //     for y in (0..=(height as i32)).step_by(self.cell_size as usize) {
    //         let line = Line::new(
    //             Point::new(self.o, y as f64 + self.i),
    //             Point::new(width as f64 + self.o, y as f64 + self.i),
    //         );
    //         scene.stroke(&stroke, transform, color, None, &line);
    //     }
    // }

    pub fn draw_grid(&self, scene: &mut Scene) {
        for x in 0..=self.width {
            for y in 0..=self.height {
                // Horizontal border
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
        let stroke = Stroke::new(0.5); // Robot border

        let center_x = self.cell_size / 2.0 + self.cell_size * (self.x as f64) + self.o;
        let center_y = self.cell_size / 2.0 + self.cell_size * (self.y as f64) + self.i;

        let robot_shape = Rect::from_center_size(
            (center_x, center_y),
            (self.cell_size / 3.0, self.cell_size / 3.0),
        );

        let rotation = Affine::translate((center_x, center_y))
            * Affine::rotate(45f64.to_radians())
            * Affine::translate((-center_x, -center_y));

        scene.fill(
            vello::peniko::Fill::NonZero,
            rotation,
            self.robot_color,
            None,
            &robot_shape,
        );
    }

    pub fn draw_field(&self, scene: &mut Scene) {
        self.clear_field(scene);
        self.fill_cells(scene);
        self.draw_grid(scene);
        self.draw_robot(scene);
    }

    // Robot API
    // Only god can save my ass from using this shit.

    fn move_robot(&mut self, x: i64, y: i64) {
        let new_x = self.x as i64 + x;
        let new_y = self.y as i64 + y;

        if new_x >= self.width as i64 {
            self.x = self.width - 1;
        } else if new_x < 0 {
            self.x = 0;
        }

        if new_y >= self.width as i64 {
            self.y = self.width - 1;
        } else if new_y < 0 {
            self.y = 0;
        }
    }

    pub fn move_up(&mut self) {
        self.move_robot(0, -1);
    }

    pub fn move_down(&mut self) {
        self.move_robot(0, 1);
    }

    pub fn move_right(&mut self) {
        self.move_robot(1, 0);
    }

    pub fn move_left(&mut self) {
        self.move_robot(-1, 0);
    }

    // Coloring cells

    pub fn color(&mut self) {
        self.colored[self.x][self.y] = true;
    }

    pub fn is_colored(&self) -> bool {
        self.colored[self.x][self.y]
    }

    // Getting borders...

    pub fn from_above(&self) -> bool {
        self.horizontal_borders[self.x][self.y]
    }

    pub fn from_below(&self) -> bool {
        self.horizontal_borders[self.x][self.y + 1]
    }

    pub fn from_left(&self) -> bool {
        self.vertical_borders[self.x][self.y]
    }

    pub fn from_right(&self) -> bool {
        self.vertical_borders[self.x + 1][self.y]
    }
}
