use std::{
    fs,
    io::{self, BufRead},
    str::FromStr,
};

use clap::{arg_enum, value_t, Arg};
use deunicode::deunicode;
use itertools::{EitherOrBoth::*, Itertools};
use once_cell_regex::regex;
use regex::Regex;

arg_enum! {
    #[derive(PartialEq)]
    enum ResultRejectionCriterion {
        Unchanged,
        AsciiUnchanged,
    }
}

const APP_NAME: &str = "langmangler";

fn main() {
    let args = clap::App::new(APP_NAME)
        .arg(
            Arg::with_name("rules")
                .short('r')
                .long("rules")
                .takes_value(true)
                .required(true)
                .help("Path to the rules file"),
        )
        .arg(
            Arg::with_name("reject")
                .short('x')
                .long("reject")
                .takes_value(true)
                .possible_values(&ResultRejectionCriterion::variants())
                .case_insensitive(true)
                .help("Compare outputs with inputs and reject those that pass the specified check"),
        )
        .arg(
            Arg::with_name("print-original")
                .short('o')
                .long("compare")
                .takes_value(false)
                .help("Prints original strings for comparison"),
        )
        .get_matches();

    let transform_passes = parse_rules(args.value_of("rules").unwrap());

    // Parse and tranform the input
    let input_lines: Vec<String> = io::stdin().lock().lines().map(|l| l.unwrap()).collect();
    let results: Vec<String> = input_lines
        .iter()
        .map(|line| {
            transform_passes
                .iter()
                .fold(line.clone(), |acc, pass| pass.execute(&acc))
        })
        .collect();

    // Print the results
    let result_rejection_criterion = value_t!(args, "reject", ResultRejectionCriterion).ok();
    for (result, original_input) in results.iter().zip(input_lines.iter()) {
        let skip = match &result_rejection_criterion {
            Some(crit) => match crit {
                ResultRejectionCriterion::Unchanged => result == original_input,
                ResultRejectionCriterion::AsciiUnchanged => {
                    deunicode(result) == deunicode(original_input)
                }
            },
            None => false,
        };
        if skip {
            continue;
        }

        if args.is_present("print-original") {
            println!("{} ({})", result, original_input);
        } else {
            println!("{}", result);
        }
    }
}

fn parse_rules(filepath: &str) -> Vec<TransformPass> {
    let preprocessed_lines: Vec<String> = fs::read_to_string(filepath)
        .unwrap()
        .split('\n')
        .map(|line| regex!("#.*").replace_all(line, ""))
        .map(|line| line.trim().to_owned())
        .filter(|line| !line.is_empty())
        .collect();

    let mut res: Vec<TransformPass> = Vec::new();
    let mut current_pass: Option<TransformPass> = None;
    for line in preprocessed_lines {
        if let Some(pass_name) = line.strip_prefix('@') {
            // CODE: Check https://github.com/rust-lang/rust/issues/53667 in a few months
            if let Some(pass) = current_pass {
                if !pass.has_rules() {
                    res.push(pass);
                }
            }
            current_pass = Some(
                TransformPass::from_str(pass_name)
                    .expect("Invalid transform pass definition header"),
            );
            continue;
        }

        match &mut current_pass {
            Some(pass) => pass.parse_rule(&line),
            _ => panic!("Missing transform pass definition header"),
        };
    }
    if let Some(pass) = current_pass {
        if !pass.has_rules() {
            res.push(pass);
        }
    }
    res
}

trait TransformRule {
    fn parse(rule: &str) -> Self;
}

struct SimpleTransformRule {
    from: String,
    to: String,
}

impl TransformRule for SimpleTransformRule {
    fn parse(rule: &str) -> Self {
        let mut split = rule.split('/');
        SimpleTransformRule {
            from: split.next().unwrap().to_owned().to_lowercase(),
            to: split.next().unwrap().to_owned().to_lowercase(),
        }
    }
}

struct RegexTransformRule {
    from: Regex,
    to: String,
}

impl TransformRule for RegexTransformRule {
    fn parse(rule: &str) -> Self {
        let mut split = rule.split('/');
        RegexTransformRule {
            from: Regex::new(split.next().unwrap()).unwrap(),
            to: split.next().unwrap().to_string(),
        }
    }
}

trait TransformPassLike {
    fn has_rules(&self) -> bool;
    fn parse_rule(&mut self, rule: &str);
    fn execute(&self, str: &str) -> String;
}

struct TransformPassImpl<T: TransformRule> {
    rules: Vec<T>,
}

impl<T: TransformRule> TransformPassImpl<T> {
    fn new() -> Self {
        TransformPassImpl { rules: Vec::new() }
    }
}

enum TransformPass {
    NonReturningNonSegmentedSimple(TransformPassImpl<SimpleTransformRule>),
    ReturningNonSegmentedRegex(TransformPassImpl<RegexTransformRule>),
    ReturningSegmentedRegex(TransformPassImpl<RegexTransformRule>),
}

impl FromStr for TransformPass {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            #[rustfmt::skip]
            "NonReturningNonSegmentedSimple" => Ok(
                TransformPass::NonReturningNonSegmentedSimple(
                    TransformPassImpl::<SimpleTransformRule>::new(),
                )
            ),
            #[rustfmt::skip]
            "ReturningNonSegmentedRegex" => Ok(
                TransformPass::ReturningNonSegmentedRegex(
                    TransformPassImpl::<RegexTransformRule>::new(),
                )
            ),
            #[rustfmt::skip]
            "ReturningSegmentedRegex" => Ok(
                TransformPass::ReturningSegmentedRegex(
                    TransformPassImpl::<RegexTransformRule,>::new(),
                )
            ),
            _ => Err(()),
        }
    }
}

