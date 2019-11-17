#![feature(option_result_contains)]
use clap::{App,Arg,SubCommand};

mod serve;
mod index;

// Message-Id From To Subject Date Thread-Topic 

fn main() {
    let matches = app().get_matches();
    match matches.subcommand() {
        ("index", Some(index_matches)) => index::run_index(index_matches),
        ("serve", Some(serve_matches)) => { serve::run_serve_cli(serve_matches); },
        _ => println!("Unexpected arguments")
    };
}

fn app<'a, 'b>() -> App<'a, 'b> {
    App::new("mailindex")
    .version("0.1")
    .author("Felix Konstantin Maurer <maufl@maufl.de>")
    .about("Indexes emails using tantivy")
    .subcommand(SubCommand::with_name("index")
        .arg(Arg::with_name("index").short("i").takes_value(true).required(true))
        .arg(Arg::with_name("DIR").help("The maildir to index").required(true).index(1))
    )
    .subcommand(SubCommand::with_name("serve")
        .arg(Arg::with_name("index").short("i").takes_value(true).required(true))
        .arg(Arg::with_name("port").short("p").takes_value(true))
        .arg(Arg::with_name("host").short("h").takes_value(true))
    )
}