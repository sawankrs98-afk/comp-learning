use crate::keyboard::Keyboard;
use crate::pointer::{Pointer, PointerButton};
use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
// this is the library for the synchronization...
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};


// first rectangle starts here; later ones are staggered so all stay visible...
const RECT_START_X: f64 = 100.0;
const RECT_START_Y: f64 = 80.0;
// each newly spawned rectangle is offset by this much from the previous one...
const SPAWN_STEP: f64 = 40.0;
const RECT_WIDTH: f64 = 120.0;
const RECT_HEIGHT: f64 = 200.0;
// colours cycled through by newly spawned rectangles...
const RECT_COLORS: [u32; 4] = [0x00FF0000, 0x0000CC44, 0x004477FF, 0x00FFCC00];
const ARROW_TAP_STEP: f64 = 10.0;
const KEYBOARD_SPEED: f64 = 200.0;
const KEYBOARD_INTERVAL: Duration = Duration::from_millis(16);
const MAX_KEYBOARD_ELAPSED: Duration = Duration::from_millis(50);


// one independently managed rectangle: its own position offset, size and
// colour. Vec order in App is the stacking order: last = drawn on top...
struct Rect {
    offset_x: f64,
    offset_y: f64,
    width: f64,
    height: f64,
    color: u32,
}

impl Rect {
    // spawn the n-th rectangle, staggered and coloured so it is distinguishable...
    fn spawn(n: usize) -> Self {
        Self {
            offset_x: RECT_START_X + SPAWN_STEP * n as f64,
            offset_y: RECT_START_Y + SPAWN_STEP * n as f64,
            width: RECT_WIDTH,
            height: RECT_HEIGHT,
            color: RECT_COLORS[n % RECT_COLORS.len()],
        }
    }
}

#[derive(Default)]
struct App {
    // stores the state of the window...
    // we dont have any window at the start
    // so we have to store it as an option as option --> somehting, NUll..
    // both the context and the surface needs to keep the window alive
    // so we use Arch because it keeps the window alive
    // Arc lets us share the window object without actually transferring the window object...
    // so kind of lets both softbuffer and the App use the same window without transferring the object
    
    window: Option<Arc<Window>>,

    // stores softbuffering connection to the windowing system...
    context: Option<Context<Arc<Window>>>,

    // storing the drawable surface used to present pixels...
    surface: Option<Surface<Arc<Window>, Arc<Window>>>,

    // added the pointer impl
    pointer: Pointer,

    // all rectangles, back to front: the last one is drawn on top...
    rects: Vec<Rect>,

    // index into rects of the rectangle that owns keyboard input and dragging...
    focused: Option<usize>,

    // so like when we drag... this will be true
    dragging: bool,

    // when dragging is true then the cursor will move that will be the offset...
    drag_offset_x: f64,
    drag_offset_y: f64,

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
            let window = Arc::new(event_loop.create_window(attributes).unwrap());
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

