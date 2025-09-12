use std::time::Instant;

use egui::{pos2, vec2, PointerButton, Pos2, RawInput};
use ggez::{
	input::keyboard::KeyInput, winit::{self, event::MouseButton, keyboard::ModifiersState},
};

/// Contains and manages everything related to the [`egui`] input
///
/// such as the location of the mouse or the pressed keys
pub struct Input {
	dt: Instant,
	pointer_pos: Pos2,
	pub(crate) raw: RawInput,
	pub(crate) scale_factor: f32,
}

impl Default for Input {
	/// scale_factor: 1.0
	fn default() -> Self {
		Self {
			dt: Instant::now(),
			pointer_pos: Default::default(),
			raw: Default::default(),
			scale_factor: 1.0,
		}
	}
}

impl Input {
	pub(crate) fn take(&mut self) -> RawInput {
		self.raw.predicted_dt = self.dt.elapsed().as_secs_f32();
		self.dt = Instant::now();
		self.raw.take()
	}

	pub fn key_down_event(&mut self, ctx: &mut ggez::Context, input: KeyInput, repeat: bool) {
		 let winit::event::KeyEvent {
            // Represents the position of a key independent of the currently active layout.
            //
            // It also uniquely identifies the physical key (i.e. it's mostly synonymous with a scancode).
            // The most prevalent use case for this is games. For example the default keys for the player
            // to move around might be the W, A, S, and D keys on a US layout. The position of these keys
            // is more important than their label, so they should map to Z, Q, S, and D on an "AZERTY"
            // layout. (This value is `KeyCode::KeyW` for the Z key on an AZERTY layout.)
            physical_key,

            // Represents the results of a keymap, i.e. what character a certain key press represents.
            // When telling users "Press Ctrl-F to find", this is where we should
            // look for the "F" key, because they may have a dvorak layout on
            // a qwerty keyboard, and so the logical "F" character may not be located on the physical `KeyCode::KeyF` position.
            logical_key: winit_logical_key,

            text: _,

            state,

            location: _, // e.g. is it on the numpad?
            repeat: _,   // egui will figure this out for us
            ..
        } = &input.event;

        let pressed = *state == winit::event::ElementState::Pressed;

        let physical_key = if let winit::keyboard::PhysicalKey::Code(keycode) = *physical_key {
            key_from_key_code(keycode)
        } else {
            None
        };

        let logical_key = key_from_winit_key(winit_logical_key);

        // "Logical OR physical key" is a fallback mechanism for keyboard layouts without Latin characters: it lets them
        // emit events as if the corresponding keys from the Latin layout were pressed. In this case, clipboard shortcuts
        // are mapped to the physical keys that normally contain C, X, V, etc.
        // See also: https://github.com/emilk/egui/issues/3653
        if let Some(active_key) = logical_key.or(physical_key) {
		// todo: clipboard?
        //     if pressed {
        //         if is_cut_command(self.raw.modifiers, active_key) {
        //             self.raw.events.push(egui::Event::Cut);
        //             return;
        //         } else if is_copy_command(self.raw.modifiers, active_key) {
        //             self.raw.events.push(egui::Event::Copy);
        //             return;
        //         } else if is_paste_command(self.raw.modifiers, active_key) {
        //             if let Some(contents) = self.clipboard.get() {
        //                 let contents = contents.replace("\r\n", "\n");
        //                 if !contents.is_empty() {
        //                     self.raw.events.push(egui::Event::Paste(contents));
        //                 }
        //             }
        //             return;
        //         }
        //     }

            self.raw.events.push(egui::Event::Key {
                key: active_key,
                physical_key,
                pressed,
                repeat: false, // egui will fill this in for us!
                modifiers: self.raw.modifiers,
            });
        }

        // if let Some(text) = text
        //     .as_ref()
        //     .map(|t| t.as_str())
        //     .or_else(|| winit_logical_key.to_text())
        // {
        //     // Make sure there is text, and that it is not control characters
        //     // (e.g. delete is sent as "\u{f728}" on macOS).
        //     if !text.is_empty() && text.chars().all(is_printable_char) {
        //         // On some platforms we get here when the user presses Cmd-C (copy), ctrl-W, etc.
        //         // We need to ignore these characters that are side-effects of commands.
        //         // Also make sure the key is pressed (not released). On Linux, text might
        //         // contain some data even when the key is released.
        //         let is_cmd = self.egui_input.modifiers.ctrl
        //             || self.egui_input.modifiers.command
        //             || self.egui_input.modifiers.mac_cmd;
        //         if pressed && !is_cmd {
        //             self.raw
        //                 .events
        //                 .push(egui::Event::Text(text.to_owned()));
        //         }
        //     }
        // }
	}

