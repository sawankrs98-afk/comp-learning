use softbuffer::{Context, Surface};
use std::{num::NonZeroU32, option};
use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::{self, ActiveEventLoop, EventLoop}, window::{self, Window, WindowId},
};

use winit::event::ElementState;
use crate::pointer::{self, Pointer, PointerButton};

#[derive(Default)]
struct App {
    // stores the state of the window...
    // we dont have any window at the start
    // so we have to store it as an option as option --> somehting, NUll..
    // both the context and the surface needs to keep the window alive
    // so we use Arch because it keeps the window alive
    // Arch lets us share the window object without actually transferring the window object...
    window: Option<std::sync::Arc<Window>>,

    // stores softbuffering connection to the windowing system...
    context: Option<Context<std::sync::Arc<Window>>>,

    // storing the drawable surface used to present pixels...
    surface: Option<Surface<std::sync::Arc<Window>, std::sync::Arc<Window>>>,

    // added the pointer impl
    pointer : Pointer,
}

// winit calls this when window event happens...
impl ApplicationHandler for App {
    // called when application is ready to create windows...
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // if there is no window we will create it
        if self.window.is_none() {
            // set the size and everything before making the window...
            let attributes = Window::default_attributes()
                .with_title("My first working window...")
                .with_inner_size(winit::dpi::LogicalSize::new(640.0, 400.0));

            // asking winit to create the actual window
            let window = std::sync::Arc::new(event_loop.create_window(attributes).unwrap());

            // init softBuffer using the windows display connection...
            let context = Context::new(window.clone()).unwrap();

            // create the drawable surface for this window...
            let surface = Surface::new(&context, window.clone()).unwrap();

            self.window = Some(window.clone());
            self.context = Some(context);
            self.surface = Some(surface);

            // saving the created window so it stays alive...
            window.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        // matching the event
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                // getting the window and the drawing surface...
                let window = self.window.as_ref().unwrap();
                let surface = self.surface.as_mut().unwrap();

                // calculating the size of the window...
                let size = window.inner_size();

                if size.width == 0 || size.height == 0 {
                    return;
                }

                // now as we took the reference for our surface as mut we will draw upon it...
                surface
                    .resize(
                        // we can use unwrap here becuase we know
                        // that the size will not be 0 as already checked...
                        NonZeroU32::new(size.width).unwrap(),
                        NonZeroU32::new(size.height).unwrap(),
                    )
                    .unwrap();

                // now as the surface has been made
                // we take a mutable buffer to draw something on...
                let mut buffer = surface.buffer_mut().unwrap();

                for pixel in buffer.iter_mut() {
                    *pixel = 0x001E3A5F;
                }

                let width = size.width as usize;

                let rect_x = 100usize;
                let rect_y = 80usize;
                let rect_height = 200usize;
                let rect_width = 120usize;

                // visit every row of the rectangle we have to form...
                for y in rect_y..(rect_y + rect_height) {
                    for x in rect_x..(rect_x + rect_width) {
                        let index = y * width + x;
                        buffer[index] = 0x00FF0000;
                    }
                }

                // before preenting the buffer we want to show the cursor...
                let (cursor_x, cursor_y) = self.pointer.pixel_position();

                // draw cursor..
                draw_cursor(
                    &mut buffer,
                    width,
                    size.height as usize,
                    cursor_x,
                    cursor_y
                );

                buffer.present().unwrap();
            }

            WindowEvent::Resized(_) => {
                // if the window that requested for redraw is the same window
                // then we resize it...
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }

            WindowEvent::CursorMoved{ position, ..} => {
                self.pointer.motion(position.x, position.y);
                // now when the cursor is moved we
                // now want it to redraw the pixels
                // as now the cursor postion is being updated constantly...
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }

            WindowEvent::MouseInput {state, button, ..} => {
                let pointer_button = match button {
                    winit::event::MouseButton::Left => Some(PointerButton::Left),
                    winit::event::MouseButton::Right => Some(PointerButton::Right),
                    winit::event::MouseButton::Middle => Some(PointerButton::Middle),
                    _ => None, 
                };

                if let Some(button) = pointer_button {
                    let pressed = state == ElementState::Pressed;

                    self.pointer.buttons.set(button, pressed);

                    println!(
                        "{:?} button: {}",
                        button,
                        if pressed { "pressed" } else { "released" }
                    );
                }
            }

            _ => (),
        }
    }
}

// function to call all the logic so that everything starts like it did before...
pub fn run() {
    let event_loop = winit::event_loop::EventLoop::new().unwrap();

    let mut app = App::default();

    event_loop.run_app(&mut app).unwrap();
}


// Draws a simple, high-contrast crosshair cursor.
//
// The cursor is centered on the pointer position.
// Black pixels form the outline; white pixels form the interior.
fn draw_cursor(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    cursor_x: i32,
    cursor_y: i32,
) {
    // Safely draw one pixel, ignoring coordinates outside the window.
    let mut draw_pixel = |x: i32, y: i32, color: u32| {
        if x >= 0 && y >= 0 {
            let x = x as usize;
            let y = y as usize;

            if x < width && y < height {
                buffer[y * width + x] = color;
            }
        }
    };

    // Draw the black outline first.
    for offset in -8..=8 {
        draw_pixel(cursor_x + offset, cursor_y, 0x00000000);
        draw_pixel(cursor_x, cursor_y + offset, 0x00000000);
    }

    // Draw the white interior over the outline's center.
    for offset in -7..=7 {
        if offset != 0 {
            draw_pixel(cursor_x + offset, cursor_y, 0x00FFFFFF);
            draw_pixel(cursor_x, cursor_y + offset, 0x00FFFFFF);
        }
    }

    // Keep the center black for a crisp intersection.
    draw_pixel(cursor_x, cursor_y, 0x00000000);
}
