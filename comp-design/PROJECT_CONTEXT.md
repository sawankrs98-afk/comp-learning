# Compositor project context

## Purpose and workspace

Build a Wayland compositor in Rust by learning windowing, rendering, input, and window management in stages. The long-term layout goal is horizontally scrolling columns with vertically arranged windows, inspired by Niri.

Develop in `~/dummy-comp/compositor-labs`. Design material lives in `~/dummy-comp/comp-design`. The handoff names `my-compositor` as a Smallvil reference project, but that directory was absent from `~/dummy-comp` during setup.

## Architecture

- `src/main.rs` declares the modules and calls `window::run()`.
- `src/window.rs` owns the Winit event loop and application state. Winit delivers window and input events to `App`. Input handlers update state and request a redraw. The redraw handler uses Softbuffer to paint the background, rectangle, and crosshair, then presents the buffer.
- `src/pointer.rs` tracks pointer coordinates and mouse buttons. `App` uses those coordinates for hit-testing and dragging. Rendering uses rounded pixel coordinates for the crosshair.
- `src/keyboard.rs` stores held physical key codes in a `HashSet`. Press and release events update the set. Losing window focus clears it and cancels dragging.
- `src/render.rs` is empty. Drawing currently lives in `window.rs`.

The package remains `compositor-labs` and uses Rust edition 2024. The copied lockfile pins Winit 0.30.13 and Softbuffer 0.4.8. The lab uses a host window with simulated content. Wayland server and client-surface integration begin in later rounds.

## Eight-round roadmap

| Round | Lab | Status |
| --- | --- | --- |
| 1 | Native window and software rendering | Complete per prior testing |
| 2 | Pointer and mouse input | Basic crosshair, hit-testing, and dragging complete per prior testing |
| 3 | Keyboard input and focus | Complete; user confirmed continuous movement on desktop |
| 4 | Mini window manager | In progress: multiple rects, focus and stacking done in code, desktop run pending |
| 5 | Minimal Wayland server with Smithay | Planned |
| 6 | Real Wayland surfaces and buffers | Planned |
| 7 | XDG toplevel windows | Planned |
| 8 | Integrate into a working nested compositor | Planned |

## Round 3 implementation and verification

`cargo check --locked` passes with zero warnings. The user ran the app on the desktop and confirmed continuous arrow movement works; Round 3 is verified interactively.

- A fresh arrow press moves the rectangle by 10 pixels. Holding an arrow also moves it at 200 pixels per second along that axis. The event loop aims for a movement update every 16 ms and uses elapsed time to calculate distance.
- `about_to_wait()` uses `WaitUntil` when movement is active and `Wait` when idle. It caps a delayed update at 50 ms of movement. Opposite arrows cancel during held movement. Keyboard movement pauses while dragging.
- Space requests a redraw. The rectangle is always drawn red, so there is no visible color toggle.
- Escape exits. Focus loss clears held keys, resets the movement clock, and stops dragging. Repeat presses and synthetic focus-gain presses do not start actions.
- Keyboard movement keeps the rectangle within the window to match the existing drawing loop. The original drawing loop can still exceed the buffer when dragging or resizing.

## Round 4 implementation (mini window manager)

- `window.rs` defines a `Rect` struct (position offset, size, colour); `App.rects: Vec<Rect>` holds all rectangles with Vec order as stacking order (last = top). `App.focused: Option<usize>` indexes the focused rectangle; `focused_rect_mut()` returns `Option<&mut Rect>` via `get_mut` + `?`.
- Rendering loops over `rects` back to front, so later rectangles stack on top; the focused rectangle gets a white 1px outline. Per-pixel bounds checks keep off-screen rectangles from panicking.
- Click hit-tests top-down with `Vec::rposition`; a hit is focused and raised to the top (`Vec::remove` + `push`); clicking empty background clears focus. Dragging always moves the focused rectangle.
- Space spawns a new staggered rectangle (`SPAWN_STEP` 40px, colour cycled from `RECT_COLORS`) and focuses it. Arrow taps and continuous movement act on the focused rectangle only, with per-rectangle clamp bounds.
- Fixed since Round 3: unused `EventLoop` import, redundant parens, dead `Pointer::new`, dead `ARROW_TAP_STEP` constant now used by taps.

## Next lab

Verify Round 4 on the desktop: spawn several rectangles with Space, click-to-focus with raise, background click clearing focus, arrows moving only the focused rectangle, then move to Round 5 (Smithay) only after that.
