#![feature(await_macro, async_await, futures_api)]

#[macro_use]
extern crate tokio;

use std::io::{self, Write};
use rustop::opts;
use std::fs::File;

use hyper::Client;
use hyper::rt::Stream;
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

    tokio::run_async(async move {
        for (idx, url) in list.into_iter().enumerate() {
            let client = client.clone();
            tokio::spawn_async(async move {
                let uri = url.0.parse().expect("Failed parsing URL");
                let response = await!(client.get(uri)).expect("Failed connecting to server");
                crossterm::cursor().goto(0, idx as u16 * 2).unwrap();
                io::stdout().write_fmt(format_args!("Response for {}: {}", url.0, response.status())).unwrap();
                let total = response.headers().typed_get::<ContentLength>().expect("Response didn't contain a content length").0 as usize;
                let mut loaded: usize = 0;

                let mut body = response.into_body();

                while let Ok((Some(chunk), newbody)) = await!(body.into_future()) {
                    body = newbody;
                    let len = chunk.into_bytes().len();
                    loaded += len;
                    let percent = (100 * loaded) / total;
                    crossterm::cursor().goto(0, idx as u16 * 2).unwrap();
                    crossterm::terminal().clear(ClearType::CurrentLine).unwrap();
                    io::stdout().write_all("X".repeat(percent).as_bytes()).unwrap();
                }
                crossterm::cursor().goto(0, idx as u16 * 2).unwrap();
                crossterm::terminal().clear(ClearType::CurrentLine).unwrap();
                io::stdout().write_all("Done!".as_bytes()).unwrap();
             });
        }
    });

    crossterm::cursor().show().unwrap();
}
