use std::{
    fs,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use indicatif::{ProgressBar, ProgressStyle};

use concat_reader::concat_path;
use git2::Signature;
use serde_json::Value;

fn main() {
    let repo_dir = PathBuf::from("/tmp/audit-repo");

    match fs::remove_dir_all(&repo_dir) {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
        r => r.expect("failed to remove repo dir"),
    }

    fs::create_dir_all(&repo_dir).unwrap();

    let repo = git2::Repository::init(&repo_dir).unwrap();
    let mut repo_index = repo.index().unwrap();

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

        let target_dir = repo_dir
            .join(
                audit_event["objectRef"]["namespace"]
                    .as_str()
                    .unwrap_or("cluster-scoped-resources"),
            )
            .join(audit_event["objectRef"]["resource"].as_str().unwrap());

        fs::create_dir_all(&target_dir).unwrap();

        let name = audit_event["objectRef"]["name"].as_str().unwrap();
        let is_status = audit_event["objectRef"]["subresource"]
            .as_str()
            .unwrap_or("")
            == "status";

        let mut request_object = audit_event["requestObject"].clone();
        if request_object.is_null() {
            // no requestObject means it was censored, create a fake one instead
            request_object = Value::Object(serde_json::Map::new());
        }

        let status_target = target_dir.join(format!("{}_status.yaml", name));
        let status_target_rel = status_target.strip_prefix("/tmp/audit-repo");
        let regular_target = target_dir.join(format!("{}.yaml", name));
        let regular_target_rel = regular_target.strip_prefix("/tmp/audit-repo");

        if let "update" = verb {
            if is_status {
                request_object["spec"] = serde_json::Value::Null;
            } else {
                request_object["status"] = serde_json::Value::Null;
            }
        }

        let request_object: serde_yaml::Value = serde_json::from_value(request_object).unwrap();
        let request_object = serde_yaml::to_string(&request_object).unwrap();

        if let "update" = verb {
            if is_status {
                fs::write(&status_target, &request_object).unwrap();
                repo_index.add_path(status_target_rel.unwrap()).unwrap();
            } else {
                fs::write(&regular_target, &request_object).unwrap();
                repo_index.add_path(regular_target_rel.unwrap()).unwrap();
            }
        } else {
            if fs::metadata(&regular_target).is_ok() || fs::metadata(&status_target).is_ok() {
                continue;
            }

            fs::write(&regular_target, &request_object).unwrap();
            repo_index.add_path(regular_target_rel.unwrap()).unwrap();

            fs::write(&status_target, &request_object).unwrap();
            repo_index.add_path(status_target_rel.unwrap()).unwrap();
        }

        let author_name = audit_event["user"]["username"].as_str().unwrap();
        let author_email = "operator@redhat.com";
        let author_date = audit_event["requestReceivedTimestamp"].as_str().unwrap();
        let author_date = chrono::DateTime::parse_from_rfc3339(author_date)
            .unwrap()
            .timestamp();

        let tree = repo_index.write_tree();
        let tree = repo.find_tree(tree.unwrap()).unwrap();

        let current_head = repo
            .head()
            .unwrap_or_else(|_| {
                repo.commit(
                    Some("HEAD"),
                    &Signature::new(author_name, author_email, &git2::Time::new(author_date, 0))
                        .unwrap(),
                    &Signature::new(author_name, author_email, &git2::Time::new(author_date, 0))
                        .unwrap(),
                    "Initial commit",
                    &tree,
                    &[],
                )
                .unwrap();
                repo.head().unwrap()
            })
            .peel_to_commit()
            .unwrap();

        let sig =
            Signature::new(author_name, author_email, &git2::Time::new(author_date, 0)).unwrap();
        let msg = format!(
            "{} {} {} in {} by {}",
            // "{} {} {} in {} by {}\n\n{}",
            verb.chars().next().unwrap().to_uppercase().to_string() + &verb[1..],
            audit_event["objectRef"]["resource"],
            audit_event["objectRef"]["name"],
            audit_event["objectRef"]["namespace"],
            audit_event["user"]["username"],
            // serde_yaml::to_string(&audit_event).unwrap(),
        );
        repo.commit(Some("HEAD"), &sig, &sig, &msg, &tree, &[&current_head])
            .unwrap();
    }
}
