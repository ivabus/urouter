/* SPDX-License-Identifier: MIT */

/*
Global comments:
- I'm ok with unwrapping because if unwrap fails Rocket will automatically return
  500 Internal Server Error
*/

#![forbid(unsafe_code)]

mod structs;
use structs::*;

#[macro_use]
extern crate rocket;

use std::io::Write;
use std::sync::Arc;
use std::{collections::HashMap, path::PathBuf, time::Instant};

use rocket::http::{ContentType, Status};
use rocket::{
	figment::Figment,
	response::{content::RawHtml, content::RawText, Redirect},
};

use clap::Parser;
use once_cell::sync::Lazy;
use regex::Regex;

static ALIAS: Lazy<Arc<Vec<Alias>>> = Lazy::new(|| {
	let mut args = Args::parse();
	if args.alias_file.is_none() {
		args.alias_file = Some(get_config_file_location());
	}

	let file = std::fs::File::open(args.alias_file.unwrap()).unwrap();
	let alias: Vec<Alias> = if args.alias_file_is_set_not_a_list {
		serde_json::from_reader::<std::fs::File, NixJson>(file).unwrap().alias
	} else {
		serde_json::from_reader::<std::fs::File, Vec<Alias>>(file).unwrap()
	};
	Arc::new(alias)
});
static COMPILED_REGEXES: Lazy<Arc<HashMap<String, Regex>>> = Lazy::new(|| {
	let compilation_start = Instant::now();
	let mut regexes_len = 0;
	// Precompile all regexes
	let mut compiled_regexes: HashMap<String, Regex> = HashMap::new();
	for i in &**ALIAS {
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
	Arc::new(compiled_regexes)
});

fn get_return(alias: &Alias) -> Response {
	let args = Args::parse();
	let mut dir = args.dir.clone();
	match &alias.alias {
		AliasType::Url(url) => Response::Redirect(Box::from(Redirect::to(url.clone()))),
		AliasType::File(path) => {
			dir.push(&PathBuf::from(&path));
			Response::Text(Box::new(RawText(std::fs::read_to_string(&dir).unwrap())))
		}
		AliasType::Text(text) => Response::Text(Box::new(RawText(text.clone()))),
		AliasType::Html(html) => Response::Html(Box::new(RawHtml(html.clone()))),
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
	let mut pages = Vec::new();
	for i in &**ALIAS {
		if i.uri == decoded_page {
			if let Some(agent) = &i.agent {
				let regexes = &COMPILED_REGEXES;
				let re = regexes.get(&agent.regex).unwrap();

				if re.is_match(&user_agent.0) {
					return get_return(i);
				}

				if let Some(true) = agent.only_matching {
					continue;
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

fn get_config_file_location() -> PathBuf {
	if users::get_effective_uid() == 0 {
		return "/etc/urouter/alias.json".parse().unwrap();
	}

	if let Ok(config_home) = std::env::var("XDG_CONFIG_HOME") {
		return format!("{}/urouter/alias.json", config_home).parse().unwrap();
	}

	if let Ok(home) = std::env::var("HOME") {
		return format!("{}/.config/urouter/alias.json", home).parse().unwrap();
	}

	if !std::path::Path::new("alias.json").exists() {
		let mut file = std::fs::File::create("alias.json").unwrap();
		file.write_all(b"[]").unwrap();
	}
	"alias.json".parse().unwrap()
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
	let args = Args::parse();
	let _alias = &**ALIAS;
	let _regex = &**COMPILED_REGEXES;

	let figment = Figment::from(rocket::Config::default())
		.merge(("ident", format!("urouter/{}", env!("CARGO_PKG_VERSION"))))
		.merge(("port", args.port))
		.merge(("address", args.address));

	let _rocket = rocket::custom(figment).mount("/", routes![get_page, index]).launch().await?;
	Ok(())
}
