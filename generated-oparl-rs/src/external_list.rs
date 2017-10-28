#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalListPagination {}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalListLinks {}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalList<T> {
    pub data: Vec<T>,
    pub pagination: ExternalListPagination,
    pub links: ExternalListLinks,
}

