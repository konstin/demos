table! {
    body {
        id -> Integer,
        name -> VarChar,
        shortName -> Nullable<VarChar>,
        website -> Nullable<VarChar>,
    }
}

table! {
    antraege (id) {
        id -> Integer,
        vorgang_id -> Nullable<Integer>,
        typ -> Varchar,
        datum_letzte_aenderung -> Timestamp,
        ba_nr -> Nullable<Smallint>,
        gestellt_am -> Nullable<Date>,
        gestellt_von -> Mediumtext,
        erledigt_am -> Nullable<Date>,
        antrags_nr -> Varchar,
        bearbeitungsfrist -> Nullable<Date>,
        registriert_am -> Nullable<Date>,
        referat -> Varchar,
        referent -> Varchar,
        referat_id -> Nullable<Integer>,
        wahlperiode -> Varchar,
        antrag_typ -> Varchar,
        betreff -> Mediumtext,
        kurzinfo -> Mediumtext,
        status -> Varchar,
        bearbeitung -> Varchar,
        fristverlaengerung -> Nullable<Date>,
        initiatorInnen -> Mediumtext,
        initiative_to_aufgenommen -> Nullable<Date>,
        created -> Timestamp,
        modified -> Timestamp,
    }
}

table! {
    dokumente (id) {
        id -> Integer,
        typ -> Nullable<Varchar>,
        antrag_id -> Nullable<Integer>,
        termin_id -> Nullable<Integer>,
        tagesordnungspunkt_id -> Nullable<Integer>,
        vorgang_id -> Nullable<Integer>,
        rathausumschau_id -> Nullable<Integer>,
        url -> Varchar,
        deleted -> Tinyint,
        name -> Varchar,
        datum -> Timestamp,
        datum_dokument -> Nullable<Timestamp>,
        text_ocr_raw -> Nullable<Longtext>,
        text_ocr_corrected -> Nullable<Longtext>,
        text_ocr_garbage_seiten -> Nullable<Mediumtext>,
        text_pdf -> Nullable<Longtext>,
        seiten_anzahl -> Nullable<Integer>,
        ocr_von -> Nullable<Varchar>,
        highlight -> Nullable<Timestamp>,
        name_title -> Varchar,
        created -> Timestamp,
        modified -> Timestamp,
    }
}

joinable!(dokumente -> antraege (antrag_id));
