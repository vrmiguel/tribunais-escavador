#![feature(try_blocks)]

use std::{env, fs, io::BufReader, path::Path};

use anyhow::{Context, Result};
use convert_case::{Case, Casing};
use serde_json::{json, Value};

struct Court {
    name: String,
    acronym: String,
}

struct Courts {
    inner: Vec<Court>,
}

impl Courts {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let file = fs::File::open(path)?;
        let buf_reader = BufReader::new(file);
        let json: Value = serde_json::from_reader(buf_reader)?;
        let courts: Option<Vec<Court>> = try {
            dbg!(json.get("items")?.is_array());
            let mut courts = Vec::with_capacity(212);
            let items = json.get("items")?.as_array()?;
            let one = &json!(1);

            for item in items {
                let map = match item.as_object() {
                    Some(map) => map,
                    None => {
                        println!("{item} is not a map");
                        continue;
                    }
                };

                if map.get("busca_documento")? == one {
                    let court = Court {
                        name: map.get("nome")?.as_str()?.to_string(),
                        acronym: map.get("sigla")?.as_str()?.to_string(),
                    };

                    courts.push(court);
                }
            }

            courts
        };

        let courts = courts.with_context(|| "failed to parse courts")?;
        dbg!(courts.len());

        Ok(Self { inner: courts })
    }

    pub fn display_postgres_enum(&self) {
        println!("CREATE TYPE kyc.courts AS enum(");
        for court in self.inner.iter() {
            println!("\t\"{}\"", court.acronym);
        }
        println!(");")
    }

    pub fn display_rust_enum(&self) {
        println!("pub enum Courts {{");
        for court in self.inner.iter() {
            // Doc-string
            println!("\t\\\\\\ {}", court.name);
            println!("\t{},", court.acronym.to_case(Case::Pascal));
        }
        println!("}}")
    }
}

fn main() -> Result<()> {
    let path = env::args_os()
        .nth(1)
        .with_context(|| "Filename not supplied")?;
    let courts = Courts::from_file(path)?;

    courts.display_postgres_enum();
    courts.display_rust_enum();

    Ok(())
}
