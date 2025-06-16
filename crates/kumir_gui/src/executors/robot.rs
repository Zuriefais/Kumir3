use vello::Scene;
use vello::kurbo::{Affine, Circle, Ellipse, Line, Point, Rect, RoundedRect, Stroke};
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
    vertical_borders: Vec<Vec<bool>>,
    horizontal_borders: Vec<Vec<bool>>,
    colored: Vec<Vec<bool>>,
    fill_color: Color,
    grid_color: Color,
    stroke_active: Stroke,
    storke_inactive: Stroke,
    cell_color: Color,
}

impl Robot {
    pub fn new(
        // scene: &mut Scene,
        width: usize,
        height: usize,
        cell_size: f64,
    ) -> Self {
        Self {
            width: width,
            height: height,
            cell_size: cell_size,
            vertical_borders: {
                let mut borders = vec![vec![false; height]; width + 1];

                for i in 0..height {
                    borders[0][i] = true;
                    borders[width][i] = true;
                }

                borders
            },
            horizontal_borders: {
                let mut borders = vec![vec![false; height]; width];

                for i in 0..width {
                    borders[i][0] = true;
                    borders[i][height - 1] = true;
                }

                borders
            },
            colored: vec![vec![false; height]; width],
            fill_color: Color::from_rgb8(40, 150, 40),
            grid_color: Color::from_rgb8(200, 200, 16),
            stroke_active: Stroke::new(6.0),
            storke_inactive: Stroke::new(2.0),
            cell_color: Color::from_rgb8(147, 112, 219),
        }
    }

    pub fn clear_field(&self, scene: &mut Scene) {
        scene.fill(
            vello::peniko::Fill::NonZero,
            Affine::IDENTITY,
            self.fill_color,
            None,
            &Rect::from_origin_size(
                (0.0, 0.0),
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
                            ((x as f64) * self.cell_size, (y as f64) * self.cell_size),
                            (self.cell_size, self.cell_size),
                        ),
                    )
                }
            }
        }
    }

    pub fn draw_grid(&self, scene: &mut Scene) {
        let stroke = Stroke::new(0.5); // Line thickness
        let color = Color::from_rgb8(255, 255, 255); // Black lines
        let transform = Affine::IDENTITY; // No transformation
        let width = (self.width as f64) * self.cell_size;
        let height = (self.height as f64) * self.cell_size;

        // Draw vertical lines
        for x in (0..=(width as i32)).step_by(self.cell_size as usize) {
            let line = Line::new(
                Point::new(x as f64, 0.0),
                Point::new(x as f64, height as f64),
            );
            scene.stroke(&stroke, transform, color, None, &line);
        }

        // Draw horizontal lines
        for y in (0..=(height as i32)).step_by(self.cell_size as usize) {
            let line = Line::new(
                Point::new(0.0, y as f64),
                Point::new(width as f64, y as f64),
            );
            scene.stroke(&stroke, transform, color, None, &line);
        }
    }

    pub fn draw_field(&self, scene: &mut Scene) {
        self.clear_field(scene);
        self.fill_cells(scene);
        self.draw_grid(scene);
    }
}
