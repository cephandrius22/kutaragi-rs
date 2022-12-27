#![deny(clippy::all)]
#![forbid(unsafe_code)]

use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use std::time::{Duration, SystemTime};

use glam::{Mat4, Vec2, Vec3, Vec4};

mod util;
use util::{perspective_divide, Triangle};

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;
const BOX_SIZE: i16 = 64;

/// Representation of the application state. In this example, a box will bounce around the screen.
struct World {
    box_x: i16,
    box_y: i16,
    velocity_x: i16,
    velocity_y: i16,
}

fn set_pixel(frame: &mut [u8], x: u32, y: u32, color: &mut [u8; 4]) {
    let index: usize = ((y * WIDTH * 4) + (x * 4)) as usize;
    frame[index..index + 4].copy_from_slice(color);
}

fn line(frame: &mut [u8], color: &mut [u8; 4], x0: u32, x1: u32, y0: u32, y1: u32) {
    let xdiff = (x0 as i32 - x1 as i32).abs();
    let ydiff = (y0 as i32 - y1 as i32).abs();
    let (mut xa, mut xb) = (x0, x1);
    let (mut ya, mut yb) = (y0, y1);

    let steep = if xdiff < ydiff {
        (xa, ya) = (ya, xa);
        (xb, yb) = (yb, xb);
        true
    } else {
        false
    };

    if xa > xb {
        (xa, xb) = (xb, xa);
        (ya, yb) = (yb, ya);
    }

    let dx: i32 = (xb as i32 - xa as i32) as i32;
    let dy = (yb as i32 - ya as i32) as i32;
    let derror = (dy as i32 * 2).abs();
    let mut error = 0;

    let mut y = ya;
    for x in xa..=xb {
        if steep {
            set_pixel(frame, y, x, color);
        } else {
            set_pixel(frame, x, y, color);
        }

        error += derror;
        if error > dx {
            y = if yb > ya { y + 1 } else { y - 1 };
            error -= dx * 2;
        }
    }
}

fn convert_to_ndc(v: &mut Vec2, width: f32, height: f32) {
    v.x = (v.x + (width / 2.0)) / width;
    v.y = (v.y + (height / 2.0)) / height;
}

fn convert_to_pixel(v: &mut Vec2, width: f32, height: f32) {
    v.x = v.x * width;
    v.y = (1.0 - v.y) * height;
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Hello Pixels")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };
    let mut world = World::new();

    let mut triangles = Vec::new();
    triangles.push(Triangle::new(
        Vec3::new(-50.0, 0.0, 0.0),
        Vec3::new(50.0, 0.0, 0.0),
        Vec3::new(0.0, 50.0, 0.0),
    ));

    let translation: Mat4 = Mat4::from_translation(Vec3::new(0.0, 0.0, -1.0));
    let projection: Mat4 = Mat4::perspective_rh_gl(f32::to_radians(90.0), 16.0 / 9.0, 0.1, 100.0);

    let orig = SystemTime::now();

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            if let Err(err) = pixels.render() {
                // world.draw(pixels.get_frame_mut());
                error!("pixels.render() failed: {err}");
                *control_flow = ControlFlow::Exit;
                return;
            }
            // TODO: not positive about this clearing of the
            // frame buffer.
            for elem in pixels.get_frame_mut() { *elem = 0; }
        }

        let diff = orig.elapsed().unwrap();
        let rotation = Mat4::from_rotation_x(diff.as_secs_f32());

        let mut color: [u8; 4] = [0x5e, 0x48, 0xe8, 0xff];
        let mvp: Mat4 = projection * translation * rotation;
        // let mvp: Mat4 = projection * rotation * translation;

        for tri in &triangles {
            let v1 = mvp * Vec4::new(tri.v1.x, tri.v1.y, tri.v1.z, 1.0);
            let v2 = mvp * Vec4::new(tri.v2.x, tri.v2.y, tri.v2.z, 1.0);
            let v3 = mvp * Vec4::new(tri.v3.x, tri.v3.y, tri.v3.z, 1.0);

            let mut s1 = perspective_divide(v1);
            let mut s2 = perspective_divide(v2);
            let mut s3 = perspective_divide(v3);

            // TODO: probably should just have these functions
            // return a vec2 and now use references.
            convert_to_ndc(&mut s1, WIDTH as f32, HEIGHT as f32);
            convert_to_ndc(&mut s2, WIDTH as f32, HEIGHT as f32);
            convert_to_ndc(&mut s3, WIDTH as f32, HEIGHT as f32);
            convert_to_pixel(&mut s1, WIDTH as f32, HEIGHT as f32);
            convert_to_pixel(&mut s2, WIDTH as f32, HEIGHT as f32);
            convert_to_pixel(&mut s3, WIDTH as f32, HEIGHT as f32);

            line(
                pixels.get_frame_mut(),
                &mut color,
                s1.x as u32,
                s2.x as u32,
                s1.y as u32,
                s2.y as u32,
            );

            line(
                pixels.get_frame_mut(),
                &mut color,
                s1.x as u32,
                s3.x as u32,
                s1.y as u32,
                s3.y as u32,
            );

            line(
                pixels.get_frame_mut(),
                &mut color,
                s2.x as u32,
                s3.x as u32,
                s2.y as u32,
                s3.y as u32,
            );
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    error!("pixels.resize_surface() failed: {err}");
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }

            // Update internal state and request a redraw
            world.update();
            window.request_redraw();
        }
    });
}

impl World {
    /// Create a new `World` instance that can draw a moving box.
    fn new() -> Self {
        Self {
            box_x: 24,
            box_y: 16,
            velocity_x: 1,
            velocity_y: 1,
        }
    }

    /// Update the `World` internal state; bounce the box around the screen.
    fn update(&mut self) {
        if self.box_x <= 0 || self.box_x + BOX_SIZE > WIDTH as i16 {
            self.velocity_x *= -1;
        }
        if self.box_y <= 0 || self.box_y + BOX_SIZE > HEIGHT as i16 {
            self.velocity_y *= -1;
        }

        self.box_x += self.velocity_x;
        self.box_y += self.velocity_y;
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % WIDTH as usize) as i16;
            let y = (i / WIDTH as usize) as i16;

            let inside_the_box = x >= self.box_x
                && x < self.box_x + BOX_SIZE
                && y >= self.box_y
                && y < self.box_y + BOX_SIZE;

            let rgba = if inside_the_box {
                [0x5e, 0x48, 0xe8, 0xff]
            } else {
                [0x48, 0xb2, 0xe8, 0xff]
            };

            pixel.copy_from_slice(&rgba);
        }
    }
}
