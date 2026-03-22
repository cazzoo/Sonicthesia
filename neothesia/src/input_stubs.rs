pub struct UnifiedInputManager {
    elements: Vec<FocusableElement>,
    focused_id: Option<String>,
}

impl UnifiedInputManager {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
            focused_id: None,
        }
    }

    pub fn focus(&mut self) -> &mut Self {
        self
    }

    pub fn priority(&mut self) -> &mut Self {
        self
    }

    pub fn set_cursor_visibility_callback(&mut self, _callback: Box<dyn Fn(bool)>) {}

    pub fn register_element(&mut self, element: FocusableElement) {
        self.elements.push(element);
    }

    pub fn elements(&self) -> &Vec<FocusableElement> {
        &self.elements
    }

    pub fn set_focus(&mut self, id: &str) {
        self.focused_id = Some(id.to_string());
    }

    pub fn focused_index(&self) -> Option<usize> {
        self.focused_id
            .as_ref()
            .and_then(|id| self.elements.iter().position(|e| e.id == *id))
    }

    pub fn focused_element(&mut self) -> Option<&mut FocusableElement> {
        let focused_id = self.focused_id.clone()?;
        self.elements.iter_mut().find(|e| e.id == *focused_id)
    }

    pub fn get_priority(&self) -> InputPriority {
        InputPriority::None
    }

    pub fn has_mouse_priority(&self) -> bool {
        false
    }

    pub fn has_keyboard_priority(&self) -> bool {
        false
    }

    pub fn set_keyboard_priority(&mut self) {}

    pub fn update_mouse_position(&mut self, _x: f32, _y: f32) {}

    pub fn update(&mut self, _delta: f64) {}

    pub fn focus_next(&mut self) {
        if let Some(current_idx) = self.focused_index() {
            if !self.elements.is_empty() {
                let next_idx = (current_idx + 1) % self.elements.len();
                self.focused_id = Some(self.elements[next_idx].id.clone());
            }
        } else if !self.elements.is_empty() {
            self.focused_id = Some(self.elements[0].id.clone());
        }
    }

    pub fn focus_previous(&mut self) {
        if let Some(current_idx) = self.focused_index() {
            if !self.elements.is_empty() {
                let prev_idx = if current_idx == 0 {
                    self.elements.len() - 1
                } else {
                    current_idx - 1
                };
                self.focused_id = Some(self.elements[prev_idx].id.clone());
            }
        } else if !self.elements.is_empty() {
            self.focused_id = Some(self.elements[0].id.clone());
        }
    }

    pub fn update_element_position(&mut self, id: &str, position: (f32, f32)) {
        if let Some(element) = self.elements.iter_mut().find(|e| e.id == id) {
            element.position = position;
        }
    }
}

pub struct FocusableElement {
    pub id: String,
    pub label: String,
    pub element_type: ElementType,
    pub position: (f32, f32),
    pub size: (f32, f32),
    pub focusable: bool,
}

pub enum ElementType {
    Button,
    Toggle,
    Spinner,
    Slider,
    Picker,
}

#[derive(PartialEq)]
pub enum InputPriority {
    None,
    Mouse,
    Keyboard,
}
