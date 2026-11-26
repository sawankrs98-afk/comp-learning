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
    // this line lists the "parts" of the event we want to use.
    // `event_loop` : lets us ask winit to do things (like exit or redraw).
    // we removed `EventLoop` from this import because we never use that type
    // here, and unused imports produce a compiler warning...
    // `ControlFlow` tells winit WHEN to wake our program up again:
    // Wait = sleep until something happens, WaitUntil = sleep until a time.
    // we still need it for the movement timer below...
    event_loop::{ActiveEventLoop, ControlFlow},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};


// where the FIRST rectangle appears when the app starts...
const RECT_START_X: f64 = 100.0;
const RECT_START_Y: f64 = 80.0;
// how big every rectangle is (same for all of them for now)...
const RECT_WIDTH: f64 = 120.0;
const RECT_HEIGHT: f64 = 200.0;
// each new rectangle is pushed 40 pixels down-right from the last one,
// so when you spawn several they fan out like a deck of cards...
const SPAWN_STEP: f64 = 40.0;
// colours for new rectangles, one after another (RRGGBB as hex numbers).
// `%` (remainder) cycles back to the first colour after the last one...
const RECT_COLORS: [u32; 4] = [0x00FF0000, 0x0000CC44, 0x004477FF, 0x00FFCC00];

const ARROW_TAP_STEP: f64 = 10.0;
const KEYBOARD_SPEED: f64 = 200.0;
const KEYBOARD_INTERVAL: Duration = Duration::from_millis(16);
const MAX_KEYBOARD_ELAPSED: Duration = Duration::from_millis(50);


// One rectangle that manages itself: it knows its own position, size and
// colour. Before this struct there was ONE hard-coded rectangle spread
// across many separate App fields; now we can have as many as we want...
struct Rect {
    // top-left corner inside the window. We store the raw position
    // (not an offset from 100/80) because each rect now starts at
    // a different place...
    offset_x: f64,
    offset_y: f64,

    // size, same for every rect for now...
    width: f64,
    height: f64,

    // fill colour as a 0x00RRGGBB pixel value, exactly what the
    // framebuffer wants...
    color: u32,
}

