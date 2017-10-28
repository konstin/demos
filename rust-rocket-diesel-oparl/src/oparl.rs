use db::{Antrag, Dokument};
use chrono::prelude::*;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Body {
    pub id: String,
    #[serde(rename = "type")]
    pub oparl_type: String,
    pub name: String,
    pub short_name: Option<String>,
    pub website: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Paper {
    pub id: String,
    #[serde(rename = "type")]
    pub oparl_type: String,
    pub name: String,
    pub date: Option<NaiveDate>,
    pub auxiliary_file: Vec<File>,
}

impl Paper {
    pub fn from_antrag(antrag: Antrag, files: Vec<File>) -> Paper {
        Paper {
            id: format!("https://{}", antrag.id),
            oparl_type: super::oparl_types::PAPER.into(),
            name: antrag.betreff,
            date: antrag.gestellt_am,
            auxiliary_file: files,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct File {
    pub id: String,
    #[serde(rename = "type")]
    pub oparl_type: String,
    pub name: String,
    pub file_name: String,
    pub created: DateTime<Local>,
}

impl File {
    pub fn from_dokument(dokument: Dokument) -> File {
        File {
            id: format!("https://{}", dokument.id),
            oparl_type: super::oparl_types::PAPER.into(),
            name: dokument.name,
            file_name: dokument.name_title,
            created: Local.from_local_datetime(&dokument.created).unwrap(),
        }
    }
}