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
extern crate itertools;

use std::path::Path;
use std::collections::{HashSet, LinkedList};
use std::hash::{Hash, Hasher};
use std::fmt::{Display, Formatter, Error};
use itertools::Itertools;

use glob::glob;
use xdg_basedir as xdg;
use time::{Duration, Timespec};
use rusqlite::SqliteConnection;

#[derive(Debug, Eq)]
struct Fact {
    activity: String,
    start_time: Timespec,
    end_time: Option<Timespec>,
    duration: Option<Duration>,
    tags: HashSet<String>,
    description: Option<String>
}

trait ClearTabs {
	fn clear_tabs(&self) -> Self;
}

impl Fact {
    fn new(activity: String, start_time: Timespec, end_time: Option<Timespec>, tags: HashSet<String>, description: Option<String>) -> Fact {
        Fact {
            activity: activity,
            start_time: start_time,
            end_time: end_time,
            duration: match end_time {
                Some(end) => { Some(end - start_time) },
                None => None
            },
            tags: tags,
            description: description
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

impl ClearTabs for String {
	fn clear_tabs(&self) -> String {
		self.replace("\t", "        ").replace("\r", "").replace("\n", "\\\\")
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

        try!(f.write_str("\t"));

        let mut tag_names = LinkedList::<String>::new();
        for tag_name in self.tags.clone() {
            tag_names.push_back(tag_name.replace(";", ","));
        }

        try!(tag_names.iter().join(";").fmt(f));

        try!(f.write_str("\t"));

        match self.description {
            Some(ref str) => { try!(str.clear_tabs().fmt(f)); },
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
        SELECT facts.id, activities.name, facts.start_time, facts.end_time, facts.description
        FROM facts
        JOIN activities ON activities.id = facts.activity_id").unwrap();

    let mut tag_stmt = conn.prepare("
        SELECT tags.name
        FROM tags
        JOIN fact_tags ON fact_tags.tag_id = tags.id
        WHERE fact_tags.fact_id = ?
        ").unwrap();

    let iter = stmt.query_map(&[], |row| {
        let fact_id: i32 = row.get(0);
        let name = row.get(1);
        let start_time = row.get(2);
        let end_time = row.get(3);
        let description = row.get(4);

        let mut tags = HashSet::<String>::new();

        let tag_iter = tag_stmt.query_map(&[&fact_id], |tag_row| {
            tag_row.get(0)
        }).unwrap();

        for tag in tag_iter {
            match tag {
                 Ok(s) => {
                     let tag_name: String = s;
                     tags.insert(tag_name.clear_tabs());
                 },
                 _ => {}
            }
        }

        Fact::new(name, start_time, end_time, tags, description)
    }).unwrap();

    for fact in iter {
        facts.insert(fact.unwrap());
    }
}
