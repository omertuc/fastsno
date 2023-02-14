# Fast SNO

A collection of scripts tools to analyze / improve SNO installation time, as a proof of concept

# Prerequisites

* Only applicable for the bootstrap-in-place SNO installation
* Before running most scripts here, set your `KUBECONFIG` correctly
* Set `SSH_IDENTITIY_FILE` to the path of your private key first (defaults to `../bootstrap-in-place-poc/ssh-key/key`)
* To get audit logs during bootstrap, run `./enable_bootstrap_audit.sh` very early during the bootstrapping process

# Manifests

The `./manifests` directory contains some useful installation manifests

* `0000_00_openshift-*-namespace.yaml` - Create important namespaces early. This speeds up the installation process.
* `audit.yaml` - Enable request body audit logging for post-bootstrap API server (complements `./enable_bootstrap_audit.sh`)

# Audit

`./downloads_audit.sh` is a script to download all audit logs from your
cluster. Some of the scripts here that analyze audit logs expect this to be
done prior to running them

# Timeline

This script converts audit logs into an HTML page that displays an interactive
timeline of interesting things of your choice

e.g. run `./graph.sh jq/clusteroperators.jq jq/resources.jq` to generate a
timeline containing both the transitions of `clusteroperator`s and the creation
of cluster resources

To actually view the timeline, use `./display_timeline.sh` or start a Python HTTP
server, due to CORS limitations

# Repoify

This tool converts audit logs into a git repo with directories corresponding to
namespaces/resource types, files within the directories are resources, file
content is the resource YAMLs, commits are resource creation/updates, commit authors
are the service accounts which made the change, commit timestamps are accurate to when
the change request was made.

This allows you to `git log` and `git blame` resources within the cluster to
see when they were modified, by who, when and what changes were made to them
exactly.

It also allows you to `git checkout` to see the state of the cluster at a
particular point in time.

- [ ] TODO: Resources created too early don't show up because they're not in the audit
logs, merge audit logs with must-gather logs to have the full picture

- [ ] TODO: Handle resource deletion

# Record Duration 

Run `./record_duration.sh` early on during installation to record how long the installation
took overall

