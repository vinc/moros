#!/bin/sh
title="$(gh pr view --json title,number --template '- {{.title}} (#{{.number}})')"
sed -i.tmp "s|Unreleased|Unreleased\n$title|" CHANGELOG.md
rm CHANGELOG.md.tmp
