use engine::{CommandTrait, AssetMutTrait};
use file_manipulation::NewOrExFilePathBuf;
use anyhow::Result;
use ecs::Resources;
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use crate::assets::FileSystem;
use std::path::{Path, PathBuf};
use std::ffi::OsString;
use std::convert::TryFrom;

#[derive(Debug, Clone, Copy)]
pub struct FileSystemCommand;

impl CommandTrait for FileSystemCommand {
    fn name(&self) -> &'static str {
        "fs"
    }

    fn description(&self) -> &'static str {
        "Accesses file system assets"
    }

    fn run(&self, _: &Resources, args: &[String]) -> Result<()> {
        let matches = App::new("fs")
            .setting(AppSettings::DisableVersion)
            .setting(AppSettings::SubcommandRequiredElseHelp)
            .subcommand(
                SubCommand::with_name("new")
                    .about("Creates a new file system asset")
                    .setting(AppSettings::DisableVersion)
                    .setting(AppSettings::ArgRequiredElseHelp)
                    .arg(
                        Arg::with_name("path")
                            .required(true)
                            .takes_value(true)
                            .validator_os(|s| {
                                NewOrExFilePathBuf::try_from(s)
                                    .map(|_| ())
                                    .map_err(|e| OsString::from(format!("{}", e)))
                            })
                            .help("Sets the path of the file to write to"),
                    )
            )
            .get_matches_from_safe(args)?;

        if let Some(new_matches) = matches.subcommand_matches("new") {
            if let Some(path_str) = new_matches.value_of("path") {
                let path = PathBuf::from(path_str);
                FileSystem::default().to_path(&path)?;
            }
        }

        Ok(())
    }
}