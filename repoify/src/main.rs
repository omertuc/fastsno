use std::{
    fs,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use indicatif::{ProgressBar, ProgressStyle};

use concat_reader::concat_path;
use git2::Signature;

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

    // glob audit files
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

        let x: serde_json::Value = serde_json::from_str(&line).unwrap();
        if x["objectRef"]["name"].as_str().unwrap_or("").is_empty() {
            continue;
        }

        if x["verb"].as_str().unwrap_or("") != "update" {
            continue;
        }

        let target_dir = repo_dir
            .join(
                x["objectRef"]["namespace"]
                    .as_str()
                    .unwrap_or("cluster-scoped-resources"),
            )
            .join(x["objectRef"]["resource"].as_str().unwrap());

        fs::create_dir_all(&target_dir).unwrap();

        let name = x["objectRef"]["name"].as_str().unwrap();
        let is_status = x["objectRef"]["subresource"].as_str().unwrap_or("") == "status";

        let target_name = if is_status {
            target_dir.join(format!("{}_status.yaml", name))
        } else {
            target_dir.join(format!("{}.yaml", name))
        };

        let mut request_object = x["requestObject"].clone();

        if request_object.is_null() {
            continue;
        }

        if is_status {
            request_object["spec"] = serde_json::Value::Null;
        } else {
            request_object["status"] = serde_json::Value::Null;
        }

        let request_object: serde_yaml::Value = serde_json::from_value(request_object).unwrap();
        let request_object = serde_yaml::to_string(&request_object).unwrap();

        fs::write(&target_name, request_object).unwrap();

        let author_name = x["user"]["username"].as_str().unwrap();
        let author_email = "operator@redhat.com";
        let author_date = x["requestReceivedTimestamp"].as_str().unwrap();
        let author_date = chrono::DateTime::parse_from_rfc3339(author_date)
            .unwrap()
            .timestamp();

        repo_index
            .add_path(target_name.strip_prefix("/tmp/audit-repo").unwrap())
            .unwrap();
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
            "Update {} {} in {} by {}\n{}",
            x["objectRef"]["resource"],
            x["objectRef"]["name"],
            x["objectRef"]["namespace"],
            x["user"]["username"],
            line,
        );
        repo.commit(Some("HEAD"), &sig, &sig, &msg, &tree, &[&current_head])
            .unwrap();
    }
}
