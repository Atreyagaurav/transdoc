use crate::{
    components::*,
    errors::{MatchRes, ParseError},
    syntax::*,
    tokenizer::TokenList,
};
use nom::{
    branch::alt,
    combinator::{map, opt},
    multi::{many0, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair},
    Finish, Parser,
};
use std::collections::HashMap;
use std::str::FromStr;

pub fn linetag(inp: TokenList<'_>) -> MatchRes<'_, String> {
    delimited(at, maybe_space(string_val), many1_newlines).parse(inp)
}

pub fn str_trimmed(inp: TokenList<'_>) -> MatchRes<'_, String> {
    map(string_val, |s| s.trim().to_string()).parse(inp)
}

pub fn dict_meaning(inp: TokenList<'_>) -> MatchRes<'_, OrgFragment> {
    map(
        separated_pair(
            str_trimmed,
            maybe_space(equal),
            maybe_space(separated_list1(
                maybe_space(semicolon),
                maybe_space(str_trimmed),
            )),
        ),
        |(v, m)| OrgFragment::Meaning(v, m),
    )
    .parse(inp)
}

pub fn org_frag_dict(inp: TokenList<'_>) -> MatchRes<'_, OrgFragment> {
    delimited(
        angle_start,
        maybe_space(alt((
            dict_meaning,
            map(str_trimmed, OrgFragment::DictLookup),
        ))),
        maybe_space(angle_end),
    )
    .parse(inp)
}

pub fn lines_separator(inp: TokenList<'_>) -> MatchRes<'_, Option<String>> {
    newline_terminated(preceded(dash, maybe_space(opt(str_trimmed)))).parse(inp)
}

pub fn key_val(inp: TokenList<'_>) -> MatchRes<'_, (String, String)> {
    separated_pair(str_trimmed, maybe_space(equal), maybe_space(str_trimmed)).parse(inp)
}

pub fn original_sentence(inp: TokenList<'_>) -> MatchRes<'_, Vec<OrgFragment>> {
    newline_terminated(many0(alt((
        org_frag_dict,
        map(string_val, OrgFragment::Simple),
    ))))
    .parse(inp)
}

pub fn attrs(inp: TokenList<'_>) -> MatchRes<'_, HashMap<String, String>> {
    map(many0(newline_terminated(maybe_newline(key_val))), |vals| {
        vals.into_iter().collect()
    })
    .parse(inp)
}

pub fn tl_sentence(inp: TokenList<'_>) -> MatchRes<'_, Translation> {
    map(
        pair(newline_terminated(string_val), maybe_newline(attrs)),
        |(s, a)| Translation {
            content: s,
            attrs: a,
        },
    )
    .parse(inp)
}

pub fn sentence(inp: TokenList<'_>) -> MatchRes<'_, Sentence> {
    map(
        (
            linetag,
            maybe_newline(original_sentence),
            maybe_newline(attrs),
            many0(pair(
                maybe_newline(lines_separator),
                maybe_newline(tl_sentence),
            )),
        ),
        |(tag, org, attrs, tls)| Sentence {
            label: tag,
            original: org,
            orgattrs: attrs,
            translations: tls
                .into_iter()
                .enumerate()
                .map(|(i, (l, c))| (l.unwrap_or_else(|| i.to_string()), c))
                .collect(),
        },
    )
    .parse(inp)
}

pub fn chapter(inp: TokenList<'_>) -> MatchRes<'_, Chapter> {
    map(pair(attrs, many0(maybe_newline(sentence))), |(a, s)| {
        Chapter {
            title: a
                .get("title")
                .map(String::from)
                .unwrap_or("Unnamed Chapter".into()),
            language: a
                .get("language")
                .map(String::from)
                .unwrap_or("english".into()),
            tl_languages: a
                .get("tranlations")
                .map(|v| v.split(",").map(|l| l.to_string()).collect())
                .unwrap_or_default(),
            dictionary: a
                .get("dictionary")
                .map(|d| load_dictionary(d))
                .unwrap_or_default(),
            sentences: s,
            attrs: a,
        }
    })
    .parse(inp)
}

impl FromStr for Chapter {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens = crate::tokenizer::get_tokens(s);
        match chapter(TokenList::new(&tokens)).finish() {
            Ok((rest, chapter)) => {
                if rest.is_empty() {
                    Ok(chapter)
                } else {
                    let err = maybe_newline(sentence)
                        .parse(rest)
                        .finish()
                        .expect_err("Rest should be empty if network parse is complete");
                    Err(ParseError::new(
                        TokenList::new(&tokens),
                        err.internal.input,
                        err.ty,
                    ))
                }
            }
            Err(e) => Err(ParseError::new(
                TokenList::new(&tokens),
                e.internal.input,
                e.ty,
            )),
        }
    }
}

fn load_dictionary(file: &str) -> HashMap<String, Vec<String>> {
    let mut dict = HashMap::new();
    if let Ok(s) = std::fs::read_to_string(file) {
        let tokens = crate::tokenizer::get_tokens(&s);

        match trailing_newlines(attrs)
            .parse(TokenList::new(&tokens))
            .finish()
        {
            Ok((rest, attrs)) => {
                dict.extend(attrs.into_iter().map(|(k, v)| (k, vec![v])));
                if !rest.is_empty() {
                    let err = key_val(rest)
                        .finish()
                        .expect_err("Rest should be empty if network parse is complete");
                    eprintln!(
                        "{}",
                        ParseError::new(TokenList::new(&tokens), err.internal.input, err.ty,)
                            .user_msg(Some(file))
                    )
                }
            }
            Err(e) => eprintln!(
                "{}",
                ParseError::new(TokenList::new(&tokens), e.internal.input, e.ty,)
            ),
        }
    }
    dict
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::{get_tokens, TokenList};
    use rstest::rstest;

    #[rstest] // newline
    #[case("my name is")]
    #[should_panic]
    #[case("@what")]
    #[case("何か?")]
    #[case("यो काम गर्छ र")]
    fn sentence_valid_test(#[case] txt: &str) {
        let tk = get_tokens(txt);
        let (rest, _) = original_sentence(TokenList::new(&tk)).unwrap();
        assert_eq!(rest, TokenList::new(&[]));
    }

    #[rstest]
    #[case("<<abb>>")]
    #[case("<< ab b>>")]
    #[case("<< a = b>>")]
    #[case("<< a = b >>")]
    #[should_panic]
    #[case("x <<a>>")]
    #[case("<<何か>>")]
    #[case("<<काम=work>>")]
    fn org_frag_test(#[case] txt: &str) {
        let tk = get_tokens(txt);
        let (rest, _) = org_frag_dict(TokenList::new(&tk)).unwrap();
        assert_eq!(rest, TokenList::new(&[]));
    }

    #[rstest]
    #[case("a = b")]
    #[case("a = b")]
    #[should_panic]
    #[case("a")]
    #[case("काम=work")]
    fn dict_meaning_test(#[case] txt: &str) {
        let tk = get_tokens(txt);
        let (rest, _) = dict_meaning(TokenList::new(&tk)).unwrap();
        assert_eq!(rest, TokenList::new(&[]));
    }
}
