---

title: "Contribute to Tinkerbench"
description: "Report a problem, suggest an improvement, test a map, improve the guides or contribute code."
---

Help make Tinkerbench more useful for Warcraft III map makers. You can report a confusing
field, test an editor change, improve a guide or contribute code. Documentation work does
not require a Warcraft III installation.

## Choose a place to start

- **Found a problem?** [Search existing issues](https://github.com/gabaudette/wc3-toolkit/issues)
  before [opening a new issue](https://github.com/gabaudette/wc3-toolkit/issues/new).
- **Have an idea?** Describe the map-making task, what gets in your way and the behavior you
  would like. For a larger change, discuss the scope in an issue before starting.
- **Want to work on the editor?** Follow [local development](./development/) to run the app
  and its checks.
- **Want to improve this site?** Follow [documentation contributions](./documentation/) to
  preview guides, field explanations and examples.

Small corrections to labels, explanations and examples are useful contributions too. The
[project goals](https://github.com/gabaudette/wc3-toolkit/blob/main/docs/pet-peeves.md) describe
the editor’s intended improvements.

## Report a useful bug

Include enough information for someone else to reproduce it:
1. Your Tinkerbench version, operating system and Warcraft III version, if relevant.
2. Whether you are using HD art, and whether the problem occurs with game data only or with
   an open map.
3. The object type, raw code and field involved, such as Footman `hfoo` and maximum life
   `uhpm`.
4. The steps you took, what you expected and what actually happened.
5. The exact error message and a screenshot when they help explain the problem.

Test on a copy of your map. If the issue needs a particular map, describe the smallest
reproduction you can make. Share only material you have permission to share; do not attach
Blizzard game scripts, models, textures, icons or data tables.

## Open a pull request

1. Create a branch for one focused change. Link the issue when there is one.
2. Follow the [engineering standards](https://github.com/gabaudette/wc3-toolkit/tree/main/docs/standards)
   and run the checks for the files you changed. Keep Windows and Linux working.
3. Update `CHANGELOG.md` for users and add one entry to `docs/devlog.md` for the pull request.
   Explain the behavior in the changelog and the implementation, reasoning and validation
   in the devlog. Add the PR number to its devlog entry once it is available.
4. Describe the problem, resulting behavior and validation in the PR. Include before/after
   screenshots for visual changes and state whether you tested in Warcraft III.

Use conventional commit prefixes such as `feat:`, `fix:` and `docs:`. Fix errors and warnings
before requesting review. The pre-push hook checks the changelog and devlog; the appropriate
CI workflows build and test the change.

The repository’s [CONTRIBUTING.md](https://github.com/gabaudette/wc3-toolkit/blob/main/CONTRIBUTING.md)
contains the full contribution rules, validation commands and release process.
