use clap::Parser;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::response::content::RawText;
use rocket::response::Redirect;
use rocket::Request;
use serde::Deserialize;
use std::net::IpAddr;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(about, author)]
pub struct Args {
	#[arg(long, default_value = "./alias.json")]
	pub alias_file: PathBuf,

	/// For internal usage
	#[arg(long, default_value = "false")]
	pub alias_file_is_set_not_a_list: bool,

	/// Dir to lookup file alias
	#[arg(long, default_value = ".")]
	pub dir: PathBuf,

	#[arg(short, long, default_value = "127.0.0.1")]
	pub address: IpAddr,

	#[arg(short, long, default_value = "8080")]
	pub port: u16,
}

// For better compatability with Nix (with set on the top of alias.json instead of a list)
#[derive(Deserialize, Clone, Debug)]
pub struct NixJson {
	pub alias: Vec<Alias>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Alias {
	pub uri: String,
	pub alias: AliasType,
	pub agent: Option<Agent>,
}

#[derive(Deserialize, Clone, Debug)]
pub enum AliasType {
	#[serde(alias = "url")]
	Url(String),
	#[serde(alias = "file")]
	File(String),
	#[serde(alias = "text")]
	Text(String),
}

#[derive(Deserialize, Clone, Debug)]
pub struct Agent {
	pub regex: String,
	pub only_matching: Option<bool>,
}

#[derive(Responder)]
pub enum Response {
	Text(RawText<String>),
	Redirect(Redirect),
	Status(Status),
}

pub struct UserAgent(pub String);

#[derive(Debug)]
pub enum UserAgentError {}

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
