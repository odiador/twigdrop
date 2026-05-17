# Contributing to Twigdrop

First off, thank you for considering contributing to Twigdrop! It's people like you that make Twigdrop such a great tool.

## Where do I go from here?

If you've noticed a bug or have a feature request, make sure to check our [Issues](https://github.com/your-org/twigdrop/issues) to see if someone else has already created a ticket. If not, go ahead and [make one](https://github.com/your-org/twigdrop/issues/new)!

## Fork & create a branch

If this is something you think you can fix, then [fork Twigdrop](https://github.com/your-org/twigdrop/fork) and create a branch with a descriptive name.

A good branch name would be (where issue #325 is the ticket you're working on):

```sh
git checkout -b 325-add-new-feature
```

## Get the test suite running

Make sure you're running the latest stable version of Rust.

```sh
cargo test
cargo fmt
cargo clippy
```

## Implement your fix or feature

At this point, you're ready to make your changes! Feel free to ask for help; everyone is a beginner at first.

## Make a Pull Request

At this point, you should switch back to your master branch and make sure it's up to date with Twigdrop's master branch:

```sh
git remote add upstream git@github.com:your-org/twigdrop.git
git checkout master
git pull upstream master
```

Then update your feature branch from your local copy of master, and push it!

```sh
git checkout 325-add-new-feature
git rebase master
git push --set-upstream origin 325-add-new-feature
```

Finally, go to GitHub and make a Pull Request!
