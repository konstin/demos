use serde_json::Value as JsonValue;
use urls::{OParlUrl, ExternalListUrl};
use chrono::prelude::*;
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgendaItem{
    pub license: Option<String>,
    pub keyword: Option<Vec<String>>,
    pub auxiliary_file: Option<Vec<File>>,
    pub result: Option<String>,
    pub consultation: Option<OParlUrl<Consultation>>,
    pub resolution_file: Option<File>,
    pub resolution_text: Option<String>,
    #[serde(rename = "type")]
    pub oparl_type: String,
    pub number: Option<String>,
    pub end: Option<DateTime<FixedOffset>>,
    pub public: Option<bool>,
    pub meeting: Option<OParlUrl<Meeting>>,
    pub id: String,
    pub name: Option<String>,
    pub start: Option<DateTime<FixedOffset>>,
    pub web: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Person{
    pub keyword: Option<Vec<String>>,
    pub title: Option<Vec<String>>,
    pub body: Option<OParlUrl<Body>>,
    pub form_of_address: Option<String>,
    pub phone: Option<Vec<String>>,
    pub location: Option<Location>,
    #[serde(rename = "type")]
    pub oparl_type: String,
    pub web: Option<String>,
    pub modified: Option<DateTime<FixedOffset>>,
    pub license: Option<String>,
    pub name: Option<String>,
    pub status: Option<Vec<String>>,
    pub given_name: Option<String>,
    pub gender: Option<String>,
    pub life: Option<String>,
    pub life_source: Option<String>,
    pub family_name: Option<String>,
    pub id: String,
    pub created: Option<DateTime<FixedOffset>>,
    pub deleted: Option<bool>,
    pub affix: Option<String>,
    pub email: Option<Vec<String>>,
    pub membership: Option<Vec<Membership>>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct File{
    pub text: Option<String>,
    pub file_name: Option<String>,
    pub keyword: Option<Vec<String>>,
    pub paper: Option<Vec<String>>,
    pub sha1_checksum: Option<String>,
    pub file_license: Option<String>,
    pub mime_type: Option<String>,
    pub agenda_item: Option<Vec<String>>,
    #[serde(rename = "type")]
    pub oparl_type: String,
    pub web: Option<String>,
    pub modified: Option<DateTime<FixedOffset>>,
    pub license: Option<String>,
    pub name: Option<String>,
    pub meeting: Option<Vec<String>>,
    pub access_url: String,
    pub date: Option<String>,
    pub master_file: Option<OParlUrl<File>>,
    pub size: Option<usize>,
    pub derivative_file: Option<Vec<String>>,
    pub id: String,
    pub created: Option<DateTime<FixedOffset>>,
    pub deleted: Option<bool>,
    pub sha512_checksum: Option<String>,
    pub external_service_url: Option<String>,
    pub download_url: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LegislativeTerm{
    pub keyword: Option<Vec<String>>,
    #[serde(rename = "type")]
    pub oparl_type: String,
    pub web: Option<String>,
    pub end_date: Option<String>,
    pub start_date: Option<String>,
    pub license: Option<String>,
    pub body: Option<OParlUrl<Body>>,
    pub name: Option<String>,
    pub id: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Body{
    pub website: Option<String>,
    pub keyword: Option<Vec<String>>,
    pub location: Option<Location>,
    pub equivalent: Option<Vec<String>>,
    pub oparl_since: Option<DateTime<FixedOffset>>,
    pub paper: ExternalListUrl<Paper>,
    pub short_name: Option<String>,
    #[serde(rename = "type")]
    pub oparl_type: String,
    pub web: Option<String>,
    pub modified: Option<DateTime<FixedOffset>>,
    pub rgs: Option<String>,
    pub classification: Option<String>,
    pub meeting: ExternalListUrl<Meeting>,
    pub name: String,
    pub license: Option<String>,
    pub ags: Option<String>,
    pub person: ExternalListUrl<Person>,
    pub contact_email: Option<String>,
    pub system: Option<OParlUrl<System>>,
    pub id: String,
    pub created: Option<DateTime<FixedOffset>>,
    pub deleted: Option<bool>,
    pub organization: ExternalListUrl<Organization>,
    pub contact_name: Option<String>,
    pub license_valid_since: Option<DateTime<FixedOffset>>,
    pub legislative_term: Vec<LegislativeTerm>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Consultation{
    pub agenda_item: Option<OParlUrl<AgendaItem>>,
    pub authoritative: Option<bool>,
    #[serde(rename = "type")]
    pub oparl_type: String,
    pub organization: Option<Vec<String>>,
    pub web: Option<String>,
    pub keyword: Option<Vec<String>>,
    pub role: Option<String>,
    pub meeting: Option<OParlUrl<Meeting>>,
    pub paper: Option<OParlUrl<Paper>>,
    pub license: Option<String>,
    pub id: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Location{
    pub keyword: Option<Vec<String>>,
    pub bodies: Option<Vec<OParlUrl<Body>>>,
    pub street_address: Option<String>,
    pub papers: Option<Vec<OParlUrl<Paper>>>,
    pub geojson: Option<JsonValue>,
    pub web: Option<String>,
    pub id: String,
    pub description: Option<String>,
    pub created: Option<DateTime<FixedOffset>>,
    pub deleted: Option<bool>,
    #[serde(rename = "type")]
    pub oparl_type: String,
    pub organizations: Option<Vec<OParlUrl<Organization>>>,
    pub modified: Option<DateTime<FixedOffset>>,
    pub locality: Option<String>,
    pub license: Option<String>,
    pub persons: Option<Vec<OParlUrl<Person>>>,
    pub sub_locality: Option<String>,
    pub meetings: Option<Vec<OParlUrl<Meeting>>>,
    pub postal_code: Option<String>,
    pub room: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Organization{
    pub website: Option<String>,
    pub external_body: Option<OParlUrl<Body>>,
    pub location: Option<Location>,
    pub keyword: Option<Vec<String>>,
    pub short_name: Option<String>,
    pub organization_type: Option<String>,
    #[serde(rename = "type")]
    pub oparl_type: String,
    pub web: Option<String>,
    pub modified: Option<DateTime<FixedOffset>>,
    pub classification: Option<String>,
    pub meeting: Option<ExternalListUrl<Meeting>>,
    pub start_date: Option<String>,
    pub license: Option<String>,
    pub created: Option<DateTime<FixedOffset>>,
    pub sub_organization_of: Option<OParlUrl<Organization>>,
    pub name: Option<String>,
    pub body: Option<OParlUrl<Body>>,
    pub id: String,
    pub post: Option<Vec<String>>,
    pub deleted: Option<bool>,
    pub end_date: Option<String>,
    pub membership: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Membership{
    pub keyword: Option<Vec<String>>,
    pub web: Option<String>,
    pub person: Option<OParlUrl<Person>>,
    pub voting_right: Option<bool>,
    pub id: String,
    #[serde(rename = "type")]
    pub oparl_type: String,
    pub organization: Option<OParlUrl<Organization>>,
    pub end_date: Option<String>,
    pub role: Option<String>,
    pub license: Option<String>,
    pub start_date: Option<String>,
    pub on_behalf_of: Option<OParlUrl<Organization>>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Meeting{
    pub keyword: Option<Vec<String>>,
    pub results_protocol: Option<File>,
    pub location: Option<Location>,
    pub auxiliary_file: Option<Vec<File>>,
    pub created: Option<DateTime<FixedOffset>>,
    pub id: String,
    pub verbatim_protocol: Option<File>,
    pub invitation: Option<File>,
    pub cancelled: Option<bool>,
    pub agenda_item: Option<Vec<AgendaItem>>,
    pub deleted: Option<bool>,
    #[serde(rename = "type")]
    pub oparl_type: String,
    pub organization: Option<Vec<String>>,
    pub modified: Option<DateTime<FixedOffset>>,
    pub end: Option<DateTime<FixedOffset>>,
    pub meeting_state: Option<String>,
    pub license: Option<String>,
    pub participant: Option<Vec<String>>,
    pub name: Option<String>,
    pub start: Option<DateTime<FixedOffset>>,
    pub web: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct System{
    pub website: Option<String>,
    pub product: Option<String>,
    pub created: Option<DateTime<FixedOffset>>,
    pub contact_email: Option<String>,
    pub body: ExternalListUrl<Body>,
    pub id: String,
    pub oparl_version: String,
    pub deleted: Option<bool>,
    #[serde(rename = "type")]
    pub oparl_type: String,
    pub other_oparl_versions: Option<Vec<String>>,
    pub modified: Option<DateTime<FixedOffset>>,
    pub vendor: Option<String>,
    pub contact_name: Option<String>,
    pub license: Option<String>,
    pub name: Option<String>,
    pub web: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Paper{
    pub keyword: Option<Vec<String>>,
    pub related_paper: Option<Vec<String>>,
    pub paper_type: Option<String>,
    pub auxiliary_file: Option<Vec<File>>,
    pub subordinated_paper: Option<Vec<String>>,
    pub consultation: Option<Vec<Consultation>>,
    pub location: Option<Vec<Location>>,
    #[serde(rename = "type")]
    pub oparl_type: String,
    pub web: Option<String>,
    pub modified: Option<DateTime<FixedOffset>>,
    pub license: Option<String>,
    pub name: Option<String>,
    pub reference: Option<String>,
    pub date: Option<String>,
    pub body: Option<OParlUrl<Body>>,
    pub under_direction_of: Option<Vec<String>>,
    pub id: String,
    pub created: Option<DateTime<FixedOffset>>,
    pub deleted: Option<bool>,
    pub originator_person: Option<Vec<String>>,
    pub superordinated_paper: Option<Vec<String>>,
    pub originator_organization: Option<Vec<String>>,
    pub main_file: Option<File>,
}
