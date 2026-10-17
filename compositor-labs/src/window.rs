// all the things to build a window...
use softbuffer::{Context, Surface};
use std::{num::NonZeroU32, option};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{self, ActiveEventLoop, EventLoop},
    keyboard::PhysicalKey,
    window::{self, Window, WindowId},
};

// for the pointer movements and things
use crate::pointer::{self, Pointer, PointerButton};
use winit::event::ElementState;

// for the key states and presses...
use crate::keyboard::Keyboard;
use winit::keyboard::KeyCode;

// imports for the time
use std::time::{Duration, Instant};
use winit::event_loop::ControlFlow;








#[derive(Default)]
struct App {
    // stores the state of the window...
    // we dont have any window at the start
    // so we have to store it as an option as option --> somehting, NUll..
    // both the context and the surface needs to keep the window alive
    // so we use Arch because it keeps the window alive
    // Arc lets us share the window object without actually transferring the window object...
    window: Option<std::sync::Arc<Window>>,

    // stores softbuffering connection to the windowing system...
    context: Option<Context<std::sync::Arc<Window>>>,

    // storing the drawable surface used to present pixels...
    surface: Option<Surface<std::sync::Arc<Window>, std::sync::Arc<Window>>>,

    // added the pointer impl
    pointer: Pointer,

    // so like when we drag... this will be true
    dragging: bool,

    // when dragging is true then the cursor will move that will be the offset...
    drag_offset_x: f64,
    drag_offset_y: f64,

    // when the cusor has offset we will calculate the rectangle offset and it will be used when the rectangle is redrawn...
    rect_offset_x: f64,
    rect_offset_y: f64,

    // the keyboard class which already has Keybaord...
    keyboard: Keyboard,

    last_keyboard_update : Option<Instant>,
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
            window.set_cursor_visible(false);

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

                let rect_x = (100.0 + self.rect_offset_x) as usize;
                let rect_y = (80.0 + self.rect_offset_y) as usize;
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
                draw_cursor(&mut buffer, width, size.height as usize, cursor_x, cursor_y);

                buffer.present().unwrap();
            }

            WindowEvent::Resized(_) => {
                // if the window that requested for redraw is the same window
                // then we resize it...
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.pointer.motion(position.x, position.y);
                // now when the cursor is moved we
                // now want it to redraw the pixels
                // as now the cursor postion is being updated constantly...

                // also if the window is being dragged then we need to redraw it also when it moves with the cursor...
                if self.dragging {
                    self.rect_offset_x = position.x - self.drag_offset_x - 100.0;
                    self.rect_offset_y = position.y - self.drag_offset_y - 80.0;
                }

                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }

            WindowEvent::MouseInput { state, button, .. } => {
                let pointer_button = match button {
                    winit::event::MouseButton::Left => Some(PointerButton::Left),
                    winit::event::MouseButton::Right => Some(PointerButton::Right),
                    winit::event::MouseButton::Middle => Some(PointerButton::Middle),
                    _ => None,
                };

                let pressed = state == ElementState::Pressed;

                if let Some(pointer_button) = pointer_button {
                    self.pointer.buttons.set(pointer_button, pressed);

                    println!(
                        "{:?} button: {}",
                        pointer_button,
                        if pressed { "pressed" } else { "released" }
                    );

                    if pointer_button == PointerButton::Left && pressed {
                        let rect_x = 100.0 + self.rect_offset_x;
                        let rect_y = 80.0 + self.rect_offset_y;

                        let rect_width = 120.0;
                        let rect_height = 200.0;

                        let inside_rectangle = self.pointer.x >= rect_x
                            && self.pointer.x < rect_x + rect_width
                            && self.pointer.y >= rect_y
                            && self.pointer.y < rect_y + rect_height;

                        if inside_rectangle {
                            self.dragging = true;

                            self.drag_offset_x = self.pointer.x - rect_x;
                            self.drag_offset_y = self.pointer.y - rect_y;

                            if let Some(window) = &self.window {
                                window.request_redraw();
                            }
                        }
                    }

                    if pointer_button == PointerButton::Left && !pressed {
                        self.dragging = false;
                    }

                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
            }

            WindowEvent::KeyboardInput { event, .. } => {
                // uses physical code to see the physical key...
                if let PhysicalKey::Code(code) = event.physical_key {
                    // to see whether the key is pressed or no...
                    let pressed = event.state == ElementState::Pressed;

                    // set the pressed key to hash
                    self.keyboard.set_key(code, pressed);

                    // perform the action only on the initial key press...
                    if pressed  {
                        let mut needs_redraw = false;

                        match code {
                            KeyCode::ArrowLeft => {
                                self.rect_offset_x -= 10.0;
                                needs_redraw = true;
                            }
                            KeyCode::ArrowRight => {
                                self.rect_offset_x += 10.0;
                                needs_redraw = true;
                            }
                            KeyCode::ArrowUp => {
                                self.rect_offset_y -= 10.0;
                                needs_redraw = true;
                            }
                            KeyCode::ArrowDown => {
                                self.rect_offset_y += 10.0;
                                needs_redraw = true;
                            }
                            KeyCode::Space => {
                                needs_redraw = true;
                            }
                            KeyCode::Escape => {
                                event_loop.exit();
                            }

                            _ => {}
                        }

                        if needs_redraw {
                            if let Some(window) = &self.window {
                                window.request_redraw();
                            }
                        }
                    }
                }
            }

            // this window event is for when the window will lose focus
            // there can be when the keys will still be pressed but
            // the focus will be lost...
            // so for that
            WindowEvent::Focused(false) => {
                // clear all the keys for this specific window...
                self.keyboard.clear();

                // stop dragging as the focus is now lost...
                self.dragging = false;
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
fn draw_cursor(buffer: &mut [u32], width: usize, height: usize, cursor_x: i32, cursor_y: i32) {
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