	/// It updates egui of what is happening in the input (keys pressed, mouse position, etc), but it doesn't updates
	/// the information of the pressed characters, to update that information you have to
	/// use the function [text_input_event](Input:: text_input_event)
	pub fn update(&mut self, ctx: &ggez::Context) {
		let modifiers = translate_modifier(ctx.keyboard.active_modifiers);
		// /*======================= Keyboard =======================*/
		// for key in &ctx.keyboard.pressed_logical_keys {
		// 	if ctx.keyboard.is_logical_key_just_pressed(key) {
		// 		if let Some(key) = translate_keycode(key) {
		// 			self.raw.events.push(egui::Event::Key {
		// 				key,
		// 				physical_key: None,
		// 				pressed: true,
		// 				repeat: false,
		// 				modifiers: modifier,
		// 			})
		// 		}
		// 	}
		// }

		/*======================= Mouse =======================*/
		let ggez::mint::Point2 { x, y } = ctx.mouse.position();
		self.pointer_pos = pos2(x / self.scale_factor, y / self.scale_factor);
		self.raw
			.events
			.push(egui::Event::PointerMoved(self.pointer_pos));

		for button in [MouseButton::Left, MouseButton::Middle, MouseButton::Right] {
			if ctx.mouse.button_just_pressed(button) {
				self.raw.events.push(egui::Event::PointerButton {
					button: match button {
						MouseButton::Left => PointerButton::Primary,
						MouseButton::Right => PointerButton::Secondary,
						MouseButton::Middle => PointerButton::Middle,
						_ => unreachable!(),
					},
					pos: self.pointer_pos,
					pressed: true,
					modifiers,
				});
			} else if ctx.mouse.button_just_released(button) {
				self.raw.events.push(egui::Event::PointerButton {
					button: match button {
						MouseButton::Left => PointerButton::Primary,
						MouseButton::Right => PointerButton::Secondary,
						MouseButton::Middle => PointerButton::Middle,
						_ => unreachable!(),
					},
					pos: self.pointer_pos,
					pressed: false,
					modifiers,
				});
			}
		}
	}

	/// Set the scale_factor and update the screen_rect
	pub fn set_scale_factor(&mut self, scale_factor: f32, (w, h): (f32, f32)) {
		self.scale_factor = scale_factor;
		self.resize_event(w, h);
	}

	/// Update screen_rect data with window size
	pub fn resize_event(&mut self, w: f32, h: f32) {
		self.raw.screen_rect = Some(egui::Rect::from_min_size(
			Default::default(),
			vec2(w, h) / self.scale_factor,
		));
	}

	/// lets you know the rotation of the mouse wheel
	pub fn mouse_wheel_event(&mut self, x: f32, y: f32, mods: ModifiersState) {
		self.raw.events.push(egui::Event::MouseWheel {
			unit: egui::MouseWheelUnit::Point,
			delta: vec2(x, y),
			modifiers: translate_modifier(mods),
		})
	}

