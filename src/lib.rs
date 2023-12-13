use std::{os::{raw::c_void, windows::ffi::OsStrExt}, ffi::OsStr};
use windows::{core::{Result, PCWSTR, Error, HSTRING}, Win32::{Foundation::{HWND, RECT, LPARAM, LRESULT, WPARAM}, UI::WindowsAndMessaging::{self, CS_HREDRAW, CS_VREDRAW, WS_EX_TOPMOST, WS_OVERLAPPEDWINDOW, HICON, RegisterClassW, LoadCursorW, WNDCLASSW, IDC_ARROW, DefWindowProcW, GetWindowLongPtrW, SetWindowLongPtrW, WM_NCCREATE, CREATESTRUCTW, GWLP_USERDATA, TranslateMessage, DispatchMessageW, GetMessageW, PostQuitMessage, MSG, CreateWindowExW, CW_USEDEFAULT, SW_SHOW, ShowWindow, GetClientRect, WINDOW_EX_STYLE, CreateMenu, MF_STRING, AppendMenuW, SetMenu, MF_POPUP, AdjustWindowRectEx, SetTimer, KillTimer}, System::{WinRT::{DispatcherQueueOptions, RoInitialize, DQTYPE_THREAD_CURRENT, DQTAT_COM_NONE, RO_INIT_SINGLETHREADED, CreateDispatcherQueueController}, LibraryLoader::GetModuleHandleW}, Graphics::Gdi::{PAINTSTRUCT, BeginPaint, EndPaint, SelectObject, DeleteObject, CreateCompatibleDC, BITMAPINFO, BITMAPINFOHEADER, RGBQUAD, BI_RGB, CreateDIBSection, DIB_RGB_COLORS, BitBlt, SRCCOPY, DeleteDC, HBRUSH, HBITMAP, GetDC, ReleaseDC, InvalidateRect}}, Foundation::AsyncActionCompletedHandler};

mod tests;


// Todo:
// port timer as a settimeout function
// replace unwraps with early error
// fix up size and sizing handlers
// menu builder
// more handles


pub type Rect = RECT;
trait SimpleWindowsRect {
	fn width(&self) -> i32;
	fn height(&self) -> i32;
}
impl SimpleWindowsRect for Rect {
	fn width(&self) -> i32 {
		self.right - self.left
	}
	fn height(&self) -> i32 {
		self.bottom - self.top
	}
}


pub trait SimpleWindowApp {
	#[allow(unused_variables)]
	fn on_paint(&mut self, pixel_buffer: &mut [u8], client_rect: &Rect) {}
	#[allow(unused_variables)]
	fn on_command(&mut self, pixel_buffer: &mut [u8], client_rect: &Rect, command_id: u16) {}
	#[allow(unused_variables)]
	fn on_resize(&mut self, pixel_buffer: &mut [u8], client_rect: &Rect) {}
	#[allow(unused_variables)]
	fn on_resizing(&mut self, pixel_buffer: &mut [u8], client_rect: &mut Rect) {
		self.on_resize(pixel_buffer, client_rect);
	}
	#[allow(unused_variables)]
	fn on_mouse_move(&mut self, pixel_buffer: &mut [u8], client_rect: &Rect, mouse_x: i16, mouse_y: i16) {}
	#[allow(unused_variables)]
	fn on_mouse_left_down(&mut self, pixel_buffer: &mut [u8], client_rect: &Rect, mouse_x: i16, mouse_y: i16) {}
	#[allow(unused_variables)]
	fn on_mouse_right_down(&mut self, pixel_buffer: &mut [u8], client_rect: &Rect, mouse_x: i16, mouse_y: i16) {}
	#[allow(unused_variables)]
	fn on_exit(&mut self) {}
}


struct App {
	window_handle: HWND,
	client_rect: Rect,
	bitmap: Option<HBITMAP>,
	pixel_buffer: std::mem::ManuallyDrop<Box<[u8]>>,
	user_state: Box<dyn SimpleWindowApp>
}

unsafe extern "system" fn wnd_proc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
	if message == WM_NCCREATE {
		let cs = lparam.0 as *const CREATESTRUCTW;
		let app_ptr = (*cs).lpCreateParams as *mut c_void;
		set_window_handle(app_ptr, window);

		SetWindowLongPtrW(window, GWLP_USERDATA, app_ptr as isize);
	} else {
		let app_ptr = GetWindowLongPtrW(window, GWLP_USERDATA) as *mut c_void;
		if !app_ptr.is_null() {
			return handle_message(app_ptr, message, wparam, lparam);
		}
	}
	DefWindowProcW(window, message, wparam, lparam)
}

