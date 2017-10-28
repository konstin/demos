#![feature(plugin)]
#![plugin(rocket_codegen)]
#![feature(custom_attribute)]
#![recursion_limit="128"]

extern crate rocket;
extern crate rocket_contrib;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
#[macro_use] extern crate serde_derive;
extern crate dotenv;
extern crate chrono;

use rocket_contrib::Json;
use diesel::prelude::*;
use diesel::mysql::MysqlConnection;
use dotenv::dotenv;
use std::env;

pub mod schema;
pub mod db;
pub mod oparl;

pub mod oparl_types {
    pub const PREFIX: &'static str = "https://schema.oparl.org/1.1/";
    pub const SYSTEM: &'static str = "https://schema.oparl.org/1.1/System";
    pub const BODY: &'static str = "https://schema.oparl.org/1.1/Body";
    pub const PAPER: &'static str = "https://schema.oparl.org/1.1/Paper";
    pub const FILE: &'static str = "https://schema.oparl.org/1.1/File";
}

const BASE_URL: &'static str = "http://localhost:8000";

pub fn establish_connection() -> MysqlConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    MysqlConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

#[get("/paper/<from_id>")]
fn paper_from_id(from_id: i32) -> Json<Vec<oparl::Paper>> {
    let connection = establish_connection();

    let antraege: Vec<db::Antrag> = schema::antraege::table
        .filter(schema::antraege::dsl::id.gt(from_id as i32))
        .limit(100)
        .order(schema::antraege::dsl::id)
        .load::<db::Antrag>(&connection)
        .expect("Error loading papers");

    let max_id = match antraege.last() {
        None => return Json(Vec::new()),
        Some(last) => last.id,
    };

    // Apparently more performant, but not as elegant
    /*let mut dokumente: Vec<db::Dokument> = schema::dokumente::table
        .filter(schema::dokumente::columns::antrag_id.gt(from_id))
        .filter(schema::dokumente::columns::antrag_id.le(max_id))
        .order(schema::dokumente::columns::antrag_id.desc())
        .load::<db::Dokument>(&connection)
        .expect("Error loading papers");


    let mut paper = Vec::new();

    for antrag in antraege {
        let index = dokumente.iter().rposition(|x| x.antrag_id.unwrap() != antrag.id).unwrap_or(dokumente.len()-1);
        let files_for_papers = dokumente.split_off(index+1);
        let files = files_for_papers.iter().map(|x| oparl::File::from_dokument(x.clone())).collect();
        paper.push(oparl::Paper::from_antrag(antrag, files));
    }*/

    let dokumente = db::Dokument::belonging_to(&antraege).load(&connection).expect("Error loading papers");
    let grouped_dokumente = dokumente.grouped_by(&antraege);

    let paper = antraege.into_iter().zip(grouped_dokumente).map(|(antrag, dokumente_for_antrag)| {
        let files = dokumente_for_antrag.into_iter().map(|x| oparl::File::from_dokument(x)).collect();
        oparl::Paper::from_antrag(antrag, files)
    }).collect();

    Json(paper)
}

#[get("/paper")]
pub fn paper() -> Json<Vec<db::Antrag>> {
    let connection = establish_connection();

    let results: Vec<db::Antrag> = schema::antraege::table
        .limit(100)
        .order(schema::antraege::dsl::id)
        .load::<db::Antrag>(&connection)
        .expect("Error loading papers");

    Json(results)
}

#[get("/")]
pub fn index() -> Json<oparl::Body> {
    let connection = establish_connection();
    let results: Vec<db::Body> = schema::body::table
        .limit(5)
        .load::<db::Body>(&connection)
        .expect("Error loading posts");

    let db_body = results[0].clone();

    let system = oparl::Body {
        id: BASE_URL.to_string(),
        oparl_type: oparl_types::SYSTEM.to_string(),
        name: db_body.name,
        short_name: db_body.short_name,
        website: db_body.website,
    };

    Json(system)
}

fn main() {
    rocket::ignite().mount("/", routes![index, paper, paper_from_id]).launch();
}
