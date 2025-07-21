use crate::egui_tools::EguiRenderer;
use crate::executors::add_shapes_to_scene;
use crate::gruvbox_egui::gruvbox_dark_theme;
use crate::gui::{KumirGui, VelloWindowOptions};
use egui_wgpu::wgpu::SurfaceError;
use egui_wgpu::{ScreenDescriptor, wgpu};
use log::info;
use std::sync::{Arc, Mutex};
use vello::peniko::color::palette;
use vello::wgpu::TextureFormat;
use vello::{AaConfig, Renderer, RendererOptions, Scene};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use wgpu::Texture;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::event_loop::EventLoop;
#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowExtWebSys;
use winit::window::{Window, WindowId};

pub struct AppState {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub surface: wgpu::Surface<'static>,
    pub scale_factor: f32,
    pub egui_renderer: EguiRenderer,
    pub kumir_gui: KumirGui,
    pub vello_renderer: Renderer,
    pub vello_scene: Scene,
    pub vello_texture: Texture,
    pub window: Arc<Window>,
    vello_window_options: Arc<Mutex<VelloWindowOptions>>,
}

fn create_vello_texture(device: &wgpu::Device, width: u32, height: u32) -> Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Vello Texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}

impl AppState {
    async fn new(window: Arc<Window>) -> Self {
        info!("Creating App State...");
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::BROWSER_WEBGPU,
            ..Default::default()
        });

        let surface = instance
            .create_surface(window.clone())
            .expect("Failed to create surface!");
        info!("Surface created");
        let initial_width = 1920;
        let initial_height = 1080;

        let _ = window.request_inner_size(PhysicalSize::new(initial_width, initial_height));
        let size = window.inner_size();
        info!("Window size: {:?}", size);
        let (width, height) = size.into();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let features = wgpu::Features::empty();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: features,
                    required_limits: Default::default(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let selected_format = wgpu::TextureFormat::Bgra8Unorm;
        let swapchain_format = swapchain_capabilities
            .formats
            .iter()
            .find(|d| **d == selected_format)
            .expect("failed to select proper surface texture format!");
        info!("Supported formats: {:?}", swapchain_capabilities.formats);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: *swapchain_format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 0,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &surface_config);

        let egui_renderer = EguiRenderer::new(&device, surface_config.format, None, 1, &window);
        egui_renderer.context().set_style(gruvbox_dark_theme());

        let scale_factor = 1.0;
        let vello_window_options: Arc<Mutex<VelloWindowOptions>> = Default::default();
        let kumir_gui = KumirGui::new(egui_renderer.context(), vello_window_options.clone());

        let vello_texture = create_vello_texture(&device, 100, 100);
        let vello_renderer = Renderer::new(
            &device,
            RendererOptions {
                ..Default::default()
            },
        )
        .expect("Couldn't create renderer");
        let vello_scene = Scene::new();
        info!("App State created!!");
        Self {
            device,
            queue,
            surface,
            surface_config,
            egui_renderer,
            scale_factor,
            kumir_gui,
            vello_renderer,
            vello_scene,
            window,
            vello_texture,
            vello_window_options,
        }
    }

    fn resize_surface(&mut self, width: u32, height: u32) {
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
    }

    fn handle_redraw(&mut self, window: &Window) {
        let width = self.surface_config.width;
        let height = self.surface_config.height;
        let mut vello_window_changed = false;
        if let Ok(mut vello_size) = self.vello_window_options.lock() {
            if vello_size.changed && vello_size.height != 0 && vello_size.height != 0 {
                vello_window_changed = true;
                vello_size.changed = false;
                info!(
                    "Vello texture size changed: {}, {}",
                    vello_size.width, vello_size.height
                );
                self.vello_texture =
                    create_vello_texture(&self.device, vello_size.width, vello_size.height);
            }
        }
        self.vello_scene.reset();

        // Re-add the objects to draw to the scene.
        add_shapes_to_scene(&mut self.vello_scene, width, height);

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [width, height],
            pixels_per_point: window.scale_factor() as f32 * self.scale_factor,
        };

        let surface_texture = self.surface.get_current_texture();

        match surface_texture {
            Err(SurfaceError::Outdated) => {
                // Ignoring outdated to allow resizing and minimization
                info!("wgpu surface outdated");
                return;
            }
            Err(_) => {
                surface_texture.expect("Failed to acquire next swap chain texture");
                return;
            }
            Ok(_) => {}
        };

        let surface_texture = surface_texture.unwrap();

        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let vello_view = self
            .vello_texture
            .create_view(&wgpu::TextureViewDescriptor {
                format: Some(TextureFormat::Rgba8Unorm),
                usage: Some(
                    wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
                ),
                ..Default::default()
            });
        if vello_window_changed {
            if let Ok(mut vello_options) = self.vello_window_options.lock() {
                vello_options.texture = self
                    .egui_renderer
                    .register_native_texture(&self.device, &vello_view);
            }
        }

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            self.vello_renderer
                .render_to_texture(
                    &self.device,
                    &self.queue,
                    &self.vello_scene,
                    &vello_view,
                    &vello::RenderParams {
                        base_color: palette::css::BLACK, // Background color
                        width,
                        height,
                        antialiasing_method: AaConfig::Msaa16,
                    },
                )
                .expect("failed to render to surface");

            self.egui_renderer.begin_frame(window);
            self.kumir_gui.render_gui();
            self.egui_renderer.end_frame_and_draw(
                &self.device,
                &self.queue,
                &mut encoder,
                window,
                &surface_view,
                screen_descriptor,
            );
        }

        self.queue.submit(Some(encoder.finish()));
        surface_texture.present();
    }
}

