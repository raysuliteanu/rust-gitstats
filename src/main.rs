use std::collections::HashMap;
use std::error::Error;
use std::hash::Hash;
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use git2::{BranchType, Repository, Status};

#[derive(Default, Debug)]
struct GitStats {
    commits_by_contributor: HashMap<String, usize>,
    local_branches: usize,
    remote_branches: usize,
    _tags: usize,
    files_by_status: HashMap<StatusMapping, usize>,
}
// see git_status_t from git2::StatusEntry; it's not hashable so need to map
#[derive(Hash, Ord, Eq, Debug, PartialOrd, PartialEq)]
enum StatusMapping {
    Current ,
    IndexNew ,
    IndexModified ,
    IndexDeleted ,
    IndexRenamed ,
    IndexTypeChange ,
    WtNew ,
    WtModified ,
    WtDeleted ,
    WtTypeChange ,
    WtRenamed ,
    WtUnreadable ,
    Ignored ,
    Conflicted ,
}

// todo: this won't work :P ... rev can be in multiple states at the same time;
// they're just bit flags and so status could be 'WT_MODIFIED|CONFLICTED' or some such
impl From<Status> for StatusMapping {
    fn from(status: Status) -> Self {
        match status {
            Status::CURRENT => StatusMapping::Current,
            Status::INDEX_NEW => StatusMapping::IndexNew,
            Status::INDEX_MODIFIED => StatusMapping::IndexModified,
            Status::INDEX_DELETED => StatusMapping::IndexDeleted,
            Status::INDEX_RENAMED => StatusMapping::IndexRenamed,
            Status::INDEX_TYPECHANGE => StatusMapping::IndexTypeChange,
            Status::WT_NEW => StatusMapping::WtNew,
            Status::WT_MODIFIED => StatusMapping::WtModified,
            Status::WT_DELETED => StatusMapping::WtDeleted,
            Status::WT_RENAMED => StatusMapping::WtRenamed,
            Status::WT_TYPECHANGE => StatusMapping::WtTypeChange,
            Status::IGNORED => StatusMapping::Ignored,
            Status::CONFLICTED => StatusMapping::Conflicted,
            _ => {
                eprintln!("unexpected revision status {:?}", status);
                StatusMapping::WtUnreadable
            }
        }
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
    // TODO: optional path from CLI args
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

    let statuses = repo.statuses(None)?;
    statuses
        .iter()
        .map(|status| {
            let s = StatusMapping::from(status.status());
            let value = stats.files_by_status.entry(s).or_insert(0);
            *value += 1;
        })
        .count();

    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;

    revwalk
        .filter_map(|id| id.ok())
        .filter_map(|commit_id| repo.find_commit(commit_id).ok())
        .map(|c| c.committer().name().unwrap().to_string())
        .map(|committer_name| {
            let value = stats
                .commits_by_contributor
                .entry(committer_name)
                .or_insert(0);
            *value += 1;
        })
        .count();

    println!("{:?}", stats);
    Ok(())
}