enum CharCase {
    Lower,
    Upper,
}

impl TransformPassLike for TransformPass {
    // CODE: Dumb. Perhaps TODO conceal the dumdness behind a macro
    fn has_rules(&self) -> bool {
        match self {
            TransformPass::NonReturningNonSegmentedSimple(i) => i.has_rules(),
            TransformPass::ReturningNonSegmentedRegex(i) => i.has_rules(),
            TransformPass::ReturningSegmentedRegex(i) => i.has_rules(),
        }
    }

    fn parse_rule(&mut self, rule: &str) {
        match self {
            TransformPass::NonReturningNonSegmentedSimple(i) => i.parse_rule(rule),
            TransformPass::ReturningNonSegmentedRegex(i) => i.parse_rule(rule),
            TransformPass::ReturningSegmentedRegex(i) => i.parse_rule(rule),
        }
    }

    fn execute(&self, str: &str) -> String {
        match self {
            TransformPass::NonReturningNonSegmentedSimple(pass) => {
                let mut res = String::with_capacity(str.len() * 2);

                let input_chars: Vec<char> = str.chars().collect();
                let input_chars_cases: Vec<CharCase> = input_chars
                    .iter()
                    .map(|ch| match ch.is_lowercase() {
                        true => CharCase::Lower,
                        false => CharCase::Upper,
                    })
                    .collect();

                let mut i = 0;
                'input_chars: while i < input_chars.len() {
                    'rules: for rule in &pass.rules {
                        // Check for mismatch between rule chars and input chars
                        for (j, rule_ch) in rule.from.chars().enumerate() {
                            if i + j >= input_chars.len() {
                                // We went over the input string without having matched the rule
                                continue 'rules;
                            }
                            let rule_ch_str = rule_ch.to_string();
                            if input_chars[i + j].to_lowercase().to_string() != rule_ch_str {
                                // Found a mismatch, the rule doesn't apply here
                                continue 'rules;
                            }
                        }

                        // The rule applies, do the transformation
                        if rule.to.is_empty() {
                            // Skip the input char
                            i += 1;
                            continue;
                        } else {
                            // Output a string with casing equivalent to the input
                            let input_match_len = rule.from.chars().count();
                            let cased_output_chunk =
                                apply_casing(&rule.to, &input_chars_cases[i..i + input_match_len]);
                            res.push_str(&cased_output_chunk);
                            i += input_match_len;
                        }
                        continue 'input_chars;
                    }
                    // No matching rule found, leave the character as it was
                    res.push(input_chars[i]);
                    i += 1;
                }

                res
            }

            TransformPass::ReturningNonSegmentedRegex(pass) => {
                let mut res = str.to_owned();
                for rule in &pass.rules {
                    res = rule.from.replace_all(&res, &rule.to).to_string();
                }
                res
            }

            TransformPass::ReturningSegmentedRegex(pass) => {
                let (segs, seg_delims) = segmentize(str);

                let mut transformed_segs: Vec<String> = Vec::with_capacity(segs.len());
                for seg in segs {
                    let mut transformed_seg = seg.clone();
                    for rule in &pass.rules {
                        transformed_seg = rule
                            .from
                            .replace_all(&transformed_seg, &rule.to)
                            .to_string();
                    }
                    transformed_segs.push(transformed_seg);
                }

                desegmentize(transformed_segs, seg_delims)
            }
        }
    }
}

impl<T: TransformRule> TransformPassImpl<T> {
    fn has_rules(&self) -> bool {
        self.rules.is_empty()
    }

    fn parse_rule(&mut self, rule: &str) {
        self.rules.push(T::parse(rule))
    }
}

const SEG_DELIMS: [char; 2] = [' ', '-'];

fn segmentize(str: &str) -> (Vec<String>, Vec<char>) {
    let mut segs = Vec::<String>::new();
    let mut popped_seg_delims = Vec::<char>::new();

    let mut seg_buf = String::new();
    let mut char_iter = str.chars().peekable();
    while let Some(c) = char_iter.next() {
        if SEG_DELIMS.contains(&c) {
            // Delimiter
            popped_seg_delims.push(c);
            segs.push(seg_buf.clone());
            seg_buf.clear();
        } else {
            // Non-delimiter
            seg_buf.push(c);
        }
        if char_iter.peek().is_none() && !seg_buf.is_empty() {
            // End of string, seg_buf holds the final segment
            segs.push(seg_buf);
            break;
        }
    }

    (segs, popped_seg_delims)
}

fn desegmentize(segs: Vec<String>, seg_delims: Vec<char>) -> String {
    segs.iter()
        .enumerate()
        .map(|(i, seg)| {
            if i == 0 {
                seg.clone()
            } else {
                format!("{}{}", seg_delims[i - 1], seg)
            }
        })
        .collect()
}

// ("asdf", [Lower, Upper, Upper]) -> "aSDf"
fn apply_casing(lowercased_str: &str, char_cases: &[CharCase]) -> String {
    lowercased_str
        .chars()
        .zip_longest(char_cases)
        .map(|pair| match pair {
            Both(to_char, from_char_case) => match from_char_case {
                CharCase::Lower => to_char.to_string(),
                CharCase::Upper => to_char.to_uppercase().to_string(),
            },
            Left(to_char) => to_char.to_string(),
            Right(_) => "".to_owned(),
        })
        .collect()
}
