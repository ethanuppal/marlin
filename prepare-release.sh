#!/usr/bin/env sh
# usage: /bin/sh prepare-release.sh

git --version || (echo "git is not installed" && exit 1)
cargo --version || (echo "cargo is not installed" && exit 1)

if [ -n "$(git status --porcelain)" ]; then
    echo "There are unstaged or staged changes. Please commit or stash them before switching branches."
    exit 1
fi

CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)

git fetch origin release-please--branches--main || exit 1
git checkout release-please--branches--main || exit 1
git reset --hard origin/release-please--branches--main || exit 1

cargo check || exit 1

git add . || exit 1
git status || exit 1
git commit --sign --signoff -m "chore: Trigger CI and update lockfile" || exit 1
git push origin release-please--branches--main || exit 1

git checkout "$CURRENT_BRANCH"
