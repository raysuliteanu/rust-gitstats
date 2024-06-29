use std::error::Error;

use anyhow::Result;
use git2::Repository;

fn main() -> Result<(), Box<dyn Error>> {
    // TODO: optional path from CLI args
    let path = ".";
    let repo = Repository::open(path)?;
    let mut revwalk = repo.revwalk()?;

    revwalk.push_head()?;

    macro_rules! filter_try {
        ($e:expr) => {
            match $e {
                Ok(t) => t,
                Err(e) => return Some(Err(e)),
            }
        };
    }
    let revwalk = revwalk
        .filter_map(|id| {
            let id = filter_try!(id);
            let commit = filter_try!(repo.find_commit(id));
            //        let parents = commit.parents().len();
            Some(Ok(commit))
        })
        .take(10);

    revwalk.for_each(|c| println!("{:?}", c));

    Ok(())
}
