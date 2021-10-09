- support environment variables for commands
- logging verbosity more than just debug?
- look at Strum for enums to reduce boilerplate code


// https://stackoverflow.com/questions/32555589/is-there-a-clean-way-to-have-a-global-mutable-state-in-a-rust-plugin
// https://stackoverflow.com/questions/61159698/update-re-initialize-a-var-defined-in-lazy-static

// TODO: Automatically update readme TOC
// Update CVS file? From Oxide - we automatically update a CSV file of all the RFDs along with their state, links, and other information in the repo for easy parsing.
// TODO: configuration
// TODO: output options
// config file
// env var DOCTAVIOUS_DEFAULT_OUTPUT - overrides config
// --output  - overrides env var and config
// TODO: RFD / ADR meta frontmatter
// Create ADR from RFD - essentially a link similar to linking ADRs to one another
// ADR generate should take in a template?
// How to specify just to inject into existing markdown?
// generate csv - create/update CSV of rfds
// generate file - given a custom template generate file
// generate toc - generate table and insert into existing file. could this be a template as well? why not?
// generate graph - generate graph

// Extract frontmatter and use that for ADR and RFD tables
// https://www.11ty.dev/docs/data-frontmatter-customize/
// https://crates.io/crates/gray_matter


// TODO: automatically update README(s) / CSVs
// or at the very least lint
// - pass over readme to verify that the links are properly formed and ADR/RFD numbers appropriately used in the RFD table.
// - iterate over every RFD and make sure that it's in the table in README.md and in a matching state and with sane metadata.
// - https://github.com/joyent/rfd/blob/master/tools/rfdlint

// TODO: we can prompt user if they try to init multiple times
// https://github.com/sharkdp/bat/blob/5ef35a10cf880c56b0e1c1ca7598ec742030eee1/src/bin/bat/config.rs#L17

// executable path
// https://github.com/rust-lang/rust-clippy/blob/master/src/main.rs#L120

// clippy dogfood
// https://github.com/rust-lang/rust-clippy/blob/master/src/main.rs#L132

// TODO: review https://github.com/simeg/eureka
// some good ideas here

// TODO: review https://github.com/jakedeichert/mask
// TODO: review https://github.com/sharkdp/bat/tree/master/src

// TODO: architecture diagrams as code
// If we do some sort of desktop app we should have preview function to see as you code

// TODO: Today I learned CLI
// https://github.com/danielecook/til-tool
// https://github.com/danielecook/til

// ADR: build ToC / readme / graph (dot/graphviz)
// RFC: build ToC / readme / graph
// Til: build ToC / readme

// serial number

// TODO: share structs between ADR and RFD

// TODO: ADR/RFD ToC vs TiL readme

// TODO: Generate CSV for ADRs and RFDs?

// TODO: better code organization
// maybe we have a markup module with can contain markdown and asciidoc modules
// common things can link in markup module
// such as common structs and traits
// can we move common methods into traits as default impl?
// example would be ToC
// what things would impl the traits?
// Where should TiL live? markup as well? Can override default impl for example til toc/readme

// implement ADR / RFD reserve command
// 1. get latest number
// 2. verify it doesnt exist
// git branch -rl *0042
// 3. checkout
// $ git checkout -b 0042
// 4. create the placeholder
// 5. Push your RFD branch remotely
// $ git add rfd/0042/README.md
// $ git commit -m '0042: Adding placeholder for RFD <Title>'
// $ git push origin 0042

// add command to automatically update README on master as a git hook
// After your branch is pushed, the table in the README on the master branch will update automatically with the new RFD.
// git hook should be able to curl for hook. see python pre-commit hook. something related to gitlab

integrate tera in our templates

integrate unified/remark to build/edit AST while we work on marcup

add #[non_exhaustive] to enums. Add Other or something

