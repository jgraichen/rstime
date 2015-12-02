/*
 *  Copyright (C) 2015 Jan Graichen <jgraichen@altimos.de>
 *
 *  This program is free software; you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation; either version 2 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License along
 *  with this program; if not, write to the Free Software Foundation, Inc.,
 *  51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
 */
extern crate xdg_basedir;
extern crate rusqlite;
extern crate time;
extern crate glob;

use std::path::Path;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::fmt::{Display, Formatter, Error};

use glob::glob;
use xdg_basedir as xdg;
use time::{Duration, Timespec};
use rusqlite::SqliteConnection;

#[derive(Debug, Eq)]
struct Fact {
    activity: String,
    start_time: Timespec,
    end_time: Option<Timespec>,
    duration: Option<Duration>
}

impl Fact {
    fn new(activity: String, start_time: Timespec, end_time: Option<Timespec>) -> Fact {
        Fact {
            activity: activity,
            start_time: start_time,
            end_time: end_time,
            duration: match end_time {
                Some(end) => { Some(end - start_time) },
                None => None
            }
        }
    }
}

impl PartialEq for Fact {
    fn eq(&self, other: &Fact) -> bool {
        self.activity == other.activity && self.start_time == other.start_time
    }
}

impl Hash for Fact {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.activity.hash(state);
        self.start_time.sec.hash(state);
    }
}

impl Display for Fact {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        try!(self.activity.fmt(f));
        try!(f.write_str("\t"));

        try!(time::at(self.start_time).rfc3339().fmt(f));
        try!(f.write_str("\t"));

        match self.end_time {
            Some(tm) => { try!(time::at(tm).rfc3339().fmt(f)); },
            None => {}
        };

        try!(f.write_str("\t"));

        match self.duration {
            Some(tm) => { try!(tm.num_minutes().fmt(f)); },
            None => {}
        };

        Ok(())
    }
}

fn main() {
    let mut facts = HashSet::new();
    let     paths = get_db_paths();

    for path in paths.iter() {
        for entry in glob(path).unwrap() {
            match entry {
                Ok(path) => load_facts(&mut facts, &path),
                Err(e) => println!("{:?}", e)
            }
        }
    }

    for fact in facts.iter() {
        println!("{}", *fact);
    }
}

fn get_db_paths() -> Vec<String> {
    let mut args : Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        args.remove(0);
        return args;
    } else {
        let path   = xdg::get_data_home().unwrap().join("hamster-applet/hamster.db*");
        let string = String::from(path.to_str().unwrap());

        let mut paths = Vec::new();
        paths.push(string);
        return paths;
    }
}

fn load_facts(facts: &mut HashSet<Fact>, path: &Path) {
    println!("Loading {:?}...", path);

    let conn = SqliteConnection::open(&path).unwrap();
    let mut stmt = conn.prepare("
        SELECT activities.name, facts.start_time, facts.end_time
        FROM facts
        JOIN activities ON activities.id = facts.activity_id").unwrap();

    let iter = stmt.query_map(&[], |row| {
        Fact::new(row.get(0), row.get(1), row.get(2))
    }).unwrap();

    for fact in iter {
        facts.insert(fact.unwrap());
    }
}