	/// lets know what character is pressed on the keyboard
	pub fn text_input_event(&mut self, ch: char) {
		if is_printable_char(ch) {
			self.raw.events.push(egui::Event::Text(ch.to_string()));
		}
	}
}

#[inline]
fn key_from_winit_key(key: &winit::keyboard::Key) -> Option<egui::Key> {
    match key {
        winit::keyboard::Key::Named(named_key) => key_from_named_key(*named_key),
        winit::keyboard::Key::Character(str) => egui::Key::from_name(str.as_str()),
        winit::keyboard::Key::Unidentified(_) | winit::keyboard::Key::Dead(_) => None,
    }
}

#[inline]
fn key_from_named_key(named_key: winit::keyboard::NamedKey) -> Option<egui::Key> {
    use egui::Key;
    use winit::keyboard::NamedKey;

    Some(match named_key {
        NamedKey::Enter => Key::Enter,
        NamedKey::Tab => Key::Tab,
        NamedKey::ArrowDown => Key::ArrowDown,
        NamedKey::ArrowLeft => Key::ArrowLeft,
        NamedKey::ArrowRight => Key::ArrowRight,
        NamedKey::ArrowUp => Key::ArrowUp,
        NamedKey::End => Key::End,
        NamedKey::Home => Key::Home,
        NamedKey::PageDown => Key::PageDown,
        NamedKey::PageUp => Key::PageUp,
        NamedKey::Backspace => Key::Backspace,
        NamedKey::Delete => Key::Delete,
        NamedKey::Insert => Key::Insert,
        NamedKey::Escape => Key::Escape,
        NamedKey::Cut => Key::Cut,
        NamedKey::Copy => Key::Copy,
        NamedKey::Paste => Key::Paste,

        NamedKey::Space => Key::Space,

        NamedKey::F1 => Key::F1,
        NamedKey::F2 => Key::F2,
        NamedKey::F3 => Key::F3,
        NamedKey::F4 => Key::F4,
        NamedKey::F5 => Key::F5,
        NamedKey::F6 => Key::F6,
        NamedKey::F7 => Key::F7,
        NamedKey::F8 => Key::F8,
        NamedKey::F9 => Key::F9,
        NamedKey::F10 => Key::F10,
        NamedKey::F11 => Key::F11,
        NamedKey::F12 => Key::F12,
        NamedKey::F13 => Key::F13,
        NamedKey::F14 => Key::F14,
        NamedKey::F15 => Key::F15,
        NamedKey::F16 => Key::F16,
        NamedKey::F17 => Key::F17,
        NamedKey::F18 => Key::F18,
        NamedKey::F19 => Key::F19,
        NamedKey::F20 => Key::F20,
        NamedKey::F21 => Key::F21,
        NamedKey::F22 => Key::F22,
        NamedKey::F23 => Key::F23,
        NamedKey::F24 => Key::F24,
        NamedKey::F25 => Key::F25,
        NamedKey::F26 => Key::F26,
        NamedKey::F27 => Key::F27,
        NamedKey::F28 => Key::F28,
        NamedKey::F29 => Key::F29,
        NamedKey::F30 => Key::F30,
        NamedKey::F31 => Key::F31,
        NamedKey::F32 => Key::F32,
        NamedKey::F33 => Key::F33,
        NamedKey::F34 => Key::F34,
        NamedKey::F35 => Key::F35,

        NamedKey::BrowserBack => Key::BrowserBack,
        _ => {
            return None;
        }
    })
}