fn set_window_handle(app_ptr: *mut c_void, handle: HWND) {
	unsafe { (*(app_ptr as *mut App)).window_handle = handle; }
}


fn handle_message(app_ptr: *mut c_void, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
	let app = unsafe { &mut *(app_ptr as *mut App) };
	
	match message {
		WindowsAndMessaging::WM_TIMER => {
			match wparam.0 {
				1 => {
					unsafe {
						KillTimer(app.window_handle, 1).unwrap();
						InvalidateRect(app.window_handle, None, false);
					}
				}
				_ => {}
			}
		}
		WindowsAndMessaging::WM_SIZE | WindowsAndMessaging::WM_SIZING => {
			let mut rect = RECT::default();
			unsafe { GetClientRect(app.window_handle, &mut rect) }.unwrap();
			
			if rect.width() > 0 && rect.height() > 0 {
				app.client_rect = rect;
				
				let buffer_size = (app.client_rect.width() * app.client_rect.height() * 4) as usize;
				
				if let Some(bitmap) = app.bitmap {
					unsafe { DeleteObject(bitmap) };
				}
				
				let bmi = BITMAPINFO {
					bmiHeader: BITMAPINFOHEADER {
						biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
						biWidth: app.client_rect.width() as i32,
						biHeight: -(app.client_rect.height() as i32),
						biPlanes: 1,
						biBitCount: 32,
						biCompression: BI_RGB.0,
						biSizeImage: 0,
						biXPelsPerMeter: 0,
						biYPelsPerMeter: 0,
						biClrUsed: 0,
						biClrImportant: 0
					},
					bmiColors: [RGBQUAD::default(); 1]
				};
				
				let mut pixel_data_pointer: *mut c_void = std::ptr::null_mut();
				unsafe {
					let dc = GetDC(app.window_handle);
					app.bitmap = Some(CreateDIBSection(dc, &bmi, DIB_RGB_COLORS, &mut pixel_data_pointer, None, 0).unwrap());
					ReleaseDC(app.window_handle, dc);
					app.pixel_buffer = std::mem::ManuallyDrop::new(Vec::from_raw_parts(pixel_data_pointer as *mut u8, buffer_size, buffer_size).into_boxed_slice());
				}
			}
		}
		WindowsAndMessaging::WM_MOUSEMOVE => {
			app.user_state.on_mouse_move(&mut app.pixel_buffer, &app.client_rect, lparam.0 as i16, (lparam.0 >> 16) as i16);
		}
		WindowsAndMessaging::WM_LBUTTONDOWN => {
			app.user_state.on_mouse_left_down(&mut app.pixel_buffer, &app.client_rect, lparam.0 as i16, (lparam.0 >> 16) as i16);
		}
		WindowsAndMessaging::WM_RBUTTONDOWN => {
			app.user_state.on_mouse_right_down(&mut app.pixel_buffer, &app.client_rect, lparam.0 as i16, (lparam.0 >> 16) as i16);
		}
		WindowsAndMessaging::WM_COMMAND => {
			app.user_state.on_command(&mut app.pixel_buffer, &app.client_rect, wparam.0 as u16);
		}
		WindowsAndMessaging::WM_PAINT => {
			unsafe { SetTimer(app.window_handle, 1, 1000 / 60, None) };
			
			app.user_state.on_paint(&mut app.pixel_buffer, &app.client_rect);
			
			unsafe {
				let mut ps = PAINTSTRUCT::default();
				let hdc = BeginPaint(app.window_handle, &mut ps);
				
				if let Some(bitmap) = app.bitmap {
					let memory_dc = CreateCompatibleDC(hdc);
					SelectObject(memory_dc, bitmap);
					BitBlt(hdc, 0, 0, app.client_rect.width(), app.client_rect.height(), memory_dc, 0, 0, SRCCOPY).unwrap();
					DeleteDC(memory_dc);
				}
				
				EndPaint(app.window_handle, &mut ps);
			}
		}
		WindowsAndMessaging::WM_DESTROY => {
			app.user_state.on_exit();
			
			unsafe { PostQuitMessage(0) };
			return LRESULT(0);
		}
		_ => {}
	}
	unsafe { DefWindowProcW(app.window_handle, message, wparam, lparam) }
}




