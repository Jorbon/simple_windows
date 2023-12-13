#[cfg(test)]

use super::*;
use crate::{SimpleWindowApp, Rect};


struct MyAppState {
	time: f64
}

impl SimpleWindowApp for MyAppState {
	fn on_paint(&mut self, pixel_buffer: &mut [u8], client_rect: &Rect) {
		
	}
	fn on_command(&mut self, pixel_buffer: &mut [u8], client_rect: &Rect, command_id: u16) {
		println!("{}", command_id);
	}
	fn on_mouse_left_down(&mut self, pixel_buffer: &mut [u8], client_rect: &Rect, mouse_x: i16, mouse_y: i16) {
		println!("Left click");
	}
	fn on_mouse_right_down(&mut self, pixel_buffer: &mut [u8], client_rect: &Rect, mouse_x: i16, mouse_y: i16) {
		println!("Right click");
	}
}

#[test]
fn it_works() {
	
	run_window_process("a", 800, 600, "title yes", false, MyAppState { time: 0.0 }).unwrap();
	
}