#[inline]
fn key_from_key_code(key: winit::keyboard::KeyCode) -> Option<egui::Key> {
    use egui::Key;
    use winit::keyboard::KeyCode;

    Some(match key {
        KeyCode::ArrowDown => Key::ArrowDown,
        KeyCode::ArrowLeft => Key::ArrowLeft,
        KeyCode::ArrowRight => Key::ArrowRight,
        KeyCode::ArrowUp => Key::ArrowUp,

        KeyCode::Escape => Key::Escape,
        KeyCode::Tab => Key::Tab,
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Enter | KeyCode::NumpadEnter => Key::Enter,

        KeyCode::Insert => Key::Insert,
        KeyCode::Delete => Key::Delete,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,

        // Punctuation
        KeyCode::Space => Key::Space,
        KeyCode::Comma => Key::Comma,
        KeyCode::Period => Key::Period,
        // KeyCode::Colon => Key::Colon, // NOTE: there is no physical colon key on an american keyboard
        KeyCode::Semicolon => Key::Semicolon,
        KeyCode::Backslash => Key::Backslash,
        KeyCode::Slash | KeyCode::NumpadDivide => Key::Slash,
        KeyCode::BracketLeft => Key::OpenBracket,
        KeyCode::BracketRight => Key::CloseBracket,
        KeyCode::Backquote => Key::Backtick,
        KeyCode::Quote => Key::Quote,

        KeyCode::Cut => Key::Cut,
        KeyCode::Copy => Key::Copy,
        KeyCode::Paste => Key::Paste,
        KeyCode::Minus | KeyCode::NumpadSubtract => Key::Minus,
        KeyCode::NumpadAdd => Key::Plus,
        KeyCode::Equal => Key::Equals,

        KeyCode::Digit0 | KeyCode::Numpad0 => Key::Num0,
        KeyCode::Digit1 | KeyCode::Numpad1 => Key::Num1,
        KeyCode::Digit2 | KeyCode::Numpad2 => Key::Num2,
        KeyCode::Digit3 | KeyCode::Numpad3 => Key::Num3,
        KeyCode::Digit4 | KeyCode::Numpad4 => Key::Num4,
        KeyCode::Digit5 | KeyCode::Numpad5 => Key::Num5,
        KeyCode::Digit6 | KeyCode::Numpad6 => Key::Num6,
        KeyCode::Digit7 | KeyCode::Numpad7 => Key::Num7,
        KeyCode::Digit8 | KeyCode::Numpad8 => Key::Num8,
        KeyCode::Digit9 | KeyCode::Numpad9 => Key::Num9,

        KeyCode::KeyA => Key::A,
        KeyCode::KeyB => Key::B,
        KeyCode::KeyC => Key::C,
        KeyCode::KeyD => Key::D,
        KeyCode::KeyE => Key::E,
        KeyCode::KeyF => Key::F,
        KeyCode::KeyG => Key::G,
        KeyCode::KeyH => Key::H,
        KeyCode::KeyI => Key::I,
        KeyCode::KeyJ => Key::J,
        KeyCode::KeyK => Key::K,
        KeyCode::KeyL => Key::L,
        KeyCode::KeyM => Key::M,
        KeyCode::KeyN => Key::N,
        KeyCode::KeyO => Key::O,
        KeyCode::KeyP => Key::P,
        KeyCode::KeyQ => Key::Q,
        KeyCode::KeyR => Key::R,
        KeyCode::KeyS => Key::S,
        KeyCode::KeyT => Key::T,
        KeyCode::KeyU => Key::U,
        KeyCode::KeyV => Key::V,
        KeyCode::KeyW => Key::W,
        KeyCode::KeyX => Key::X,
        KeyCode::KeyY => Key::Y,
        KeyCode::KeyZ => Key::Z,

        KeyCode::F1 => Key::F1,
        KeyCode::F2 => Key::F2,
        KeyCode::F3 => Key::F3,
        KeyCode::F4 => Key::F4,
        KeyCode::F5 => Key::F5,
        KeyCode::F6 => Key::F6,
        KeyCode::F7 => Key::F7,
        KeyCode::F8 => Key::F8,
        KeyCode::F9 => Key::F9,
        KeyCode::F10 => Key::F10,
        KeyCode::F11 => Key::F11,
        KeyCode::F12 => Key::F12,
        KeyCode::F13 => Key::F13,
        KeyCode::F14 => Key::F14,
        KeyCode::F15 => Key::F15,
        KeyCode::F16 => Key::F16,
        KeyCode::F17 => Key::F17,
        KeyCode::F18 => Key::F18,
        KeyCode::F19 => Key::F19,
        KeyCode::F20 => Key::F20,
        KeyCode::F21 => Key::F21,
        KeyCode::F22 => Key::F22,
        KeyCode::F23 => Key::F23,
        KeyCode::F24 => Key::F24,
        KeyCode::F25 => Key::F25,
        KeyCode::F26 => Key::F26,
        KeyCode::F27 => Key::F27,
        KeyCode::F28 => Key::F28,
        KeyCode::F29 => Key::F29,
        KeyCode::F30 => Key::F30,
        KeyCode::F31 => Key::F31,
        KeyCode::F32 => Key::F32,
        KeyCode::F33 => Key::F33,
        KeyCode::F34 => Key::F34,
        KeyCode::F35 => Key::F35,

        _ => {
            return None;
        }
    })
}

