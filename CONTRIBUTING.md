# CONTRIBUTING

Any contribution is welcomed.

## Introduction

We globally follow the [Angular Convention](https://github.com/angular/angular/blob/master/CONTRIBUTING.md#-commit-message-format)
with some small customization

Each commit message consists of a header, a body, and a footer.

```html
<header>
<BLANK LINE>
<body>
<BLANK LINE>
<footer>
```

The header must answer to the following format:

```bash
<type>(<scope>): <short summary>
  │       │             │
  │       │             └─⫸ Summary in present tense. Not capitalized. No period at the end.
  │       │
  │       └─⫸ Commit Scope: Any scope that you want
  │
  └─⫸ Commit Type: build|ci|docs|feat|fix|perf|refactor|style|test|breaking
```

### Type

Must be one of the following:

* `breaking`: A breaking change
* `feat`: A new feature
* `fix`: A bug fix
* `build`: Changes that affect the build system or external dependencies (example scopes: gulp, broccoli, npm)
* `ci`: Changes to our CI configuration files and scripts (example scopes: Circle, BrowserStack, SauceLabs)
* `perf`: A code change that improves performance
* `refactor`: A code change that neither fixes a bug nor adds a feature
* `docs`: Documentation only changes
* `style`: Changes that do not affect the meaning of the code (white-space, formatting, missing semi-colons, etc)
* `test`: Adding missing tests or correcting existing tests

The following rules will be applied

| Commit message                 | Release type           |
|--------------------------------|------------------------|
| test: add unit test            | None                   |
| style: fix linter              | None                   |
| docs: update readme            | None                   |
| refactor: refacto main app     | Patch Release          |
| ci: update semantic            | Patch Release          |
| build: update build tooling    | Patch Release          |
| perf: fix perf issue on my app | Patch Release          |
| fix: fix memory leak           | Patch Release          |
| feat: add new cli option       | Minor Feature Release  |
| breaking: remove cli option    | Major Breaking Release |

### Scope

Anything you want to give some more details about the commit scope into the changelog.

### Subject

Please try to follow the following rules:

* use the imperative, present tense: "change" not "changed" nor "changes"
* don't capitalize first letter
* no dot (.) at the end

## Breaking changes

To release a breaking change you have two options

### Cleanest one

Do your commits with respect of [type](#type),

Then update your MR title to fit
`BREAKING CHANGE: my breaking change description`

You must have the following
setting activated in your repo

```text
* Merge commit
  Every merge creates a merge commit
```

### Simplest one

Get one of your commits with the `breaking` keyword

This will generate a `major` release
