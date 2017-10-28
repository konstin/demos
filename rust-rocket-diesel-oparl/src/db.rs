use chrono::NaiveDateTime;
use chrono::NaiveDate;
use schema::{dokumente, antraege};

#[derive(Queryable, Clone)]
pub struct Body {
    pub id: i32,
    pub name: String,
    pub short_name: Option<String>,
    pub website: Option<String>,
}

#[derive(Identifiable, Queryable, Serialize, Clone)]
#[table_name="antraege"]
pub struct Antrag {
    pub id: i32,
    pub vorgang_id: Option<i32>,
    pub typ: String,
    pub datum_letzte_aenderung: NaiveDateTime,
    pub ba_nr: Option<i16>,
    pub gestellt_am: Option<NaiveDate>,
    pub gestellt_von: String,
    pub erledigt_am: Option<NaiveDate>,
    pub antrags_nr: String,
    pub bearbeitungsfrist: Option<NaiveDate>,
    pub registriert_am: Option<NaiveDate>,
    pub referat: String,
    pub referent: String,
    pub referat_id: Option<i32>,
    pub wahlperiode: String,
    pub antrag_typ: String,
    pub betreff: String,
    pub kurzinfo: String,
    pub status: String,
    pub bearbeitung: String,
    pub fristverlaengerung: Option<NaiveDate>,
    pub initiatorInnen: String,
    pub initiative_to_aufgenommen: Option<NaiveDate>,
    pub created: NaiveDateTime,
    pub modified: NaiveDateTime,
}

#[derive(Identifiable, Queryable, Serialize, Clone, Associations)]
#[belongs_to(Antrag, foreign_key="antrag_id")]
#[table_name="dokumente"]
pub struct Dokument {
    pub id: i32,
    pub typ: Option<String>,
    pub antrag_id: Option<i32>,
    pub termin_id: Option<i32>,
    pub tagesordnungspunkt_id: Option<i32>,
    pub vorgang_id: Option<i32>,
    pub rathausumschau_id: Option<i32>,
    pub url: String,
    pub deleted: i8,
    pub name: String,
    pub datum: NaiveDateTime,
    pub datum_dokument: Option<NaiveDateTime>,
    pub text_ocr_raw: Option<String>,
    pub text_ocr_corrected: Option<String>,
    pub text_ocr_garbage_seiten: Option<String>,
    pub text_pdf: Option<String>,
    pub seiten_anzahl: Option<i32>,
    pub ocr_von: Option<String>,
    pub highlight: Option<NaiveDateTime>,
    pub name_title: String,
    pub created: NaiveDateTime,
    pub modified: NaiveDateTime,
}

