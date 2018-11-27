#![windows_subsystem = "windows"]
#[macro_use]
extern crate nuklear;
extern crate nuklear_backend_gdi;
extern crate winapi;
extern crate regex;

use nuklear::{Color, Context, Flags};
use nuklear as nk;
use nuklear_backend_gdi::*;

use regex::*;

mod utils;

fn main() {
	let mut allo = nk::Allocator::new_vec();
	let (mut dr, mut ctx, font) = bundle(
		"idsr", 300, 100, "Segoe UI", 16, &mut allo,
	);
	let clear_color: Color = utils::color_from_hex(0xc47fef);

	let mut buf: [u8; 1000] = [0; 1000];

	let re = Regex::new(r"(\d+)([+,-])?/?(\d+)?([+,-])?").unwrap();

	let mut state = State {
		input_buf: &mut buf[..],
		input_buf_len: 0,
		last_buf_len: 0,
		avg: None,
		re,
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
	avg: Option<f64>,
	re: Regex,
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

	ctx.layout_row_dynamic(32f32, 1);
	ctx.edit_string(nk::EditType::NK_EDIT_FIELD as Flags,
					&mut state.input_buf, &mut state.input_buf_len, None);

	if state.input_buf_len != state.last_buf_len {
		let text: &str = unsafe {
			std::str::from_utf8_unchecked(
				&state.input_buf[0..(state.input_buf_len as usize)])
		};

		state.avg = calc_marks_average(&state.re, text);
		state.last_buf_len = state.input_buf_len;
	}

	if let Some(avg) = state.avg {
		ctx.layout_row_dynamic(32f32, 1);
		ctx.text(format!("Średnia: {:.2}", avg).as_str(),
				 nk::TextAlignment::NK_TEXT_CENTERED as Flags);
	}

	ctx.end();
}

fn calc_marks_average(re: &Regex, input: &str) -> Option<f64> {
	let tokens = input.split(" ").collect::<Vec<&str>>();
	let mut nums: Vec<f64> = Vec::with_capacity(tokens.len());

	for capture in re.captures_iter(input) {
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

			nums.push((num + num_corrected) as f64 / 2.0);
		}
	}

	if nums.len() == 0 { None } else { Some(nums.iter().sum::<f64>() / nums.len() as f64) }
}