                // draw every rectangle back to front so the last one stacks on top...
                for (i, rect) in self.rects.iter().enumerate() {
                    let rect_x = rect.offset_x as usize;
                    let rect_y = rect.offset_y as usize;
                    let rect_width = rect.width as usize;
                    let rect_height = rect.height as usize;

                    // visit every row of the rectangle we have to form...
                    for y in rect_y..(rect_y + rect_height) {
                        for x in rect_x..(rect_x + rect_width) {
                            let index = y * width + x;
                            // bounds check: rectangles may be nudged partly off-screen...
                            if index < buffer.len() {
                                buffer[index] = rect.color;
                            }
                        }
                    }

                    // outline the focused rectangle so focus is visible...
                    if Some(i) == self.focused {
                        for y in rect_y..(rect_y + rect_height) {
                            for x in rect_x..(rect_x + rect_width) {
                                let on_edge = x == rect_x
                                    || x == rect_x + rect_width - 1
                                    || y == rect_y
                                    || y == rect_y + rect_height - 1;
                                if on_edge {
                                    let index = y * width + x;
                                    if index < buffer.len() {
                                        buffer[index] = 0x00FFFFFF;
                                    }
                                }
                            }
                        }
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
                    if let Some(rect) = self.focused_rect_mut() {
                        // the focused rectangle follows the cursor minus the grab offset...
                        rect.offset_x = position.x - self.drag_offset_x;
                        rect.offset_y = position.y - self.drag_offset_y;
                    }
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
                        // hit-test from the top of the stack down so clicks land
                        // on the topmost rectangle under the cursor...
                        let hit = self.rects.iter().rposition(|rect| {
                            self.pointer.x >= rect.offset_x
                                && self.pointer.x < rect.offset_x + rect.width
                                && self.pointer.y >= rect.offset_y
                                && self.pointer.y < rect.offset_y + rect.height
                        });

                        match hit {
                            Some(i) => {
                                // focus the clicked rectangle and raise it to the top...
                                let rect = self.rects.remove(i);
                                self.rects.push(rect);
                                self.focused = Some(self.rects.len() - 1);

                                self.dragging = true;
                                let rect = &self.rects[self.rects.len() - 1];
                                self.drag_offset_x = self.pointer.x - rect.offset_x;
                                self.drag_offset_y = self.pointer.y - rect.offset_y;
                            }
                            // clicking empty space clears the focus...
                            None => self.focused = None,
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

            WindowEvent::KeyboardInput { event, is_synthetic, .. } => {
                // uses physical code to see the physical key...
                if let PhysicalKey::Code(code) = event.physical_key {
                    // to see whether the key is pressed or no...
                    let pressed = event.state == ElementState::Pressed;

                    if pressed && (event.repeat || is_synthetic) {
                        return;
                    }

                    // set the pressed key to hash
                    self.keyboard.set_key(code, pressed);

                    // perform the action only on the initial key press...
                    if pressed   {
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
                self.last_keyboard_update = None;
                // stop dragging as the focus is now lost...
                self.dragging = false;
            }

            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop : &ActiveEventLoop){
        let mut horizontal = 0.0;
        let mut vertical = 0.0;

        if self.keyboard.is_pressed(KeyCode::ArrowRight){
            horizontal += 1.0;
        }
        
        if self.keyboard.is_pressed(KeyCode::ArrowLeft){
            horizontal -= 1.0;
        }
        
        if self.keyboard.is_pressed(KeyCode::ArrowUp){
            vertical += 1.0;
        }
        
        if self.keyboard.is_pressed(KeyCode::ArrowDown){
            vertical -= 1.0;
        }

        if self.dragging || (horizontal == 0.0 && vertical == 0.0) {
            self.last_keyboard_update = None;
            event_loop.set_control_flow(ControlFlow::Wait);
            return;
        }

        let now = Instant::now();

        // start the clock on the first instant of now...
        let mut last_update = self.last_keyboard_update.unwrap_or(now);
        let elapsed = now.duration_since(last_update);

        if elapsed >= KEYBOARD_INTERVAL {
            let seconds = elapsed.min(MAX_KEYBOARD_ELAPSED).as_secs_f64();
            let distance = KEYBOARD_SPEED * seconds;

            if self.move_with_keyboard(horizontal * distance, vertical * distance){
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            last_update = now;
        }

        self.last_keyboard_update = Some(last_update);
        event_loop.set_control_flow(ControlFlow::WaitUntil(last_update + KEYBOARD_INTERVAL));
    }
}

// this is for the bounds of the rectangle
// if it goes out of bounds then we can stop it 
// and use maths to render it better
impl App {
    fn move_with_keyboard(&mut self, dx : f64, dy : f64) -> bool{
        let Some(window) = &self.window else {
            return false;
        };

        let size = window.inner_size();

        let max_x = f64::from(size.width) - RECT_WIDTH as f64;
        let max_y = f64::from(size.height) - RECT_HEIGHT as f64;
        
        // either no dragging or the free range of the rectangle 
        // isnt good enough...
        if self.dragging || max_x < 0.0 || max_y < 0.0 {
            return false;
        }

        let x = (RECT_START_X + self.rect_offset_x + dx).clamp(0.0, max_x);
        let y = (RECT_START_Y + self.rect_offset_y + dy).clamp(0.0, max_y);
        
        let offset_x = x - RECT_START_X;
        let offset_y = y - RECT_START_Y;

        let moved = offset_x != self.rect_offset_x || offset_y != self.rect_offset_y;
        self.rect_offset_x = offset_x;
        self.rect_offset_y = offset_y;

        moved
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
