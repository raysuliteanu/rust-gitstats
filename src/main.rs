use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use git2::{BranchType, Repository};

#[derive(Default, Debug)]
struct GitStats {
    commits_by_contributor: HashMap<String, usize>,
    local_branches: usize,
    remote_branches: usize,
    tags: usize,
    _files: usize,
}

impl Display for GitStats {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut commits = String::new();
        self.commits_by_contributor
            .iter()
            .for_each(|e| commits.push_str(format!("{}: {}\n", e.0, e.1).as_str()));

        f.write_fmt(format_args!(
            "{commits}\nBranches:\n\tLocal: {}\n\tRemote: {}\ntags: {}",
            self.local_branches, self.remote_branches, self.tags
        ))
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Options {
    /// The path to the Git repo
    #[arg(short, long, value_name = "DIR", default_value = ".")]
    repo_dir: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let options = Options::parse();

    let path = options.repo_dir;
    let repo = Repository::open(path)?;
    println!("Repository state: {:?}", repo.state());

    let mut stats = GitStats::default();

    let branches = repo.branches(None)?;
    branches
        .filter_map(|res| res.ok())
        .map(|(_, branch_type)| match branch_type {
            BranchType::Local => stats.local_branches += 1,
            BranchType::Remote => stats.remote_branches += 1,
        })
        .count();

    stats.tags = repo.tag_names(None).map_or(0, |t| t.len());

    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;

    revwalk
        .filter_map(|id| id.ok())
        .filter_map(|commit_id| repo.find_commit(commit_id).ok())
        .map(|c| {
            /*if let Ok(tree) = c.tree() {

            }*/
            c.committer().name().unwrap().to_string()
        })
        .map(|committer_name| {
            let value = stats
                .commits_by_contributor
                .entry(committer_name)
                .or_insert(0);
            *value += 1;
        })
        .count();

    println!("{stats}");
    Ok(())
}
