use clap::{ArgMatches};
use maildir::{Maildir};
use mailparse::{ParsedMail, MailHeader};
use tantivy::schema::*;
use tantivy::{doc,Index,directory::MmapDirectory};
use chrono::{DateTime,offset::Utc};

pub fn run_index(matches: &ArgMatches) {
    let dir = matches.value_of("DIR").unwrap();
    let index = matches.value_of("index").unwrap_or("~/.test");
    let maildir = Maildir::from(dir);
    let schema = schema();
    let index = Index::open_or_create(MmapDirectory::open(index).expect("Unable to open index directory"), schema.clone()).expect("Failed to create index");
    let mut index_writer = index.writer(50_000_000).expect("Failed to create index");
    let from_field = schema.get_field("from").unwrap();
    let to_field = schema.get_field("to").unwrap();
    let body_field = schema.get_field("body").unwrap();
    let subject_field = schema.get_field("subject").unwrap();
    let date_field = schema.get_field("date").unwrap();
    for entry in maildir.list_cur() {
        let mut mail = match entry {
            Ok(m) => m,
            Err(_) => continue,
        };
        let parsed = match mail.parsed() {
            Ok(p) => p,
            Err(_) => continue,
        };
        let to = get_header(&parsed, "to").unwrap_or("".into());
        let from = get_header(&parsed, "from").unwrap_or("".into());
        let subject = get_header(&parsed, "subject").unwrap_or("".into());
        let date_string = get_header(&parsed, "date").unwrap_or("".into());
        let date = DateTime::parse_from_rfc2822(&date_string).ok();
        let mut body = "".to_owned(); 
        if parsed.ctype.mimetype == "text/plain" {
            body = parsed.get_body().unwrap_or("".into());
        }
        for part in &parsed.subparts {
            if part.ctype.mimetype == "text/plain"{
                body = part.get_body().unwrap_or(body);
            }
        }
        let mut doc = doc!(
            to_field => to,
            from_field => from,
            subject_field => subject,
            body_field => body
        );
        if let Some(date_value) = date {
            doc.add_date(date_field, &DateTime::from_utc(date_value.naive_utc(), Utc));
        }
        index_writer.add_document(doc);
    }
    index_writer.commit().expect("Unable to commit");
}

fn get_header<'a> (mail: &'a ParsedMail, key: &str) -> Option<String> {
    for header in &mail.headers {
        if header.get_key().unwrap_or("".to_owned()).to_lowercase() == key.to_lowercase() {
            return Some(header.get_value().unwrap_or("".into()));
        }
    }
    return None;
}

fn schema() -> Schema {
    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("subject", TEXT | STORED);
    schema_builder.add_text_field("body", TEXT | STORED);
    schema_builder.add_text_field("from", TEXT | STORED);
    schema_builder.add_text_field("to", TEXT | STORED);
    schema_builder.add_date_field("date", FAST | INDEXED | STORED);
    schema_builder.build()
}
