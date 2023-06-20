/*
 * MIT License
 *
 * Copyright (c) 2023 Ivan Bushchik
 *
 * Permission is hereby granted, free of charge, to any person obtaining a
 * copy of this software and associated documentation files (the "Software"),
 * to deal in the Software without restriction, including without limitation
 * the rights to use, copy, modify, merge, publish, distribute, sublicense,
 * and/or sell copies of the Software, and to permit persons to whom the
 * Software is furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL
 * THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
 * FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
 * DEALINGS IN THE SOFTWARE.
 */

#[macro_use]
extern crate rocket;

use std::cell::OnceCell;
use std::path::PathBuf;

use rocket::http::Status;
use rocket::response::content::RawText;
use rocket::response::Redirect;
use serde::Deserialize;

const INDEX_REDIRECT: &'static str = "https://ivabus.dev";
const _ALIAS: &'static str = include_str!("../alias.json");
static mut ALIAS: OnceCell<Vec<Alias>> = OnceCell::new();

#[derive(Deserialize, Clone)]
struct Alias {
	uri: String,
	alias: String,
	is_url: Option<bool>,
}

#[get("/<page>")]
async fn get_page(page: String) -> Result<RawText<String>, Redirect> {
	let mut decoded_page = String::new();
	url_escape::decode_to_string(page, &mut decoded_page);
	let alias = unsafe { ALIAS.get().unwrap() };

	for i in alias {
		if i.uri == decoded_page {
			return match i.is_url {
				Some(true) => Err(Redirect::to(i.alias.clone())),
				_ => Ok(RawText(smurf::io::read_file_to_str(&PathBuf::from(&i.alias)).unwrap())),
			};
		}
	}

	Err(Redirect::to("/404"))
}

#[get("/")]
async fn get_index() -> Redirect {
	Redirect::to(INDEX_REDIRECT)
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
	unsafe {
		ALIAS.set(serde_json::from_str(_ALIAS).unwrap()).unwrap_unchecked();
	}
	let _rocket = rocket::build().mount("/", routes![get_page, get_index]).launch().await?;
	Ok(())
}
