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

mod structs;

use structs::*;
#[macro_use]
extern crate rocket;

use rocket::http::Status;
use std::cell::{OnceCell, RefCell};
use std::collections::HashMap;
use std::hint::unreachable_unchecked;
use std::path::PathBuf;
use std::time::Instant;

use rocket::response::content::RawText;
use rocket::response::Redirect;

use clap::Parser;
use regex::Regex;
use rocket::figment::Figment;

static mut ALIAS: OnceCell<Vec<Alias>> = OnceCell::new();
static mut COMPILED_REGEXES: RefCell<Option<HashMap<String, Regex>>> = RefCell::new(None);

fn get_return(alias: &Alias) -> Response {
	let args = Args::parse();
	let mut dir = args.dir.clone();
	match &alias.alias {
		AliasType::Url(url) => Response::Redirect(Box::from(Redirect::to(url.clone()))),
		AliasType::File(path) => {
			dir.push(&PathBuf::from(&path));
			Response::Text(RawText(smurf::io::read_file_str(&dir).unwrap()))
		}
		AliasType::Text(text) => Response::Text(RawText(text.clone())),
	}
}

#[get("/<page>")]
fn get_page(page: &str, user_agent: UserAgent) -> Response {
	let mut decoded_page = String::new();
	url_escape::decode_to_string(page, &mut decoded_page);
	let alias = unsafe { ALIAS.get().unwrap() };
	let mut pages = Vec::new();
	for i in alias {
		if i.uri == decoded_page {
			if let Some(agent) = &i.agent {
				unsafe {
					let regexes = COMPILED_REGEXES.get_mut();
					let re = if let Some(r) = regexes {
						// Unwrapping safely, guaranteed to be generated during initialization
						r.get(&agent.regex).unwrap()
					} else {
						unreachable_unchecked()
					};

					if re.is_match(&user_agent.0) {
						return get_return(i);
					}

					if let Some(true) = agent.only_matching {
						continue;
					}
				}
			}
			pages.push(i);
		}
	}
	// Returning normal page (if  found) to curl users.
	if !pages.is_empty() {
		return get_return(pages[0]);
	}
	Response::Status(Status::NotFound)
}

#[get("/")]
async fn index(user_agent: UserAgent) -> Response {
	get_page("/", user_agent)
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
	let args = Args::parse();
	let file = std::fs::File::open(args.alias_file).unwrap();
	let alias: Vec<Alias> = if args.alias_file_is_set_not_a_list {
		serde_json::from_reader::<std::fs::File, NixJson>(file).unwrap().alias
	} else {
		serde_json::from_reader::<std::fs::File, Vec<Alias>>(file).unwrap()
	};
	unsafe {
		ALIAS.set(alias).unwrap();

		let compilation_start = Instant::now();
		let mut regexes_len = 0;
		// Precompile all regexes
		let mut compiled_regexes: HashMap<String, Regex> = HashMap::new();
		for i in ALIAS.get().unwrap() {
			if let Some(agent) = &i.agent {
				compiled_regexes.insert(agent.regex.clone(), Regex::new(&agent.regex).unwrap());
				regexes_len += 1;
			}
		}
		if regexes_len != 0 {
			println!(
				"Compiled {} regexes in {} ms",
				regexes_len,
				(Instant::now() - compilation_start).as_secs_f64() * 1000.0
			);
		}
		*COMPILED_REGEXES.get_mut() = Some(compiled_regexes);
	}

	let figment = Figment::from(rocket::Config::default())
		.merge(("ident", format!("urouter/{}", env!("CARGO_PKG_VERSION"))))
		.merge(("port", args.port))
		.merge(("address", args.address));

	let _rocket = rocket::custom(figment).mount("/", routes![get_page, index]).launch().await?;
	Ok(())
}