// "impl Rect" = functions that belong to Rect. You call them like
// Rect::spawn(3) — no existing rect needed, so it's like a constructor...
impl Rect {
    // builds the n-th rectangle (0th, 1st, 2nd...). `n as f64` converts
    // the whole number into a decimal so we can multiply by SPAWN_STEP...
    fn spawn(n: usize) -> Self {
        Self {
            offset_x: RECT_START_X + SPAWN_STEP * n as f64,
            offset_y: RECT_START_Y + SPAWN_STEP * n as f64,
            width: RECT_WIDTH,
            height: RECT_HEIGHT,
            // n % 4 gives 0,1,2,3,0,1,2,3... so colours repeat forever...
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

    // ALL the rectangles, stored in a Vec (a growable list).
    // THE ORDER IS THE STACKING: index 0 is drawn first (bottom of the
    // stack), the LAST index is drawn last (on top of everything).
    // "raising" a rectangle to the front = moving it to the end of the Vec...
    rects: Vec<Rect>,

    // which rectangle has keyboard focus, as its index into `rects`.
    // Option means "maybe none": we start with no focus at all, and a
    // background click clears it. We NEVER use .unwrap() here, so a
    // missing focus can never crash the program...
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

                // draw EVERY rectangle, first to last. Because the last one
                // paints its pixels last, it covers the ones below it —
                 // that IS the stacking order, no extra bookkeeping needed...
                for (i, rect) in self.rects.iter().enumerate() {
                    // f64 -> usize: pixel coordinates must be whole numbers...
                    let rect_x = rect.offset_x as usize;
                    let rect_y = rect.offset_y as usize;
                    let rect_width = rect.width as usize;
                    let rect_height = rect.height as usize;

                    // fill the rectangle row by row, like before...
                    for y in rect_y..(rect_y + rect_height) {
                        for x in rect_x..(rect_x + rect_width) {
                            // framebuffer is one flat list of pixels:
                            // row y, column x lives at position y*width + x...
                            let index = y * width + x;
                            // bounds check: a rect dragged partly off the
                            // window would otherwise crash here (index out
                            // of range). `if` + skip instead of .unwrap()...
                            if index < buffer.len() {
                                buffer[index] = rect.color;
                            }
                        }
                    }

                    // draw a 1-pixel WHITE outline on the focused rectangle
                    // so you can SEE which one the keyboard controls...
                    if Some(i) == self.focused {
                        for y in rect_y..(rect_y + rect_height) {
                            for x in rect_x..(rect_x + rect_width) {
                                // a pixel is on the outline if it sits on any
                                // of the four edges of the rectangle...
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
                    // COPY the grab offsets into plain local variables FIRST.
                    // Why? `focused_rect_mut()` borrows `self` mutably, and
                    // Rust forbids touching `self.drag_offset_x` through
                    // `self` while that borrow is alive (error E0503).
                    // Two short copies before the borrow = no overlap...
                    let grab_x = self.drag_offset_x;
                    let grab_y = self.drag_offset_y;

                    // run the body only if something is focused;
                    // `if let` = "if the Option is Some, give me the value"...
                    if let Some(rect) = self.focused_rect_mut() {
                        // the rect jumps to where it was grabbed: cursor
                        // position minus the grab point inside the rect...
                        rect.offset_x = position.x - grab_x;
                        rect.offset_y = position.y - grab_y;
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
                        // HIT-TEST: which rectangle is under the cursor?
                        // `rposition` searches the Vec BACKWARDS (from the
                        // last index to 0). That matters: when two rectangles
                        // overlap, the one later in the Vec is drawn on top,
                        // so a click must land on IT, not the hidden one...
                        let hit = self.rects.iter().rposition(|rect| {
                            // the same four-way check as before, but per rect.
                            // `rect` here is `&Rect`; the comparison works
                            // because the closure receives each element...
                            self.pointer.x >= rect.offset_x
                                && self.pointer.x < rect.offset_x + rect.width
                                && self.pointer.y >= rect.offset_y
                                && self.pointer.y < rect.offset_y + rect.height
                        });

                        // `match` on the Option<usize> we just got...
                        match hit {
                            Some(i) => {
                                // RAISE TO TOP: remove() takes the rect out of
                                // the Vec (shifting the rest left) and returns
                                // it; push() puts it back at the END = on top.
                                // Classic way to reorder a stack in Rust...
                                let rect = self.rects.remove(i);
                                self.rects.push(rect);

                                // the raised rect is now the last element, so
                                // its new index is len() - 1. Focus follows it...
                                self.focused = Some(self.rects.len() - 1);

                                self.dragging = true;

                                // remember WHERE inside the rect you grabbed
                                // it, so it doesn't snap its corner to the
                                // cursor. We look it up by that same index...
                                let rect = &self.rects[self.rects.len() - 1];
                                self.drag_offset_x = self.pointer.x - rect.offset_x;
                                self.drag_offset_y = self.pointer.y - rect.offset_y;
                            }
                            // clicked on empty background: no rectangle
                            // should listen to the keyboard any more...
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

                    // The movement timer handles repeats. A synthetic press on
                    // focus gain should also wait for a fresh press.
                    if pressed && (event.repeat || is_synthetic) {
                        return;
                    }

                    // set the pressed key to hash
                    self.keyboard.set_key(code, pressed);

                    // perform the action only on the initial key press...
                    if pressed  {
                        let mut needs_redraw = false;

                        match code {
                            KeyCode::ArrowLeft => {
                                needs_redraw = self.move_with_keyboard(-ARROW_TAP_STEP, 0.0);
                            }
                            KeyCode::ArrowRight => {
                                needs_redraw = self.move_with_keyboard(ARROW_TAP_STEP, 0.0);
                            }
                            KeyCode::ArrowUp => {
                                needs_redraw = self.move_with_keyboard(0.0, -ARROW_TAP_STEP);
                            }
                            KeyCode::ArrowDown => {
                                needs_redraw = self.move_with_keyboard(0.0, ARROW_TAP_STEP);
                            }
                            KeyCode::Space => {
                                // SPAWN a new rectangle: len() is how many we
                                // have, which is also the index the new one
                                // will get — so spawn(len) staggers it down-
                                // right from all the previous ones...
                                self.rects.push(Rect::spawn(self.rects.len()));

                                // the new rect sits at the END of the Vec,
                                // which means ON TOP of the stack. Focus it
                                // right away so arrows move the newest one...
                                self.focused = Some(self.rects.len() - 1);
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

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Right and down are positive. Opposite arrows add up to zero.
        let mut horizontal = 0.0;
        let mut vertical = 0.0;
        if self.keyboard.is_pressed(KeyCode::ArrowRight) {
            horizontal += 1.0;
        }
        if self.keyboard.is_pressed(KeyCode::ArrowLeft) {
            horizontal -= 1.0;
        }
        if self.keyboard.is_pressed(KeyCode::ArrowDown) {
            vertical += 1.0;
        }
        if self.keyboard.is_pressed(KeyCode::ArrowUp) {
            vertical -= 1.0;
        }

        if self.dragging || (horizontal == 0.0 && vertical == 0.0) {
            // Start a fresh clock when keyboard movement resumes.
            self.last_keyboard_update = None;
            event_loop.set_control_flow(ControlFlow::Wait);
            return;
        }

        let now = Instant::now();
        // On the first pass of a hold, start the clock at now.
        let mut last_update = self.last_keyboard_update.unwrap_or(now);
        let elapsed = now.duration_since(last_update);

        if elapsed >= KEYBOARD_INTERVAL {
            // Distance = speed times seconds. Limit the step after a long pause.
            let seconds = elapsed.min(MAX_KEYBOARD_ELAPSED).as_secs_f64();
            let distance = KEYBOARD_SPEED * seconds;
            if self.move_with_keyboard(horizontal * distance, vertical * distance) {
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            last_update = now;
        }

        // An early wake-up leaves last_update alone, so the deadline stays put.
        self.last_keyboard_update = Some(last_update);
        event_loop.set_control_flow(ControlFlow::WaitUntil(last_update + KEYBOARD_INTERVAL));
    }
}

impl App {
    // Gives a MUTABLE borrow (`&mut` = "you may change it") of the rectangle
    // that currently has focus. Returns Option: Some(&mut Rect) when focused,
    // None when nothing is focused. The `?` at the end means: if
    // self.focused is None, return None from THIS function right away;
    // otherwise hand over the usize index it contained...
    fn focused_rect_mut(&mut self) -> Option<&mut Rect> {
        // get_mut(index) is the SAFE way to index a Vec: it returns an Option
        // instead of crashing when the index is out of range...
        self.rects.get_mut(self.focused?)
    }

    // Both taps and holds use this so keyboard movement stays inside the buffer
    // bounds that the existing rectangle drawing loop expects.
    fn move_with_keyboard(&mut self, dx: f64, dy: f64) -> bool {
        // `let Some(window) = ... else { return false };` = "unwraps" the
        // Option<Arc<Window>>: if there is no window yet, we return early;
        // otherwise `window` is the real value for the rest of the function...
        let Some(window) = &self.window else {
            return false;
        };
        let size = window.inner_size();

        // Copy the dragging flag into a local BEFORE the mutable borrow below.
        // Rust's borrow rules: while `rect` holds a &mut borrow of self, we
        // cannot ALSO read self.dragging. Copying first keeps the borrows
        // from overlapping (this is exactly error E0503 otherwise)...
        let dragging = self.dragging;

        // keyboard movement goes to the FOCUSED rectangle only; when nothing
        // is focused there is nothing to move, so return "did not move"...
        let Some(rect) = self.focused_rect_mut() else {
            return false;
        };
        
        // clamp bounds use THIS rectangle's own size, so every rectangle
        // stops exactly at the window edge whatever its dimensions are...
        let max_x = f64::from(size.width) - rect.width;
        let max_y = f64::from(size.height) - rect.height;
        if dragging || max_x < 0.0 || max_y < 0.0 {
            return false;
        }

        // Positions are stored directly now (no RECT_START_X base to add and
        // subtract). clamp() keeps the new value inside [0, max], so a held
        // arrow can never push the rectangle off-screen...
        let x = (rect.offset_x + dx).clamp(0.0, max_x);
        let y = (rect.offset_y + dy).clamp(0.0, max_y);

        // Only report "moved" when a pixel actually changed, so the caller
        // does not waste time redrawing on a blocked arrow key...
        let moved = x != rect.offset_x || y != rect.offset_y;
        rect.offset_x = x;
        rect.offset_y = y;
        moved
    }
}








// function to call all the logic so that everything starts like it did before...
pub fn run() {
    let event_loop = winit::event_loop::EventLoop::new().unwrap();

    let mut app = App::default();

    // Seed the FIRST rectangle so the app looks the same as before on startup.
    // Rect::spawn(0) = the 0th rectangle: starts at RECT_START_X/Y, red.
    // push() adds it to the END of the Vec = bottom of the (one-item) stack...
    app.rects.push(Rect::spawn(0));

    // Focus it from the start so the arrow keys work immediately, like in
    // Round 3. Some(0) = "index 0 has focus"...
    app.focused = Some(0);

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
