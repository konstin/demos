#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;
extern crate reqwest;
extern crate url_serde;
extern crate url;
extern crate chrono;

pub mod schema;
pub mod external_list;
pub mod urls;

use schema::*;
use external_list::ExternalList;

use std::error::Error;

fn get() -> Result<(), Box<Error>> {
    let system: System = reqwest::get("http://localhost:8080/oparl/v1.0/")?.json()?;
    println!("{}", system.id);

    let bodies: ExternalList<Body> = system.body.try_into()?;
    for body in bodies.data {
        if let Some(x) = body.system {
            println!("System: {}", x.try_into()?.name.unwrap_or("Unknown".into()));
        }

        println!("Persons of {}, Url {}", body.name, body.person.get_url());
        for person in body.person.try_into()?.data {
            println!("Name: {:?}", person.name);
        }
    }
    Ok(())
}

fn main() {
    get().unwrap();
}
