// the pointer only needs a x and y val to show itself...
#[derive(Debug, Default)]
pub struct Pointer {
    pub x: f64,
    pub y: f64,
    pub buttons : PointerButtons,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// the mouse pointer our system supports...

pub enum PointerButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Default)]
pub struct PointerButtons {
    pub left: bool,
    pub right: bool,
    pub middle: bool,
}

impl PointerButtons {
    // updates the state of a button...
    pub fn set (&mut self, button : PointerButton, pressed : bool) {
        match button {
            PointerButton::Left => self.left = pressed,
            PointerButton::Right => self.right = pressed,
            PointerButton::Middle => self.middle = pressed,
        }
    }
}

impl Pointer {
    // creates a pointer at the initial position (0,0)...
    pub fn new() -> Self {
        Self::default()
    }

    // now we need to update the pointer position
    // some other event handlers see the mouse moving and give its current location
    // we will intercept them
    // and update our mouse position automatically...

    pub fn motion(&mut self, x: f64, y: f64) -> () {
        self.x = x;
        self.y = y;
    }

    pub fn pixel_position(&self) -> (i32, i32) {
        (self.x.round() as i32, self.y.round() as i32)
    }
}
