# doctavious-cli

Command Line utility for building docs and interacting with Doctavious

## Build

```
cargo build
```


## ADR 


## RFC / RDF

https://oxide.computer/blog/rfd-1-requests-for-discussion/#changing-the-rfd-process

## Today I Learned

## Changelog

Support conventional commits as well as custom 

### Convention Commits

Structure
```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

Conventional Commits includes the following
```
fix: a commit of the type fix patches a bug in your codebase (this correlates with PATCH in Semantic Versioning).
feat: a commit of the type feat introduces a new feature to the codebase (this correlates with MINOR in Semantic Versioning).
BREAKING CHANGE: a commit that has a footer BREAKING CHANGE:, or appends a ! after the type/scope, introduces a breaking API change (correlating with MAJOR in Semantic Versioning). A BREAKING CHANGE can be part of commits of any type.
types other than fix: and feat: are allowed, for example @commitlint/config-conventional (based on the the Angular convention) recommends build:, chore:, ci:, docs:, style:, refactor:, perf:, test:, and others.
footers other than BREAKING CHANGE: <description> may be provided and follow a convention similar to git trailer format.
```

We could also have a release sub command that runs a script to increment version based on the above

### Custom

custom example would be something like cockroachdb. 
I like cockroachdb's release note and release justification style

## Git Hooks 

Git hooks manager

How do we want to share / init / etc?

I dont want to force people to have cargo installed to use. 
I much rather have than curl init script to install doctavious as an executable.

bash script to download doctavious if necessary then copy files to repo and do any additional setup.

rusy-hook copies files to `.git/hooks/`.
Other options from https://stackoverflow.com/questions/3462955/putting-git-hooks-into-repository

- symlink
- http://git-scm.com/docs/git-init#_template_directory
- git config --local core.hooksPath .githooks/



## Snippets



## Graphviz

adr graph | dot -Tpng -o adr.png

rfd graph | dot -Tpng -o rfd.png




## Commands

```shell
./target/debug/doctavious-cli help
./target/debug/doctavious-cli adr init -d docs/adr -e md
./target/debug/doctavious-cli adr init -d docs/adr -e md -s nested
./target/debug/doctavious-cli adr init -d docs/adr -e adoc
./target/debug/doctavious-cli adr init -d docs/adr -e adoc -s nested
./target/debug/doctavious-cli til init
```


## Ideas:
- rendering/local preview
    - see how [Aurelius](https://github.com/euclio/aurelius) does live previewing of markdown as html
- deploying
- generating architecture graphs
- graphs as code
- Roam style knowledge graph
- git changelogs / releases
- snippets
