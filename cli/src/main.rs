use std::ffi::OsStr;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{Error, Result};
use log::info;
use structopt::{
    clap::AppSettings::{ColoredHelp, DeriveDisplayOrder},
    StructOpt,
};
use tempfile::{Builder as TempFileBuilder, NamedTempFile};

use zhconv::{
    get_builtin_converter, get_builtin_tables, pagerules::PageRules, Variant, ZhConverterBuilder,
};

const DFA_FILESIZE: usize = 2 * 1024 * 1024;

#[derive(StructOpt, Debug)]
#[structopt(name = "zhconv", about = "Convert among Trad/Simp and regional variants of Chinese", global_settings(&[ColoredHelp, DeriveDisplayOrder]))]
struct Opt {
    /// Additional conversion rules
    #[structopt(long = "rule")]
    rules: Vec<String>,

    /// File(s) consisting of additional conversion rules seperated by LF
    #[structopt(long = "rules_file", parse(from_os_str))]
    rule_files: Vec<PathBuf>,

    /// Treat the input text as wikitext and process inline conversion rules in MediaWiki syntax
    #[structopt(long)]
    wikitext: bool,

    /// Target variant to convert to (zh, zh-Hant, zh-Hans, zh-TW, zh-HK, zh-MO, zh-CN, zh-SG, zh-MY)
    #[structopt(name = "VARIANT")]
    variant: Variant,

    /// File(s) to convert in-place (omit for stdin/out)  
    #[structopt(name = "FILE", parse(from_os_str))]
    files: Vec<PathBuf>,
}

// TODO: better param names naming

fn main() -> Result<()> {
    let Opt {
        rules,
        rule_files,
        wikitext,
        variant,
        files,
    } = Opt::from_args();

    let convert_to: Box<dyn Fn(&str, &mut String) -> ()> = if wikitext {
        if !rules.is_empty() || !rule_files.is_empty() {
            unimplemented!("Convert wikitext with additional rules are not supported yet")
        }
        let converter = get_builtin_converter(variant);
        Box::new(|text: &str, output: &mut String| {
            converter.convert_to_as_wikitext_extended(&text, output)
        })
    } else {
        let mut secondary_builder = ZhConverterBuilder::new();
        for rule in rules.into_iter().filter(|s| !s.trim().is_empty()) {
            secondary_builder = secondary_builder
                .convs([&rule.parse().map_err(|_e| Error::msg("Invalid rule"))?]);
        }
        for path in rule_files.into_iter() {
            secondary_builder = secondary_builder.conv_lines(&fs::read_to_string(path)?);
        }
        let secondary_converter = secondary_builder.build();

        let converter = get_builtin_converter(variant);
        Box::new(
            (move |text: &str, output: &mut String| {
                converter.convert_to_with_secondary_converter(&text, output, &secondary_converter)
            }),
        )
    };

    let convert = |text: &str| {
        let mut output = String::with_capacity(text.len());
        convert_to(text, &mut output);
        output
    };

    if files.is_empty() {
        let mut input = String::new();
        io::stdin().lock().read_to_string(&mut input).unwrap();
        println!("{}", convert(&input));
    } else {
        let total = files.len();
        for (idx, path) in files.into_iter().enumerate() {
            let text = fs::read_to_string(&path)?;
            info!(
                "Converting {} ... ({}/{})",
                path.to_string_lossy(),
                idx + 1,
                total
            );
            let mut tempfile = tempfile_for(&path)?;
            writeln!(tempfile, "{}", convert(&text))?;
            fs::rename(tempfile.path(), path)?;
        }
    }
    // test: 天干物燥，小心火烛。你想干什么不干他的事。天干地支，简称干支，是传统纪年方法。公交车和出租车都是公共交通工具。老挝（-{D|zh-cn:老挝; zh-hk: 寮國}-）是一个位于东南亚的国家。
    Ok(())
}

#[allow(clippy::or_fun_call)]
fn tempfile_for(path: &Path) -> io::Result<NamedTempFile> {
    TempFileBuilder::new()
        .prefix(path.file_stem().unwrap_or(OsStr::new(".zhconvtmp")))
        .tempfile_in(path.parent().unwrap_or(&"./".parse::<PathBuf>().unwrap()))
}
