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

use rocket::http::Status;
use std::cell::OnceCell;
use std::net::IpAddr;
use std::path::PathBuf;

use rocket::request::{FromRequest, Outcome};
use rocket::response::content::RawText;
use rocket::response::{Redirect, Responder};
use rocket::Request;
use serde::Deserialize;

use clap::Parser;
use rocket::figment::Figment;

static mut ALIAS: OnceCell<Vec<Alias>> = OnceCell::new();

#[derive(Parser, Debug)]
#[command(about, author)]
struct Args {
	#[arg(long, default_value = "./alias.json")]
	alias_file: PathBuf,

	#[arg(short, long, default_value = "127.0.0.1")]
	address: IpAddr,

	#[arg(short, long, default_value = "8080")]
	port: u16,
}

#[derive(Deserialize, Clone, Debug)]
struct Alias {
	uri: String,
	alias: String,
	is_url: Option<bool>,
	curl_only: Option<bool>,
}

#[derive(Responder)]
enum Response {
	Text(RawText<String>),
	Redirect(Redirect),
	Status(Status),
}

struct UserAgent(String);

#[derive(Debug)]
enum UserAgentError {}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserAgent {
	type Error = UserAgentError;

	async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
		match req.headers().get_one("user-agent") {
			Some(key) => Outcome::Success(UserAgent(key.to_string())),
			_ => Outcome::Success(UserAgent("".to_string())),
		}
	}
}

fn get_return(alias: &Alias) -> Response {
	return match alias.is_url {
		Some(true) => Response::Redirect(Redirect::to(alias.alias.clone())),
		_ => {
			Response::Text(RawText(smurf::io::read_file_str(&PathBuf::from(&alias.alias)).unwrap()))
		}
	};
}

#[get("/<page>")]
fn get_page(page: String, user_agent: UserAgent) -> Response {
	let mut decoded_page = String::new();
	url_escape::decode_to_string(page, &mut decoded_page);
	let alias = unsafe { ALIAS.get().unwrap() };
	let mut pages = Vec::new();
	let curl_check = user_agent.0.contains("curl");
	for i in alias {
		if i.uri == decoded_page {
			if (i.curl_only == Some(true) && curl_check.clone())
				|| (i.curl_only != Some(true) && !curl_check.clone())
			{
				return get_return(i);
			};
			pages.push(i);
		}
	}
	// Returning normal page (if  found) to curl users.
	for i in pages {
		if i.curl_only != Some(true) {
			return get_return(i);
		}
	}
	Response::Status(Status::NotFound)
}

#[get("/")]
async fn index(user_agent: UserAgent) -> Response {
	get_page("/".to_string(), user_agent)
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
	let args = Args::parse();
	unsafe {
		ALIAS
			.set(
				serde_json::from_str(&smurf::io::read_file_str(&args.alias_file).unwrap()).unwrap(),
			)
			.unwrap();
	}

	let figment = Figment::from(rocket::Config::default())
		.merge(("ident", "urouter"))
		.merge(("port", args.port))
		.merge(("address", args.address));

	let _rocket = rocket::custom(figment).mount("/", routes![get_page, index]).launch().await?;
	Ok(())
}
