#[cfg(test)]

use super::*;
use crate::{SimpleWindowApp, WindowHandle, Rect, Menu};


#[allow(dead_code)]
struct MyAppState {
	previous_frame_time: std::time::Instant,
	time: f64
}

impl SimpleWindowApp for MyAppState {
	fn on_paint(&mut self, handle: &WindowHandle, pixel_buffer: &mut [u8], client_rect: &Rect) {
		handle.set_timer(0, 1000);
		
		let now = std::time::Instant::now();
		let dt = now.duration_since(self.previous_frame_time).as_secs_f64();
		self.previous_frame_time = now;
		self.time += dt;
		
		let width = client_rect.width() as u32;
		let height = client_rect.height() as u32;
		
		for y in 0..height {
			for x in 0..width {
				let i = 4 * (y * width + x) as usize;
				pixel_buffer[i] = (127 + (self.time * 60.0) as u32) as u8;
				pixel_buffer[i + 1] = (y * 255 / height + (self.time * 60.0) as u32) as u8;
				pixel_buffer[i + 2] = (x * 255 / width + (self.time * 60.0) as u32) as u8;
			}
		}
	}
	fn on_timer(&mut self, handle: &WindowHandle, _pixel_buffer: &mut [u8], _client_rect: &Rect, _timer_id: usize) {
		handle.request_redraw();
	}
	fn on_command(&mut self, _handle: &WindowHandle, _pixel_buffer: &mut [u8], _client_rect: &Rect, command_id: u16) {
		println!("{}", command_id);
	}
	fn on_mouse_left_down(&mut self, _handle: &WindowHandle, _pixel_buffer: &mut [u8], _client_rect: &Rect, _mouse_x: i16, _mouse_y: i16) {
		println!("Left click");
	}
	fn on_mouse_right_down(&mut self, _handle: &WindowHandle, _pixel_buffer: &mut [u8], _client_rect: &Rect, _mouse_x: i16, _mouse_y: i16) {
		println!("Right click");
	}
	//fn on_resizing(&mut self, _handle: &WindowHandle, client_rect: &mut Rect) {}
	fn on_error(&mut self, _handle: &WindowHandle, _pixel_buffer: &mut [u8], _client_rect: &Rect, error_message: &str) {
		println!("{error_message}");
	}
	fn on_key_down(&mut self, handle: &WindowHandle, _pixel_buffer: &mut [u8], _client_rect: &Rect, key_code: u32) {
		handle.get_menu().unwrap().add_item(7, &key_code.to_string()).unwrap();
		let menu = Menu::new().unwrap();
		menu.add_item(key_code as u16, &format!("hello {key_code}")).unwrap();
		handle.set_menu(menu).unwrap();
		handle.redraw_menu().unwrap();
		
	}
}

#[test]
fn it_works() {
	
	let result = run_window_process("a", 800, 600, "title yes", false, MyAppState { previous_frame_time: std::time::Instant::now(), time: 0.0 });
	
	if let Err(message) = result {
		println!("{message}");
	}
	
}