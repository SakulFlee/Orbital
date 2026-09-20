use orbital_ecs::{Entity, World};

use crate::components::{UiBackground, UiElement, UiEntity, UiFocus, UiTextBox};
use crate::layout::UiLayout;

/// Creates a text box entity with the given layout.
pub fn create_textbox(
    world: &mut World,
    id: impl Into<String>,
    placeholder: impl Into<String>,
    layout: UiLayout,
) -> Entity {
    let entity = world.spawn_entity();

    world
        .attach_component(&entity, UiEntity)
        .expect("Failed to attach UiEntity");
    world
        .attach_component(&entity, UiElement::new(id))
        .expect("Failed to attach UiElement");
    world
        .attach_component(&entity, UiTextBox::new().with_placeholder(placeholder))
        .expect("Failed to attach UiTextBox");
    world
        .attach_component(&entity, layout)
        .expect("Failed to attach UiLayout");
    world
        .attach_component(&entity, UiBackground::rounded([0.15, 0.15, 0.15, 0.9], 4.0))
        .expect("Failed to attach UiBackground");
    world
        .attach_component(&entity, UiFocus::default())
        .expect("Failed to attach UiFocus");

    entity
}

/// Inserts a character at the cursor position in a text box.
pub fn insert_char(textbox: &mut UiTextBox, ch: char) {
    textbox.value.insert(textbox.cursor_pos, ch);
    textbox.cursor_pos += 1;
}

/// Deletes the character before the cursor position in a text box.
pub fn delete_char(textbox: &mut UiTextBox) {
    if textbox.cursor_pos > 0 {
        textbox.cursor_pos -= 1;
        textbox.value.remove(textbox.cursor_pos);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_textbox_entity() {
        let mut world = World::new();
        let entity = create_textbox(&mut world, "tb1", "Enter text...", UiLayout::default());

        assert!(world.get_component_store::<UiEntity>().is_some());
        assert!(world.get_component_store::<UiTextBox>().is_some());
    }

    #[test]
    fn insert_char_test() {
        let mut textbox = UiTextBox::new();
        insert_char(&mut textbox, 'H');
        insert_char(&mut textbox, 'i');
        assert_eq!(textbox.value, "Hi");
        assert_eq!(textbox.cursor_pos, 2);
    }

    #[test]
    fn delete_char_test() {
        let mut textbox = UiTextBox::new();
        insert_char(&mut textbox, 'H');
        insert_char(&mut textbox, 'i');
        delete_char(&mut textbox);
        assert_eq!(textbox.value, "H");
        assert_eq!(textbox.cursor_pos, 1);
    }

    #[test]
    fn delete_char_empty() {
        let mut textbox = UiTextBox::new();
        delete_char(&mut textbox);
        assert_eq!(textbox.value, "");
        assert_eq!(textbox.cursor_pos, 0);
    }
}
