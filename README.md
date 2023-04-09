# Tempomat

A small CLI to facilitate creating Tempo worklogs for Jira issues.

## Installation

You can either build this utility from source with `cargo build` or use `cargo install tempomat --locked` to get the latest version from crates.io.

An AUR package might come in the future.

## Usage

First you must get all the required access tokens:

```sh
tempomat login --atlassian-instance <you_atlassian_instance>
```

#### Required accesses:

 - Jira: This access is needed because the Tempo API needs both the Atlassian Account and Jira issue ID, which can only be obtained from the Jira API.
 - Tempo: Should be self-explanatory

### Logging time

This tool will automatically detect the current Jira issue key you are working on by the curret branch name (eg. `feat/DV-3124` or `PROJ-30_fix_bugs`).
If the issue key cannot be detected from the current branch, you will have to pass the `-i` flag with the issue key.

Examples:
```sh
tempomat log 1h # Logs 1 hour to the current issue
tempomat log -m"Implement Bar" 30s # Logs 30 seconds to the current issue with a description
tempomat log -i PROJ-5 30m # Log 30 minutes to the issue PROJ-5
```

## TODO

This tool is not yet fully complete, watch the progress here: [TODO.md](TODO.md)
