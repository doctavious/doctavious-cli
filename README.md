# doctavious-cli

Command Line utility for building docs and interacting with Doctavious

## Build

```
cargo build
```


## ADR 


## RFC / RDF

https://oxide.computer/blog/rfd-1-requests-for-discussion/#changing-the-rfd-process

examples 
- https://github.com/vectordotdev/vector/tree/master/rfcs
- https://github.com/TritonDataCenter/rfd
- https://github.com/apache/couchdb-documentation/tree/main/rfcs

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


https://github.com/crate-ci/committed
Enforce commit standards, whether for:
- Readability, especially in logs
- Consistent styling
- Compatibility with programmatic processing
.pre-commit 

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

if I plan to split out changelog / releases name should be related to J. Jonah Jameson publisher of the Daily Bugle. Thought process went from "ship it" -> "release it" -> "publish" -> "publisher". "shipit" crate was already take.

i'm still not sure if it makes sense to keep changelogs and releases together.

https://github.com/gembaadvantage/uplift has the following 
- bump
- changelog
- tag 
- release (combines the above)

has support for pre-release suffix for tag, bump, release commands

for releases need a way to identify starting commit - likely candidate is tag

what about MRs? is there a way to bump version numbers when merging where you arent tagging?


generate Fig completions for clap based CLIs https://github.com/clap-rs/clap/tree/24a2f3a90153bbb7a7c362818aef8149f7a01722/clap_complete_fig



cockroachdb x-anchor-telemetry github tag which means there is code that points to that issue

line width
80 for comments
120 for code