pub fn run_window_process(window_id: &str, window_width: u32, window_height: u32, window_title: &str, always_on_top: bool, app_state: impl SimpleWindowApp + 'static) -> core::result::Result<i32, String> {
	
	unsafe { RoInitialize(RO_INIT_SINGLETHREADED) }.unwrap();
	let options = DispatcherQueueOptions {
		dwSize: std::mem::size_of::<DispatcherQueueOptions>() as u32,
		threadType: DQTYPE_THREAD_CURRENT,
		apartmentType: DQTAT_COM_NONE
	};
	
	let controller = unsafe { CreateDispatcherQueueController(options) }.unwrap();
	let instance = unsafe { GetModuleHandleW(None) }.unwrap();
	
	let window_id_osstr: Vec<u16> = OsStr::new(window_id).encode_wide().chain(Some(0)).collect();
	
	let class = WNDCLASSW {
		hCursor: unsafe { LoadCursorW(None, IDC_ARROW) }.ok().unwrap(),
		hInstance: instance.into(),
		lpszClassName: PCWSTR(window_id_osstr.as_ptr()),
		lpfnWndProc: Some(wnd_proc),
		style: CS_HREDRAW | CS_VREDRAW,
		cbClsExtra: 0,
		cbWndExtra: 0,
		hIcon: HICON(0),
		hbrBackground: HBRUSH(0),
		lpszMenuName: PCWSTR(std::ptr::null())
	};
	assert_ne!(unsafe { RegisterClassW(&class) }, 0);
	
	let mut window_ex_style = WINDOW_EX_STYLE(0);
	if always_on_top { window_ex_style |= WS_EX_TOPMOST; }
	let window_style = WS_OVERLAPPEDWINDOW;
	
	let mut adjust_rect = RECT {
		left: 0,
		top: 0,
		right: window_width as i32,
		bottom: window_height as i32,
	};
	unsafe { AdjustWindowRectEx(&mut adjust_rect, window_style, false, window_ex_style) }.unwrap();
	
	let mut app = App {
		window_handle: HWND(0),
		client_rect: Rect::default(),
		bitmap: None,
		pixel_buffer: core::mem::ManuallyDrop::default(),
		user_state: Box::new(app_state)
	};
	
	let window = unsafe { CreateWindowExW(
		window_ex_style,
		PCWSTR(window_id_osstr.as_ptr()),
		&HSTRING::from(window_title),
		window_style,
		CW_USEDEFAULT,
		CW_USEDEFAULT,
		adjust_rect.right - adjust_rect.left,
		adjust_rect.bottom - adjust_rect.top,
		None,
		None,
		instance,
		Some(&mut app as *mut _ as _)
	) };
	
	if window.0 == 0 {
		panic!("{}", Error::from_win32());
	}
	
	/*unsafe {
		let menu = CreateMenu().unwrap();
		let submenu = WindowsAndMessaging::CreatePopupMenu().unwrap();
		AppendMenuW(submenu, MF_STRING, 1, w!("yes")).unwrap();
		AppendMenuW(submenu, MF_STRING, 2, w!("no")).unwrap();
		AppendMenuW(menu, MF_STRING | MF_POPUP, submenu.0 as usize, w!("Gaming")).unwrap();
		
		
		let submenu2 = WindowsAndMessaging::CreatePopupMenu().unwrap();
		AppendMenuW(submenu2, MF_STRING, 3, w!("maybe")).unwrap();
		AppendMenuW(submenu2, MF_STRING, 4, w!("ok")).unwrap();
		AppendMenuW(menu, MF_STRING | MF_POPUP, submenu2.0 as usize, w!("Not gaming")).unwrap();
		
		SetMenu(app.window_handle, menu).unwrap();
	}*/
	
	unsafe { ShowWindow(window, SW_SHOW) };
	
	
	
	let mut message = MSG::default();
	unsafe {
		while GetMessageW(&mut message, None, 0, 0).into() {
			TranslateMessage(&message);
			DispatchMessageW(&message);
		}
	}
	
	
	
	let async_action = controller.ShutdownQueueAsync().unwrap();
	async_action.SetCompleted(&AsyncActionCompletedHandler::new(
		move |_, _| -> Result<()> {
			unsafe { PostQuitMessage(message.wParam.0 as i32) };
			Ok(())
		}
	)).unwrap();
	
	let mut message = MSG::default();
	unsafe {
		while GetMessageW(&mut message, None, 0, 0).into() {
			TranslateMessage(&message);
			DispatchMessageW(&message);
		}
	}
	
	Ok(message.wParam.0 as i32)
}