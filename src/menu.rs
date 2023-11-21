use macroquad::prelude::*;
use crate::ResourceBundle;

pub struct MenuState {
    selected_index: u32,
}

impl MenuState {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
        }
    }
}

pub const MENU_FONT_SIZE: f32 = 32.0;

#[derive(Copy, Clone)]
pub enum MenuOption {
    LocalGame,
    HostMultiplayer,
    JoinMultiplayer,
    Quit
}

pub fn render_menu(state: &mut MenuState, resources: &ResourceBundle) -> Option<MenuOption> {
    let (screen_width, screen_height) = (screen_width(), screen_height());
    let (mouse_x, mouse_y) = mouse_position();
    let ResourceBundle {logo, menu_item_bg, ..} = resources;
    let logo_x = (screen_width - logo.width()) / 2.0;
    // if we center it completely we have no space for the menu, so we add an offset to bump it up
    // a bit
    let logo_y = (screen_height - logo.height()) / 2.0 - 100.0;
    draw_texture(logo, logo_x, logo_y, WHITE);

    let menu_items = vec![
        (MenuOption::LocalGame, "New Local Game"),
        (MenuOption::HostMultiplayer, "Host Multiplayer Game"),
        (MenuOption::JoinMultiplayer, "Join Multiplayer Game"),
        (MenuOption::Quit, "Quit"),
    ];

    if is_key_pressed(KeyCode::Down) {
        state.selected_index = (state.selected_index + 1).clamp(0, menu_items.len() as u32 -1);
    }

    if is_key_pressed(KeyCode::Up) {
        state.selected_index = (state.selected_index as i32 - 1).clamp(0, menu_items.len() as i32 -1) as u32;
    }

    if is_key_pressed(KeyCode::Enter) {
        return Some(menu_items[state.selected_index as usize].0);
    }

    for idx in 0..menu_items.len() {
        let (item_option, item_label) = menu_items[idx];
        let item_x = (screen_width - menu_item_bg.width()) / 2.0;
        let item_y = logo_y + logo.height() + (idx as f32 * menu_item_bg.height());
        let item_text_size = measure_text(item_label, None, MENU_FONT_SIZE as u16, 1.0);
        let texture_color = if state.selected_index as usize == idx {
            WHITE
        } else {
            GRAY
        };
        draw_texture(menu_item_bg, item_x, item_y, texture_color);
        draw_text(item_label, item_x + ((menu_item_bg.width() - item_text_size.width) / 2.0), item_y + (menu_item_bg.height() / 2.0), MENU_FONT_SIZE, BEIGE);
        if mouse_x > item_x && mouse_x < item_x + menu_item_bg.width() &&
            mouse_y > item_y && mouse_y < item_y + menu_item_bg.height()
        {
            state.selected_index = idx as u32;
            if is_mouse_button_pressed(MouseButton::Left) {
                return Some(item_option);
            }
        }
    }
    None
}
