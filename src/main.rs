#![windows_subsystem = "windows"]
#[macro_use]
extern crate nuklear;
extern crate nuklear_backend_gdi;
extern crate winapi;
extern crate regex;
#[macro_use]
extern crate lazy_static;

use nuklear::{Color, Context, Flags};
use nuklear as nk;
use nuklear_backend_gdi::*;

use regex::*;

mod utils;

fn main() {
	let mut allo = nk::Allocator::new_vec();
	let (mut dr, mut ctx, font) = bundle(
		"idsr", 400, 600, "Segoe UI", 16, &mut allo,
	);
	let clear_color: Color = utils::color_from_hex(0xc47fef);

	let mut buf: [u8; 1000] = [0; 1000];

	let mut state = State {
		input_buf: &mut buf[..],
		input_buf_len: 0,
		last_buf_len: 0,
		avgs: None,
		skip_corrected: false,
	};

	loop {
		if !dr.process_events(&mut ctx) {
			break;
		}

		ctx.style_set_font(dr.font_by_id(font).unwrap());

		layout(&mut ctx, &mut dr, &mut state);
		dr.render(&mut ctx, clear_color);
	}
}

struct State<'a> {
	input_buf: &'a mut [u8],
	input_buf_len: i32,
	last_buf_len: i32,
	avgs: Option<Vec<f64>>,
	skip_corrected: bool,
}

fn layout(ctx: &mut Context, dr: &mut Drawer, state: &mut State) {
	let (w, h) = utils::get_window_size(dr.window().unwrap());

	if !ctx.begin(
		nk_string!("idsr"),
		nk::Rect { x: 0.0f32, y: 0.0f32, w: w as f32, h: h as f32 },
		0 as Flags,
	) {
		panic!("ctx.begin returned false");
	}

	ctx.layout_row_dynamic(220f32, 1);
	ctx.edit_string(nk::EditType::NK_EDIT_BOX as Flags,
					&mut state.input_buf, &mut state.input_buf_len, None);

	ctx.layout_row_dynamic(20f32, 1);
	if ctx.checkbox_text("Nie licz poprawek", &mut !state.skip_corrected) {
		state.skip_corrected = !state.skip_corrected;
		state.recalc();
	}

	if state.input_buf_len != state.last_buf_len {
		state.recalc();
		state.last_buf_len = state.input_buf_len;
	}

	if let Some(avgs) = &state.avgs {
		let mut sum = 0.0;
		for (idx, avg) in avgs.iter().enumerate() {
			ctx.layout_row_dynamic(22f32, 1);
			ctx.text(format!("{}: {:.2}", idx + 1, avg).as_str(),
					 nk::TextAlignment::NK_TEXT_CENTERED as Flags);
			sum += avg;
		}
		ctx.layout_row_dynamic(22f32, 1);
		ctx.text(format!("Średnia wszystkich: {:.2}", sum / avgs.len() as f64).as_str(),
				 nk::TextAlignment::NK_TEXT_CENTERED as Flags);
	}

	ctx.end();
}

impl<'a> State<'a> {
	fn recalc(self: &mut Self) {
		let text: &str = unsafe {
			std::str::from_utf8_unchecked(
				&self.input_buf[0..(self.input_buf_len as usize)])
		};

		let new_avgs = text.lines().filter_map(|x| calc_marks_average(x, self.skip_corrected))
			.collect::<Vec<f64>>();
		self.avgs = if new_avgs.len() == 0 { None } else { Some(new_avgs) };
	}
}

fn calc_marks_average(input: &str, skip_corrected: bool) -> Option<f64> {
	let mut nums: Vec<f64> = Vec::new();

	lazy_static! {
		static ref RE: Regex = Regex::new(r"(\d+)([+,-])?/?(\d+)?([+,-])?").unwrap();
	}

	for capture in RE.captures_iter(input) {
		let num = capture.get(1);
		let sign = capture.get(2);
		let num_corrected = capture.get(3);
		let sign_corrected = capture.get(4);

		let num = match num {
			Some(num) => {
				let mut num = match num.as_str().parse::<i32>() {
					Ok(num) => num as f64,
					Err(_) => continue,
				};
				if let Some(sign) = sign {
					match sign.as_str() {
						"+" => num += 0.5,
						"-" => num -= 0.25,
						_ => (),
					}
				}
				num
			}
			None => continue,
		};

		if num_corrected.is_none() {
			nums.push(num);
		} else {
			let mut num_corrected = match num_corrected.unwrap().as_str().parse::<i32>() {
				Ok(num_corrected) => num_corrected as f64,
				Err(_) => continue,
			};
			if let Some(sign_corrected) = sign_corrected {
				match sign_corrected.as_str() {
					"+" => num_corrected += 0.5,
					"-" => num_corrected -= 0.25,
					_ => (),
				}
			}

			if skip_corrected {
				nums.push(num_corrected);
			} else {
				nums.push((num + num_corrected) as f64 / 2.0);
			}
		}
	}

	if nums.len() == 0 { None } else { Some(nums.iter().sum::<f64>() / nums.len() as f64) }
}