pub struct App {
    state: Option<AppState>,
    window: Option<Arc<Window>>,
    #[cfg(target_arch = "wasm32")]
    proxy: Option<winit::event_loop::EventLoopProxy<AppState>>,
}

impl App {
    pub fn new(#[cfg(target_arch = "wasm32")] event_loop: &EventLoop<AppState>) -> Self {
        #[cfg(target_arch = "wasm32")]
        let proxy = Some(event_loop.create_proxy());
        Self {
            state: None,
            window: None,
            #[cfg(target_arch = "wasm32")]
            proxy,
        }
    }

    #[allow(unused_mut)]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: AppState) {
        #[cfg(target_arch = "wasm32")]
        {
            event.window.request_redraw();
            event.resize_surface(
                event.window.inner_size().width,
                event.window.inner_size().height,
            );
        }
        self.state = Some(event);
    }

    async fn set_window(&mut self, window: Arc<Window>) {
        info!("Setting window options");
        let initial_width = 1920;
        let initial_height = 1080;

        let _ = window.request_inner_size(PhysicalSize::new(initial_width, initial_height));

        let state = AppState::new(window.clone()).await;

        self.window.get_or_insert(window);
        self.state.get_or_insert(state);
    }

    fn handle_resized(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.state.as_mut().unwrap().resize_surface(width, height);
        }
    }

    fn handle_redraw(&mut self) {
        // Attempt to handle minimizing window
        if let Some(window) = self.window.as_ref() {
            if let Some(min) = window.is_minimized() {
                if min {
                    info!("Window is minimized");
                    return;
                }
            }
            if let Some(state) = &mut self.state {
                state.handle_redraw(window);
            }
        }
    }
}

impl ApplicationHandler<AppState> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        info!("Window resumed");
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes();

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWebSys;

            const CANVAS_ID: &str = "canvas";

            let window = wgpu::web_sys::window().unwrap_throw();
            let document = window.document().unwrap_throw();
            let canvas = document.get_element_by_id(CANVAS_ID).unwrap_throw();
            let html_canvas_element = canvas.unchecked_into();
            window_attributes = window_attributes.with_canvas(Some(html_canvas_element));
        }

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        info!("Window connected to winit. Starting set window future");

        #[cfg(not(target_arch = "wasm32"))]
        {
            pollster::block_on(self.set_window(window));
        }
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(proxy) = self.proxy.take() {
                wasm_bindgen_futures::spawn_local(async move {
                    assert!(proxy.send_event(AppState::new(window).await).is_ok())
                });
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        // let egui render to process the event first
        if let Some(state) = self.state.as_mut() {
            if let Some(window) = self.window.as_ref() {
                state.egui_renderer.handle_input(window, &event);
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                info!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.handle_redraw();

                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            WindowEvent::Resized(new_size) => {
                self.handle_resized(new_size.width, new_size.height);
            }
            _ => (),
        }
    }
}