#[inline]
fn translate_modifier(keymods: ModifiersState) -> egui::Modifiers {
	egui::Modifiers {
		alt: keymods.intersects(ModifiersState::ALT),
		ctrl: keymods.intersects(ModifiersState::CONTROL),
		shift: keymods.intersects(ModifiersState::SHIFT),

		#[cfg(not(target_os = "macos"))]
		mac_cmd: false,
		#[cfg(not(target_os = "macos"))]
		command: keymods.intersects(ModifiersState::CONTROL),

		#[cfg(target_os = "macos")]
		mac_cmd: keymods.intersects(ModifiersState::SUPER),
		#[cfg(target_os = "macos")]
		command: keymods.intersects(ModifiersState::SUPER),
	}
}


/// Winit sends special keys (backspace, delete, F1, …) as characters.
/// Ignore those.
/// We also ignore '\r', '\n', '\t'.
/// Newlines are handled by the `Key::Enter` event.
fn is_printable_char(chr: char) -> bool {
    let is_in_private_use_area = ('\u{e000}'..='\u{f8ff}').contains(&chr)
        || ('\u{f0000}'..='\u{ffffd}').contains(&chr)
        || ('\u{100000}'..='\u{10fffd}').contains(&chr);

    !is_in_private_use_area && !chr.is_ascii_control()
}

fn is_cut_command(modifiers: egui::Modifiers, keycode: egui::Key) -> bool {
    keycode == egui::Key::Cut
        || (modifiers.command && keycode == egui::Key::X)
        || (cfg!(target_os = "windows") && modifiers.shift && keycode == egui::Key::Delete)
}

fn is_copy_command(modifiers: egui::Modifiers, keycode: egui::Key) -> bool {
    keycode == egui::Key::Copy
        || (modifiers.command && keycode == egui::Key::C)
        || (cfg!(target_os = "windows") && modifiers.ctrl && keycode == egui::Key::Insert)
}

fn is_paste_command(modifiers: egui::Modifiers, keycode: egui::Key) -> bool {
    keycode == egui::Key::Paste
        || (modifiers.command && keycode == egui::Key::V)
        || (cfg!(target_os = "windows") && modifiers.shift && keycode == egui::Key::Insert)
}

fn translate_mouse_button(button: winit::event::MouseButton) -> Option<egui::PointerButton> {
    match button {
        winit::event::MouseButton::Left => Some(egui::PointerButton::Primary),
        winit::event::MouseButton::Right => Some(egui::PointerButton::Secondary),
        winit::event::MouseButton::Middle => Some(egui::PointerButton::Middle),
        winit::event::MouseButton::Back => Some(egui::PointerButton::Extra1),
        winit::event::MouseButton::Forward => Some(egui::PointerButton::Extra2),
        winit::event::MouseButton::Other(_) => None,
    }
}
