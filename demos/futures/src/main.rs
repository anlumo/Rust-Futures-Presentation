#![feature(futures_api)]

use std::io::{self, Write};
use rustop::opts;
use std::fs::File;

use tokio::prelude::future::collect;
use hyper::Client;
use hyper::rt::{Future, Stream};
use headers::{ContentLength, HeaderMapExt};

use crossterm::ClearType;

mod config;

fn main() {
    let (args, _rest) = opts! {
        synopsis "Load files in batch";
        param path:String, desc:"The file containing the list of URLs", required:true;
    }.parse_or_exit();

    let list = File::open(&args.path).and_then(|file| Ok(config::load_config(file).expect("Please specify a valid file!"))).expect("Failed to open config file.");

    let https = hyper_rustls::HttpsConnector::new(4);

    let client: Client<_, hyper::Body> = Client::builder().build(https);

    crossterm::terminal().clear(ClearType::All).unwrap();
    crossterm::cursor().hide().unwrap();

    let futures = list.into_iter().enumerate().map(move |(idx, url)| {
        let uri = url.0.parse().expect("Failed parsing URL");

        client.get(uri).and_then(move |response| {
            crossterm::cursor().goto(0, idx as u16 * 2).unwrap();
            print!("Response for {}: {}", url.0, response.status());
            let total = response.headers().typed_get::<ContentLength>().expect("Response didn't contain a content length").0 as usize;
            let mut loaded = 0;

            response.into_body().for_each(move |chunk| {
                let len = chunk.into_bytes().len();
                loaded += len;
                let percent = (100 * loaded) / total;
                crossterm::cursor().goto(0, idx as u16 * 2).unwrap();
                crossterm::terminal().clear(ClearType::CurrentLine).unwrap();
                io::stdout().write_all("X".repeat(percent).as_bytes()).map_err(|e| panic!("stdout write failed: {}", e))
            })
        }).map_err(|err| panic!("Download failed: {}", err)).map(move |_| {
            crossterm::cursor().goto(0, idx as u16 * 2).unwrap();
            crossterm::terminal().clear(ClearType::CurrentLine).unwrap();
            print!("Done!");
        })
    });
    tokio::run(collect(futures).map(|_| {}));
    crossterm::cursor().show().unwrap();
}
