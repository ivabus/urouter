/* SPDX-License-Identifier: MIT */

/*
Global comments:
- I'm ok with unwrapping because if unwrap fails Rocket will automatically return
  500 Internal Server Error
*/

mod structs;
use structs::*;

#[macro_use]
extern crate rocket;

use rocket::http::{ContentType, Status};
use std::{
	cell::OnceCell, collections::HashMap, hint::unreachable_unchecked, path::PathBuf, time::Instant,
};

use rocket::{
	figment::Figment,
	response::{content::RawText, Redirect},
};

use clap::Parser;
use regex::Regex;

static mut ALIAS: OnceCell<Vec<Alias>> = OnceCell::new();
static mut COMPILED_REGEXES: OnceCell<HashMap<String, Regex>> = OnceCell::new();

fn get_return(alias: &Alias) -> Response {
	let args = Args::parse();
	let mut dir = args.dir.clone();
	match &alias.alias {
		AliasType::Url(url) => Response::Redirect(Box::from(Redirect::to(url.clone()))),
		AliasType::File(path) => {
			dir.push(&PathBuf::from(&path));
			Response::Text(Box::new(RawText(smurf::io::read_file_str(&dir).unwrap())))
		}
		AliasType::Text(text) => Response::Text(Box::new(RawText(text.clone()))),
		AliasType::External(source) => {
			let mut request = ureq::get(&source.url);
			for (header, value) in &source.headers {
				request = request.set(header, value);
			}
			let result = request.call().unwrap();
			let ct = result.content_type();
			Response::Custom(Box::new((
				ContentType::parse_flexible(ct).unwrap(),
				RawText(result.into_string().unwrap()),
			)))
		}
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
	let mut args = Args::parse();
	args.alias_file = match args.alias_file {
		Some(f) => Some(f),
		None => Some(
			match users::get_effective_uid() {
				0 => "/etc/urouter/alias.json".to_string(),
				_ => match std::env::var("XDG_CONFIG_HOME") {
					Ok(config_home) => format!("{}/urouter/alias.json", config_home),
					Err(_) => match std::env::var("HOME") {
						Ok(home) => format!("{}/.config/urouter/alias.json", home),
						Err(_) => {
							panic!("Could not get config location, see README")
						}
					},
				},
			}
			.into(),
		),
	};
	let file = std::fs::File::open(args.alias_file.unwrap()).unwrap();
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
		COMPILED_REGEXES.set(compiled_regexes).unwrap();
	}

	let figment = Figment::from(rocket::Config::default())
		.merge(("ident", format!("urouter/{}", env!("CARGO_PKG_VERSION"))))
		.merge(("port", args.port))
		.merge(("address", args.address));

	let _rocket = rocket::custom(figment).mount("/", routes![get_page, index]).launch().await?;
	Ok(())
}
