#![feature(option_result_contains)]
use clap::{App,Arg};
use maildir::{Maildir};
use mailparse::{ParsedMail, MailHeader};
use tantivy::schema::*;
use tantivy::{doc,Index,directory::MmapDirectory};

// Message-Id From To Subject Date Thread-Topic 

fn main() {
    let matches = app().get_matches();
    let dir = matches.value_of("DIR").unwrap();
    let maildir = Maildir::from(dir);
    let schema = schema();
    let index = Index::open_or_create(MmapDirectory::open("/home/maufl/.test").expect("Unable to open index directory"), schema.clone()).expect("Failed to create index");
    let mut index_writer = index.writer(50_000_000).expect("Failed to create index");
    let from_field = schema.get_field("from").unwrap();
    let to_field = schema.get_field("to").unwrap();
    let body_field = schema.get_field("body").unwrap();
    let subject_field = schema.get_field("subject").unwrap();
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
        let body = parsed.get_body().unwrap_or("".into());
        let subject = get_header(&parsed, "subject").unwrap_or("".into());
        println!("Indexing email with to={} from={} subject={}", to, from, subject);
        let doc = doc!(
            to_field => to,
            from_field => from,
            subject_field => subject,
            body_field => body
        );
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
    schema_builder.build()
}

fn app<'a, 'b>() -> App<'a, 'b> {
    App::new("mailindex")
    .version("0.1")
    .author("Felix Konstantin Maurer <maufl@maufl.de>")
    .about("Indexes emails using tantivy")
    .arg(Arg::with_name("DIR").help("The maildir to index").required(true).index(1))
}