use nuklear::Color;

pub fn color_from_hex(rgb: u32) -> Color {
	Color {
		r: ((rgb & 0xff0000) >> 2 * 8) as u8,
		g: ((rgb & 0x00ff00) >> 1 * 8) as u8,
		b: (rgb & 0x0000ff) as u8,
		a: 100u8,
	}
}

pub fn get_window_size(hwnd: winapi::shared::windef::HWND) -> (i32, i32) {
	unsafe {
		use winapi::shared::windef::*;
		let mut rect = RECT{
			left: 0,
			top: 0,
			right: 0,
			bottom: 0,
		};
		winapi::um::winuser::GetClientRect(hwnd, &mut rect as *mut RECT);
		return (rect.right - rect.left, rect.bottom - rect.top);
	}
}
