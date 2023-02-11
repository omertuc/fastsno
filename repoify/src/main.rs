use std::{
    collections::HashMap,
    fs,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use git_hash::Kind;
use git_object::{bstr::BString, tree::{self, EntryMode}};
use git_odb::Write;
use indicatif::{ProgressBar, ProgressStyle};

use concat_reader::concat_path;
use serde_json::Value;

struct Cluster {
    states: Vec<git_object::Commit>,
}

fn get_tree(tree: git_object::Tree, name: &str) -> &git_object::Tree {
    for entry in tree.entries.binary_search_by_key(name, |entry| entry.name) {
        match entry.mode {
            EntryMode::Tree => {
                return entry;
            }
            _ => panic!("expected tree"),
        }
    }
}

fn main() {
    let repo_dir = PathBuf::from("/tmp/audit-repo");

    match fs::remove_dir_all(&repo_dir) {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
        r => r.expect("failed to remove repo dir"),
    }

    fs::create_dir_all(&repo_dir).unwrap();

    let audit_files = glob::glob("../audit/*audit*")
        .unwrap()
        .into_iter()
        .map(|x| x.unwrap());

    let f = concat_path(audit_files);
    let mut buffered = BufReader::new(f);

    let total_file_size = glob::glob("../audit/*audit*")
        .unwrap()
        .into_iter()
        .map(|x| x.unwrap())
        .map(|x| fs::metadata(x).unwrap().len())
        .sum::<u64>();

    let bar = ProgressBar::new(total_file_size);
    bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {bytes}/{total_bytes} {bytes_per_sec}",
        )
        .unwrap()
        .progress_chars("##-"),
    );

    let mut root_tree = git_object::Tree::empty();

    loop {
        let mut line = String::new();
        let r = buffered.read_line(&mut line).unwrap();
        if r == 0 {
            bar.finish();
            return;
        }
        bar.inc(line.len() as u64);

        let audit_event: serde_json::Value = serde_json::from_str(&line).unwrap();
        if audit_event["objectRef"]["name"]
            .as_str()
            .unwrap_or("")
            .is_empty()
        {
            continue;
        }

        let verb = audit_event["verb"].as_str().unwrap();
        match verb {
            "update" | "create" => {}
            _ => continue,
        }

        let namespace = audit_event["objectRef"]["namespace"]
            .as_str()
            .unwrap_or("cluster-scoped-resources");

        let kind = audit_event["objectRef"]["resource"].as_str().unwrap();

        root_tree.

        // fs::create_dir_all(&target_dir).unwrap();

        // let name = audit_event["objectRef"]["name"].as_str().unwrap();
        // let is_status = audit_event["objectRef"]["subresource"]
        //     .as_str()
        //     .unwrap_or("")
        //     == "status";

        // let mut request_object = audit_event["requestObject"].clone();
        // if request_object.is_null() {
        //     // no requestObject means it was censored, create a fake one instead
        //     request_object = Value::Object(serde_json::Map::new());
        // }

        // let status_target = target_dir.join(format!("{}_status.yaml", name));
        // let status_target_rel = status_target.strip_prefix("/tmp/audit-repo");

        // let regular_target = target_dir.join(format!("{}.yaml", name));
        // let regular_target_rel = regular_target.strip_prefix("/tmp/audit-repo");

        // if let "update" = verb {
        //     if is_status {
        //         request_object["spec"] = serde_json::Value::Null;
        //     } else {
        //         request_object["status"] = serde_json::Value::Null;
        //     }
        // }

        // let request_object: serde_yaml::Value = serde_json::from_value(request_object).unwrap();
        // let request_object = serde_yaml::to_string(&request_object).unwrap();

        // if let "update" = verb {
        //     if is_status {
        //         repo.write_blob(request_object.as_bytes()).unwrap();
        //     } else {
        //         fs::write(&regular_target, &request_object).unwrap();
        //     }
        // } else {
        //     fs::write(&regular_target, &request_object).unwrap();

        //     fs::write(&status_target, &request_object).unwrap();
        // }

        // let author_name = audit_event["user"]["username"].as_str().unwrap();
        // let author_email = "operator@redhat.com";
        // let author_date = audit_event["requestReceivedTimestamp"].as_str().unwrap();
        // let author_date = chrono::DateTime::parse_from_rfc3339(author_date)
        //     .unwrap()
        //     .timestamp();

        // let current_head = repo
        //     .head()
        //     .unwrap_or_else(|_| {
        //         repo.commit(
        //             Some("HEAD"),
        //             &Signature::new(author_name, author_email, &git2::Time::new(author_date, 0))
        //                 .unwrap(),
        //             &Signature::new(author_name, author_email, &git2::Time::new(author_date, 0))
        //                 .unwrap(),
        //             "Initial commit",
        //             &tree,
        //             &[],
        //         )
        //         .unwrap();
        //         repo.head().unwrap()
        //     })
        //     .peel_to_commit()
        //     .unwrap();

        // let sig =
        //     Signature::new(author_name, author_email, &git2::Time::new(author_date, 0)).unwrap();
        // let msg = format!(
        //     "{} {} {} in {} by {}",
        //     // "{} {} {} in {} by {}\n\n{}",
        //     verb.chars().next().unwrap().to_uppercase().to_string() + &verb[1..],
        //     audit_event["objectRef"]["resource"],
        //     audit_event["objectRef"]["name"],
        //     audit_event["objectRef"]["namespace"],
        //     audit_event["user"]["username"],
        //     // serde_yaml::to_string(&audit_event).unwrap(),
        // );
        // repo.commit(Some("HEAD"), &sig, &sig, &msg, &tree, &[&current_head])
        //     .unwrap();
    }
}
