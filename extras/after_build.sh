#!/bin/bash

# Cribbed from an excellent guide by Steve Klabnik:
# http://www.steveklabnik.com/automatically_update_github_pages_with_travis_example/

set -o errexit -o nounset

RUSTC_VERSION=$(rustc --version)

if [ "$TRAVIS_BRANCH" != "master" ]
then
  echo "This commit was made against the $TRAVIS_BRANCH and not the master! No deploy!"
  exit 0
fi

if [[ ! "$RUSTC_VERSION" =~ "1.8.0" ]]
then
  echo "Wrong version of rustc: expected 1.8.0, got $RUSTC_VERSION"
  exit 0
fi

rev=$(git rev-parse --short HEAD)

# Build all the docs
cargo doc
echo "<meta http-equiv=\"refresh\" content=\"0;url=metrics_distributor/index.html\">" > target/doc/index.html

# Move into the documentation root
cd target/doc

git init
git config user.name "Dirk Gadsden"
git config user.email "dirk@esherido.com"

git remote add upstream "https://$GH_TOKEN@github.com/dirk/metrics_distributor.git"
git fetch upstream
git reset -q upstream/gh-pages

touch .

git add -A .
git commit -m "Auto-rebuild pages at ${rev}"
git push -q upstream HEAD:gh-pages
