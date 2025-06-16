use vello::Scene;
use vello::kurbo::{Affine, Circle, Ellipse, Line, Point, RoundedRect, Stroke};
use vello::peniko::Color;

pub fn add_shapes_to_scene(scene: &mut Scene, width: u32, height: u32) {
    draw_grid(scene, width as f64, height as f64, 25.0);
    // Draw an outlined rectangle
    let stroke = Stroke::new(6.0);
    let rect = RoundedRect::new(10.0, 10.0, 240.0, 240.0, 20.0);
    let rect_stroke_color = Color::new([0.9804, 0.702, 0.5294, 1.]);
    scene.stroke(&stroke, Affine::IDENTITY, rect_stroke_color, None, &rect);

    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    let circle = Circle::new((center_x, center_y), 120.0);
    let circle_fill_color = Color::new([0.9529, 0.5451, 0.6588, 1.]);
    scene.fill(
        vello::peniko::Fill::NonZero,
        Affine::IDENTITY,
        circle_fill_color,
        None,
        &circle,
    );

    // Draw a filled ellipse
    let ellipse = Ellipse::new((250.0, 420.0), (100.0, 160.0), -90.0);
    let ellipse_fill_color = Color::new([0.7961, 0.651, 0.9686, 1.]);
    scene.fill(
        vello::peniko::Fill::NonZero,
        Affine::IDENTITY,
        ellipse_fill_color,
        None,
        &ellipse,
    );

    // Draw a straight line
    let line = Line::new((260.0, 20.0), (620.0, 100.0));
    let line_stroke_color = Color::new([0.5373, 0.7059, 0.9804, 1.]);
    scene.stroke(&stroke, Affine::IDENTITY, line_stroke_color, None, &line);
}

fn draw_grid(scene: &mut Scene, width: f64, height: f64, cell_size: f64) {
    let stroke = Stroke::new(0.5); // Line thickness
    let color = Color::from_rgb8(255, 255, 255); // Black lines
    let transform = Affine::IDENTITY; // No transformation

    // Draw vertical lines
    for x in (0..=(width as i32)).step_by(cell_size as usize) {
        let line = Line::new(Point::new(x as f64, 0.0), Point::new(x as f64, height));
        scene.stroke(&stroke, transform, color, None, &line);
    }

    // Draw horizontal lines
    for y in (0..=(height as i32)).step_by(cell_size as usize) {
        let line = Line::new(Point::new(0.0, y as f64), Point::new(width, y as f64));
        scene.stroke(&stroke, transform, color, None, &line);
    }
}
