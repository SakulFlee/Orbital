use orbital_ecs::Entity;

/// Event emitted when a button is pressed.
#[derive(Debug, Clone)]
pub struct ButtonPressed {
    /// The entity that owns the button.
    pub entity: Entity,
    /// The button's ID.
    pub id: String,
}

/// Event emitted when a button is released.
#[derive(Debug, Clone)]
pub struct ButtonReleased {
    /// The entity that owns the button.
    pub entity: Entity,
    /// The button's ID.
    pub id: String,
}

/// Event emitted when a text box value changes.
#[derive(Debug, Clone)]
pub struct TextBoxChanged {
    /// The entity that owns the text box.
    pub entity: Entity,
    /// The text box's ID.
    pub id: String,
    /// The new text value.
    pub value: String,
}

/// Event emitted when a checkbox is toggled.
#[derive(Debug, Clone)]
pub struct CheckboxToggled {
    /// The entity that owns the checkbox.
    pub entity: Entity,
    /// The checkbox's ID.
    pub id: String,
    /// The new checked state.
    pub checked: bool,
}

/// Event emitted when a text box gains or loses focus.
#[derive(Debug, Clone)]
pub struct FocusChanged {
    /// The entity that gained or lost focus.
    pub entity: Entity,
    /// The element's ID.
    pub id: String,
    /// Whether the element now has focus.
    pub focused: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn button_pressed_event() {
        let entity = Entity::new(0, 0);
        let event = ButtonPressed {
            entity,
            id: "play".into(),
        };
        assert_eq!(event.entity, entity);
        assert_eq!(event.id, "play");
    }

    #[test]
    fn checkbox_toggled_event() {
        let entity = Entity::new(1, 0);
        let event = CheckboxToggled {
            entity,
            id: "accept".into(),
            checked: true,
        };
        assert!(event.checked);
    }
}
