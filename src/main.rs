use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

use anyhow::Result;
use git2::Repository;

struct Person {
    name: String,
    email: String,
    commits: usize,
}

impl Person {
    fn new(name: String, email: String, commits: usize) -> Self {
        Person {
            name,
            email,
            commits,
        }
    }
}

impl Display for Person {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Name: {} ({}) Commits: {}", self.name, self.email, self.commits))
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // TODO: optional path from CLI args
    let path = ".";
    let repo = Repository::open(path)?;
    let mut revwalk = repo.revwalk()?;

    revwalk.push_head()?;

    let mut people: HashMap<String, Person> = HashMap::new();

    let _commits: Vec<_> = revwalk
        .filter_map(|id| id.ok())
        .filter_map(|commit_id| repo.find_commit(commit_id).ok())
        .map(|commit| {
            let signature = commit.committer();
            let committer_name = signature.name().unwrap();
            let committer_email = signature.email().unwrap();
            let entry = people.entry(committer_name.to_string())
                .or_insert(Person::new(committer_name.to_string(), committer_email.to_string(), 0));
            entry.commits += 1;
        })
        .collect();

    people.into_values()
        .for_each(|person| println!("{person}"));

    Ok(())
}
