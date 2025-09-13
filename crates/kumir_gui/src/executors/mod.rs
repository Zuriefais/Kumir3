use egui::{Pos2, Vec2};
use vello::{
    Scene,
    kurbo::{Affine, Rect},
    peniko::Color,
};
pub mod robot;

use std::fmt::Debug;

pub trait Executor: Debug + Send {
    fn clear_field(&self, scene: &mut Scene);
    fn draw_field(&mut self, scene: &mut Scene);
    fn base_color(&self) -> Color;
    fn change_scale(&mut self, delta_scale: f64);
    fn get_scale(&self) -> f64;
    fn hovered(&mut self, pos: Pos2, pixels_per_point: f32);
    fn clicked(&mut self);
    fn drag_started(&mut self);
    fn update_transform(&mut self, width: f64, height: f64);
    fn drag(&mut self, delta: Vec2, pixels_per_point: f32);
    fn drag_stop(&mut self);
}

#[derive(Debug)]
pub struct NoneSelected;
impl NoneSelected {
    pub fn new() -> Self {
        Self {}
    }
}
impl Executor for NoneSelected {
    fn clear_field(&self, _: &mut Scene) {}
    fn base_color(&self) -> Color {
        Color::BLACK
    }
    fn change_scale(&mut self, _: f64) {}
    fn get_scale(&self) -> f64 {
        1.0
    }
    fn hovered(&mut self, _: Pos2, _: f32) {}
    fn clicked(&mut self) {}
    fn drag_started(&mut self) {}
    fn update_transform(&mut self, _: f64, _: f64) {}
    fn drag(&mut self, _: Vec2, _: f32) {}
    fn drag_stop(&mut self) {}

    fn draw_field(&mut self, scene: &mut Scene) {}
}
// pub fn add_shapes_to_scene(scene: &mut Scene, width: u32, height: u32) {
//     let rob = Robot::new(9, 9, 100.0);
//     rob.draw_field(scene);
// }

// pub fn add_shapes_to_scene(scene: &mut Scene, width: u32, height: u32) {
//     draw_grid(scene, width as f64, height as f64, 25.0);
//     // Draw an outlined rectangle
//     let stroke = Stroke::new(6.0);
//     let rect = RoundedRect::new(10.0, 10.0, 240.0, 240.0, 20.0);
//     let rect_stroke_color = Color::new([0.9804, 0.702, 0.5294, 1.]);
//     scene.stroke(&stroke, Affine::IDENTITY, rect_stroke_color, None, &rect);

//     let center_x = width as f32 / 2.0;
//     let center_y = height as f32 / 2.0;
//     let circle = Circle::new((center_x, center_y), 120.0);
//     let circle_fill_color = Color::new([0.9529, 0.5451, 0.6588, 1.]);
//     scene.fill(
//         vello::peniko::Fill::NonZero,
//         Affine::IDENTITY,
//         circle_fill_color,
//         None,
//         &circle,
//     );

//     // Draw a filled ellipse
//     let ellipse = Ellipse::new((250.0, 420.0), (100.0, 160.0), -90.0);
//     let ellipse_fill_color = Color::new([0.7961, 0.651, 0.9686, 1.]);
//     scene.fill(
//         vello::peniko::Fill::NonZero,
//         Affine::IDENTITY,
//         ellipse_fill_color,
//         None,
//         &ellipse,
//     );

//     // Draw a straight line
//     let line = Line::new((260.0, 20.0), (620.0, 100.0));
//     let line_stroke_color = Color::new([0.5373, 0.7059, 0.9804, 1.]);
//     scene.stroke(&stroke, Affine::IDENTITY, line_stroke_color, None, &line);
// }
