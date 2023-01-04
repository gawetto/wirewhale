use crate::App;
use crossterm::event::KeyCode;

pub fn allmode_input(app: &mut App, keycode: KeyCode) {
    if let KeyCode::Tab = keycode {
        app.next_forcus();
    }
    if let KeyCode::Esc = keycode {
        app.exit();
    }
}
pub fn viewmode_input(_app: &mut App, _keycode: KeyCode) {}
pub fn listmode_input(app: &mut App, keycode: KeyCode) {
    match keycode {
        KeyCode::Left => {
            app.unselect();
        }
        KeyCode::Down => {
            app.next();
        }
        KeyCode::Up => {
            app.previous();
        }
        KeyCode::Right => {
            app.to_view();
        }
        _ => {}
    }
}
pub fn filtermode_input(app: &mut App, keycode: KeyCode) {
    if let KeyCode::Backspace = keycode {
        app.delete_filter_char();
        return;
    }
    let key_char = if let KeyCode::Char(x) = keycode {
        x
    } else {
        return;
    };
    if !key_char.is_ascii() {
        return;
    }
    app.add_filter_str(&key_char.to_string());
}
