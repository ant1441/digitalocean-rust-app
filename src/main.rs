#[macro_use]
extern crate rocket;

mod paste_id;
#[cfg(test)]
mod tests;

use std::env;
use std::io;

use rocket::data::{Data, ToByteUnit};
use rocket::response::{content::Plain, Debug};
use rocket::tokio::fs::File;

use crate::paste_id::PasteID;

const HOST: &str = "http://localhost:8000";
const ID_LENGTH: usize = 3;

#[post("/", data = "<paste>")]
async fn upload(paste: Data) -> Result<String, Debug<io::Error>> {
    let id = PasteID::new(ID_LENGTH);
    let filename = format!("upload/{id}", id = id);
    let url = format!("{host}/{id}\n", host = HOST, id = id);

    paste.open(128.kibibytes()).stream_to_file(filename).await?;
    Ok(url)
}

#[get("/<id>")]
async fn retrieve(id: PasteID<'_>) -> Option<Plain<File>> {
    let filename = format!("upload/{id}", id = id);
    File::open(&filename).await.map(Plain).ok()
}

#[get("/")]
fn index() -> &'static str {
    "
    USAGE
      POST /
          accepts raw data in the body of the request and responds with a URL of
          a page containing the body's content
          EXAMPLE: curl --data-binary @file.txt http://localhost:8000
      GET /<id>
          retrieves the content for the paste with id `<id>`
    "
}

#[launch]
fn rocket() -> rocket::Rocket {
    let port = env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8000);
    let secret_key = env::var("ROCKET_SECRET_KEY").ok();

    let config_builder =
        rocket::Config::build(rocket::config::Environment::active().unwrap()).port(port);
    let config_builder = if let Some(key) = secret_key {
        config_builder.secret_key(key)
    } else {
        config_builder
    };
    let config = config_builder.finalize().unwrap();
    rocket::custom(config).mount("/", routes![index, upload, retrieve])
}
