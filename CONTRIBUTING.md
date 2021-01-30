# Contribution Guide

> Majority taken from Sindre Sorhus' [.github](https://github.com/sindresorhus/.github) repo

Be nice and respect the maintainers' time. ❤️ Be sure to read our [Code of Conduct](CODE_OF_CONDUCT.md) if you are new to this project.

Please read this whole thing. Most of this applies to any repo on GitHub. 🙏

If you already have, or are short on time, [skip to the vercel runtime development guidance](#vercel-runtime-development)

## Contribute

Code is not the only thing you can contribute. We truly appreciate contributions in the form of:

- Fixing typos.
- Improving docs.
- Triaging [issues](https://github.com/mike-engel/now-rust/issues).
- Reviewing [pull requests](https://github.com/mike-engel/now-rust/pulls).
- Sharing your opinion on issues.

## Issues

- Before opening a new issue, look for existing issues (even closed ones).
- [Don't needlessly bump issues.](https://blog.sindresorhus.com/issue-bumping-e3b9740e2a0)
- If you're reporting a bug, include as much information as possible. Ideally, include a test case that reproduces the bug. For example, a [Runkit](https://runkit.com) or [repl.it](https://repl.it) playground. Even better, submit a pull request with a [failing test](https://github.com/avajs/ava/blob/master/docs/01-writing-tests.md#failing-tests).

## Pull requests

### Prerequisites

- If the changes are large or breaking, open an issue discussing it first.
- Don't open a pull request if you don't plan to see it through. Maintainers waste a lot of time giving feedback on pull requests that eventually go stale.
- Don't do unrelated changes.
- Adhere to the existing code style.
- If relevant, add tests, check for typos, and add docs and types.
- Don't add editor-specific metafiles. Those should be added to your own [global gitignore](https://gist.github.com/subfuzion/db7f57fff2fb6998a16c).
- Double-check your contribution by going over the diff of your changes before submitting a pull request. It's a good way to catch bugs/typos and find ways to improve the code.
- Do the pull request from a new branch. Never the default branch (`main`/`master`).

### Submission

- Give the pull request a clear title and description. It's up to you to convince the maintainers why your changes should be merged.
- If the pull request fixes an issue, reference it in the pull request description using the syntax `Fixes #123`.
- Make sure the “Allow edits from maintainers” checkbox is checked. That way I can make certain minor changes myself, allowing your pull request to be merged sooner.

### Improving

- Push new commits when doing changes to the pull request. Don't squash as it makes it hard to see what changed since the last review. I will squash when merging.
- It's better to present solutions than asking questions.
- Review the pull request diff after each new commit. It's better that you catch mistakes early than the maintainers pointing it out and having to go back and forth.
- Be patient. Maintainers often have a lot of pull requests to review. Feel free to bump the pull request if you haven't received a reply in a couple of weeks.
- And most importantly, have fun! 👌🎉

## Vercel runtime development

Resources:

- Vercel master repository: [Runtime Developer Reference](https://github.com/vercel/vercel/blob/master/DEVELOPING_A_RUNTIME.md)
- Vercel docs: [Developing your own runtime](https://vercel.com/docs/runtimes#advanced-usage/developing-your-own-runtime)
