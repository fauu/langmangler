# langmangler

> Create words in a fictional language by transforming words from a real language.

Langmangler takes a list of strings and transforms them according to a supplied list of simple rules. It was designed to produce passably consistent large sets of fictional names of people and places with the least amount of effort, leveraging existing data.

**Example input:**<br>
`Yamagata Fukushima Saitama` + [`example-japanese-rules.txt`](#rules-file-format)<br>
**Example output:**<br>
`Limegite Efoququema Sliītime`

## Table of contents

- [Building](#building)
- [Usage](#usage)
- [Rules file format](#rules-file-format)
- [License](#license)

## Building

Requirements:

- Rust stable

Run:

```
$ cargo install --git "https://github.com/fauu/langmangler"
```

or simply build and run the program with a single command, without installing it, from within the cloned repository directory:

```
$ cargo run --
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
    -x, --reject <reject>    Rejects transformed strings that pass a specified check against the original [possible
                             values: Unchanged, AsciiUnchanged]
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

Below are the contents of the [example rules file](/examples/example-japanese-rules.txt), targeted at the Japanese language:

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

The headings starting with the `@` character determine the transformation passes that each of the input strings will undergo. Under the headings there are transformation rules consisting of two segments divided by a `/` character: (1) what is transformed, (2) into what. A space character signifies a space character, no character signifies an empty string. Lastly, you can write comments starting with `#` that will be ignored by the program.

The names in the pass headings refer to the kinds of transformation passes the program can perform. Those are:

### `ReturningNonSegmentedRegex`

For every input string execute a regex replace for each specified rule.

### `ReturningSegmentedRegex`

The same as above, except split the input string further into segments by space and dash characters and then re-combine it after executing each rule on each segment separately.

### `NonReturningNonSegmentedSimple`

Execute a simple, i.e. non-Regex substitution. In contrast with the `Returning` passes, the `NonReturning` pass does not execute each rule on the entire string, but, after matching a rule, advances the cursor for all the subsequent rules in the same pass. Moreover, `Simple` rules are case-insensitive: the transformer will try to preserve the original casing.

## License

See [COPYING.md](COPYING.md).
