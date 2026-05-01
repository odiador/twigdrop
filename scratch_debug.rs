mod git;
mod models;

fn main() {
    let path = ".";
    let branches = git::build_branches(path);
    for b in branches {
        println!("Branch: {}, Status: {:?}", b.name, b.status);
    }
}

// Dummy run_git or just use the one in git.rs?
// I'll just compile the whole thing.
