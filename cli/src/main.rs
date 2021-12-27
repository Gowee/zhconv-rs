use std::array;
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

use zhconv::{get_builtin_table, pagerules::PageRules, Variant, ZhConverterBuilder};

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

    /// Processes inline MediaWiki conversion rules in the input
    #[structopt(long)]
    mediawiki: bool,

    /// Whether to build DFA for AC automaton{n}
    /// With DFA enabled by default, it is slower to warm up while faster to convert.{n}
    /// Omit to let the program to determine by input size.
    #[structopt(long)]
    dfa: Option<bool>,

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
        mediawiki,
        dfa,
        variant,
        files,
    } = Opt::from_args();

    let mut builder = ZhConverterBuilder::new()
        .target(variant)
        .table(get_builtin_table(variant));

    for rule in rules.into_iter().filter(|s| !s.trim().is_empty()) {
        builder = builder.add_conv(rule.parse().map_err(|_e| Error::msg("Invalid rule"))?);
    }
    for path in rule_files.into_iter() {
        builder = builder.conv_lines(&fs::read_to_string(path)?);
    }

    if files.is_empty() {
        let mut input = String::new();
        io::stdin().lock().read_to_string(&mut input).unwrap();
        if mediawiki {
            builder = builder.rules_from_page(&input);
        }
        builder = builder.dfa(dfa.unwrap_or(input.len() >= DFA_FILESIZE));
        let converter = builder.build();
        if mediawiki {
            println!("{}", converter.convert_allowing_inline_rules(&input));
        } else {
            println!("{}", converter.convert(&input));
        }
    } else {
        let total = files.len();
        let mut it = files.into_iter();

        let first_path = it.next().unwrap();
        let first_text = fs::read_to_string(&first_path)?;

        let dfa = dfa.unwrap_or(total > 1 || first_text.len() >= DFA_FILESIZE);
        builder = builder.dfa(dfa);

        let files = array::IntoIter::new([(first_path, Ok(first_text))])
            .into_iter()
            .chain(it.map(|path| {
                let res = fs::read_to_string(&path);
                (path, res)
            }));

        if mediawiki {
            let mut converter = None;
            for (idx, (path, res)) in files.into_iter().enumerate() {
                let text = res?;
                info!(
                    "Converting {} ... ({}/{})",
                    path.to_string_lossy(),
                    idx + 1,
                    total
                );
                let page_rules = text
                    .parse::<PageRules>()
                    .map_err(|_e| Error::msg("Invalid rules in the text"))?;
                let mut tempfile = tempfile_for(&path)?;
                if page_rules.as_conv_actions().is_empty() {
                    // no inline global rules, try to re-use the existing converter
                    let converter = converter.get_or_insert_with(|| builder.build());
                    writeln!(
                        tempfile,
                        "{}",
                        converter.convert_allowing_inline_rules(&text)
                    )?;
                } else {
                    // inline global rules exists, so build a new converter
                    let converter = builder.clone().page_rules(&page_rules).build();
                    writeln!(
                        tempfile,
                        "{}",
                        converter.convert_allowing_inline_rules(&text)
                    )?;
                }

                fs::rename(tempfile.path(), path)?;
            }
        } else {
            let converter = builder.build();
            for (idx, (path, res)) in files.into_iter().enumerate() {
                let text = res?;
                info!(
                    "Converting {} ... ({}/{})",
                    path.to_string_lossy(),
                    idx + 1,
                    total
                );
                let mut tempfile = tempfile_for(&path)?;
                writeln!(tempfile, "{}", converter.convert(&text))?;
                fs::rename(tempfile.path(), path)?;
            }
        }
    }
    //天干物燥，小心火烛。你想干什么不干他的事。天干地支，简称干支，是传统纪年方法。公交车和出租车都是公共交通工具。老挝（-{D|zh-cn:老挝; zh-hk: 寮國}-）是一个位于东南亚的国家。
    Ok(())
}

#[allow(clippy::or_fun_call)]
fn tempfile_for(path: &Path) -> io::Result<NamedTempFile> {
    TempFileBuilder::new()
        .prefix(path.file_stem().unwrap_or(OsStr::new(".zhconvtmp")))
        .tempfile_in(path.parent().unwrap_or(&"./".parse::<PathBuf>().unwrap()))
}
