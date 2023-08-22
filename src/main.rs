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

fn convert_to_ndc(v: Vec2, width: u32, height: u32) -> Vec2 {
    let fwidth = width as f32;
    let fheight = height as f32;
    return Vec2::new(
        (v.x + (fwidth as f32 / 2.0)) / fwidth,
        (v.y + (fheight as f32/ 2.0)) / fheight,
    )
}

fn convert_to_pixel(v: Vec2, width: u32, height: u32) -> Vec2 {
    let fwidth = width as f32;
    let fheight = height as f32;
    return Vec2::new(
        v.x * fwidth,
        (1.0 - v.y) * fheight,
    );
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
                error!("pixels.render() failed: {err}");
                *control_flow = ControlFlow::Exit;
                return;
            }
            // TODO: not positive about this clearing of the
            // frame buffer.
            for elem in pixels.get_frame_mut() { *elem = 0; }
        }

        let diff = orig.elapsed().unwrap();
        // let rotation = Mat4::from_rotation_x(diff.as_secs_f32());

        let mut color: [u8; 4] = [0x5e, 0x48, 0xe8, 0xff];
        let mvp: Mat4 = projection * translation;// * rotation;
        // let mvp: Mat4 = projection * rotation * translation;

        for tri in &triangles {
            let v1 = mvp * Vec4::new(tri.v1.x, tri.v1.y, tri.v1.z, 1.0);
            let v2 = mvp * Vec4::new(tri.v2.x, tri.v2.y, tri.v2.z, 1.0);
            let v3 = mvp * Vec4::new(tri.v3.x, tri.v3.y, tri.v3.z, 1.0);

            let s1 = perspective_divide(v1);
            let s2 = perspective_divide(v2);
            let s3 = perspective_divide(v3);

            let s1_ndc = convert_to_ndc(s1, WIDTH, HEIGHT);
            let s2_ndc = convert_to_ndc(s2, WIDTH, HEIGHT);
            let s3_ndc = convert_to_ndc(s3, WIDTH, HEIGHT);
            let s1_pixel = convert_to_pixel(s1_ndc, WIDTH, HEIGHT);
            let s2_pixel = convert_to_pixel(s2_ndc, WIDTH, HEIGHT);
            let s3_pixel = convert_to_pixel(s3_ndc, WIDTH, HEIGHT);

            line(
                pixels.get_frame_mut(),
                &mut color,
                s1_pixel.x as u32,
                s2_pixel.x as u32,
                s1_pixel.y as u32,
                s2_pixel.y as u32,
            );

            line(
                pixels.get_frame_mut(),
                &mut color,
                s1_pixel.x as u32,
                s3_pixel.x as u32,
                s1_pixel.y as u32,
                s3_pixel.y as u32,
            );

            line(
                pixels.get_frame_mut(),
                &mut color,
                s2_pixel.x as u32,
                s3_pixel.x as u32,
                s2_pixel.y as u32,
                s3_pixel.y as u32,
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

            window.request_redraw();
        }
    });
}
