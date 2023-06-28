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

use zhconv::get_builtin_tables;
use zhconv::{get_builtin_converter, rule::Conv, Variant, ZhConverterBuilder};

#[derive(StructOpt, Debug)]
#[structopt(name = "zhconv", about = "Convert among Trad/Simp and regional variants of Chinese", global_settings(&[ColoredHelp, DeriveDisplayOrder]))]
struct Opt {
    /// Additional conversion rules in MediaWiki syntax (excluding -{, }-)
    #[structopt(long = "rule")]
    rules: Vec<String>,

    /// File(s) consisting of additional conversion rules in MediaWiki syntax (excluding -{, }-)
    /// seperated by LF
    #[structopt(long = "rules_file", parse(from_os_str))]
    rule_files: Vec<PathBuf>,

    /// Treat the input text as wikitext and process inline conversion rules in MediaWiki syntax
    #[structopt(long)]
    wikitext: bool,

    /// Dump the built-in conversion table, along with additional rules specified if any
    #[structopt(long)]
    dump_table: bool,

    /// Target variant to convert to (zh, zh-Hant, zh-Hans, zh-TW, zh-HK, zh-MO, zh-CN, zh-SG, zh-MY)
    #[structopt(name = "VARIANT")]
    variant: Variant,

    /// File(s) to convert in-place (omit for stdio)  
    #[structopt(name = "FILE", parse(from_os_str))]
    files: Vec<PathBuf>,
}

// TODO: better param names naming

fn main() -> Result<()> {
    let Opt {
        rules,
        rule_files,
        wikitext,
        dump_table,
        variant,
        files,
    } = Opt::from_args();

    let mut secondary_builder = ZhConverterBuilder::new().target(variant);
    for rule in rules.into_iter().filter(|s| !s.trim().is_empty()) {
        secondary_builder = secondary_builder.conv_pairs(
            rule.parse::<Conv>()
                .map_err(|_e| Error::msg("Invalid rule"))?
                .get_conv_pairs(variant),
        )
    }
    for path in rule_files.into_iter() {
        secondary_builder = secondary_builder.conv_lines(fs::read_to_string(path)?.lines());
    }

    if dump_table {
        secondary_builder = secondary_builder.tables(get_builtin_tables(variant));
        for (from, to) in secondary_builder.build_mapping() {
            println!("{} {}", from, to);
        }
        return Ok(());
    }

    #[allow(clippy::type_complexity)]
    let convert_to: Box<dyn Fn(&str, &mut String)> = if wikitext {
        let converter = get_builtin_converter(variant);
        Box::new(move |text: &str, output: &mut String| {
            converter.convert_to_as_wikitext(
                text,
                output,
                &mut Some(secondary_builder.clone()),
                true,
                true,
            );
        })
    } else {
        let secondary_converter = secondary_builder.build();

        let converter = get_builtin_converter(variant);
        Box::new(move |text: &str, output: &mut String| {
            converter.convert_to_with_secondary_converter(text, output, &secondary_converter)
        })
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
