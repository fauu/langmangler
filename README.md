# langmangler

> Create words in a fictional language by transforming words from a real language

Langmangler takes a list of strings and transforms them according to a supplied list of simple rules. It was designed to produce passably consistent large sets of fictional names of people and places with the least amount of effort, leveraging existing data.

**Example input:**<br>
`Yamagata Fukushima Saitama` + [`example-japanese-rules.txt`](#rules-file-format)<br>
**Example output:**<br>
`Limegite Efoququema Sliītime`

This program was created for [Monmonde](https://github.com/fauu/Monmonde/). That project’s repository contains [more example outputs](https://github.com/fauu/Monmonde/tree/master/sim/data/names) of langmangler (human names from fictional countries).

## Table of contents

- [Getting started](#getting-started)
- [Usage](#usage)
- [Rules file format](#rules-file-format)
- [License](#license)

## Getting started

#### Requirements

- Rust stable

#### Option 1. Direct installation

```
$ cargo install --git "https://github.com/fauu/langmangler"
```

#### Option 2. Clone and run

```
$ git clone "https://github.com/fauu/langmangler"
$ cd langmangler
$ cargo run -- [program parameters]
```

## Usage

```
$ langmangler

langmangler 0.2.0
Create words in a fictional language by transforming words from a real language.

USAGE:
    langmangler [FLAGS] [OPTIONS] --rules <rules>

FLAGS:
    -h, --help       Prints help information
    -o, --compare    Prints original strings for comparison
    -V, --version    Prints version information

OPTIONS:
    -x, --reject <reject>    Compare outputs with inputs and reject those that pass the specified
                             check [possible values: Unchanged, AsciiUnchanged]
    -r, --rules <rules>      Path to the rules file
```

Example:

```
$ echo -e "Yamagata\nFukushima\nSaitama" | langmangler -or examples/example-japanese-rules.txt

Limegite (Yamagata)
Efoququema (Fukushima)
Sliītime (Saitama)
```

## Rules file format

Below are the contents of the [example rules file](/examples/example-japanese-rules.txt), designed for Japanese input:

```
@ReturningNonSegmentedRegex
-.*/# Drop dashes and whatever follows them
dos /

@ReturningSegmentedRegex
(.)a(.)a/${1}i${2}e

@NonReturningNonSegmentedSimple
c/s
k/q
ō/o

@NonReturningNonSegmentedSimple
fu/efo
mura/maro
shi/que
na/in
no/on
zu/qan
qi/i
l/y
y/l
ri/re
ji/di
tsu/tarq
qhi/qiu
wa/wo
ut/oq
su/sla
sa/sli
to/uar
ru/rus
qur/qir
sh/soq
hi/hoq
ra/res
mi/mes
ai/ia

@ReturningNonSegmentedRegex
ii/iī
```

The headings, starting with the `@` character, each define one transformation pass that each of the input strings undergoes. Under the headings, there are transformation rules consisting of two segments separated by a `/` character, which determine, in order: 1) what is transformed, 2) into what. A space character is understood literally; zero characters means an empty string. Comments, starting with `#`, are ignored by the program.

The names in the pass headings refer to the kinds of transformation passes the program can perform. Those are:

### `ReturningNonSegmentedRegex`

For every input string execute one regex replace per specified rule.

### `ReturningSegmentedRegex`

Like above, except split the input string into segments by ` ` and `-` characters, execute each rule on each segment separately, and finally recombine the string.

### `NonReturningNonSegmentedSimple`

A non-Regex substitution that, in contrast to the `Returning` passes, instead of executing each rule on the entire string, after matching a rule advances the cursor for all the subsequent rules in the same pass. `Simple` rules are, moreover, case-insensitive. The transformer will try to preserve the original casing.

## License

See [COPYING.md](COPYING.md).
