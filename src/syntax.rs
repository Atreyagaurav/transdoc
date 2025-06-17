use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

#[derive(Debug, Clone)]
pub enum OrgFragment {
    Simple(String),
    Meaning(String, Vec<String>),
    DictLookup(String),
}

impl OrgFragment {
    fn html(&self) -> String {
        match self {
            Self::Simple(s) => s.to_string(),
            Self::Meaning(s, m) => format!("<span title=\"{}\">{s}</span>", m.join("; ")),
            Self::DictLookup(s) => format!("<span class=\"unk\">{s}</span>"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Translation {
    pub content: String,
    pub attrs: HashMap<String, String>,
}

impl Translation {
    fn html(&self) -> String {
        format!("<div class=\"tl\">{}</div>", self.content)
    }
}

#[derive(Debug, Clone)]
pub struct Sentence {
    pub label: String,
    pub original: Vec<OrgFragment>,
    pub orgattrs: HashMap<String, String>,
    pub translations: HashMap<String, Translation>,
}

impl Sentence {
    fn html(&self) -> String {
        let org: Vec<String> = self.original.iter().map(OrgFragment::html).collect();
        let tls: Vec<String> = self.translations.values().map(Translation::html).collect();
        format!(
            "<p id=\"line-{}\"><div class=\"org\">{}</div>{}</p>",
            self.label,
            org.join(""),
            tls.join("")
        )
    }
}

#[derive(Debug, Clone)]
pub struct Chapter {
    pub title: String,
    pub language: String,
    pub tl_languages: Vec<String>,
    pub dictionary: HashMap<String, Vec<String>>,
    pub sentences: Vec<Sentence>,
    pub attrs: HashMap<String, String>,
}

impl Chapter {
    pub fn process(&mut self) {
        for s in &mut self.sentences {
            for w in &mut s.original {
                match w {
                    OrgFragment::Simple(_) => (),
                    OrgFragment::Meaning(s, m) => {
                        if let std::collections::hash_map::Entry::Vacant(e) =
                            self.dictionary.entry(s.to_string())
                        {
                            e.insert(m.clone());
                        }
                    }
                    OrgFragment::DictLookup(s) => {
                        if let Some(m) = self.dictionary.get(s) {
                            *w = OrgFragment::Meaning(s.to_string(), m.clone());
                        }
                    }
                }
            }
        }
    }

    pub fn to_html<P: AsRef<Path>>(&self, file: P) -> std::io::Result<()> {
        let mut f = File::create(file)?;
        write!(
            f,
            r#"
<html>
    <title> {} </title>
    <body>
	<style>
	 .tl {{
	     color: #aabbaa;
	 }}
	 .alt {{
	     color: green;
	 }}
	 .unk {{
	     color: red;
	 }}
	 span {{
	     color: blue;
	 }}
	 span:hover {{
	     background-color: pink;
	 }}
	</style>
"#,
            self.title
        )?;
        for s in &self.sentences {
            writeln!(f, "{}", s.html())?
        }
        write!(f, "</body></html>")?;
        Ok(())
    }
}
