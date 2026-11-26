use std::collections::HashSet;
use winit::keyboard::KeyCode;

// stores the current keys which are pressed...
#[derive(Debug, Default)]
pub struct Keyboard {
    pressed_keys: HashSet<KeyCode>,
}

impl Keyboard {
    // function to update the state of a key when it is pressed or released...
    pub fn set_key(&mut self, key:KeyCode, pressed : bool) {
        if pressed {
            self.pressed_keys.insert(key);
        }
        else {
            self.pressed_keys.remove(&key);
        }
    }

    // shared borrow is enough as we just have to check the thing 
    pub fn is_pressed(&self, key : KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }
    // when the window loses focus
    pub fn clear(&mut self) {
        self.pressed_keys.clear();
    }
}
