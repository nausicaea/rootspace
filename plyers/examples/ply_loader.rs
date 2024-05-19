use std::path::PathBuf;

use clap::Parser;
use plyers::{load_ply, save_ply};
use plyers::types::PropertyDescriptor;
use plyers::types::FormatType;

#[derive(Debug, Parser)]
#[command(name = "ply_loader", author, version, about = "Loads files as Stanford PLY model files", long_about = None)]
struct Args {
    #[arg(short, long, help = "Increases the output of the program", action = clap::ArgAction::Count)]
    verbose: u8,
    #[arg(short, long, help = "Print only the names of elements", action = clap::ArgAction::SetTrue)]
    element_names: bool,
    #[arg(short, long, help = "Print the names of properties for each element", action = clap::ArgAction::SetTrue)]
    property_names: bool,
    #[arg(short = 'f', long, help = "Print the primitive type (eg. triangles, quads)", action = clap::ArgAction::SetTrue)]
    primitives: bool,
    #[arg(short, long, help = "Print all obj_info directives", action = clap::ArgAction::SetTrue)]
    obj_infos: bool,
    #[arg(short, long, help = "Print all comment directives", action = clap::ArgAction::SetTrue)]
    comments: bool,
    #[arg(short = 't', long, help = "Select the format for re-serialization", default_value_t = FormatType::Ascii)]
    format: FormatType,
    #[arg(short, long, help = "Re-serialize all specified PLY files", required = false)]
    save: Option<PathBuf>,
    #[arg(help = "Specify the path(s) of the file to be parsed", action = clap::ArgAction::Append)]
    paths: Vec<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let matches = Args::parse();

    if let Some(save) = &matches.save {
        if !save.is_dir() {
            return Err(anyhow::anyhow!(
                "Save destination is not a directory: {}",
                save.display()
            ));
        }
    }

    for path in &matches.paths {
        if matches.verbose > 0 {
            println!("{}:", path.display());
        }

        let ply = load_ply(path)?;
        if matches.verbose > 1 {
            println!("{:?}", ply)
        }

        if matches.element_names {
            for element in ply.descriptor.elements.values() {
                println!("{}", &element.name);
            }
        }

        if matches.property_names {
            for element in ply.descriptor.elements.values() {
                for property in element.properties.values() {
                    match property {
                        PropertyDescriptor::Scalar { ref name, .. } | PropertyDescriptor::List { ref name, .. } => {
                            println!("{}.{}", &element.name, name)
                        }
                    }
                }
            }
        }

        if matches.primitives {
            let ft = ply.primitive();
            println!("{:?}", ft);
        }

        if matches.comments {
            for comment in &ply.descriptor.comments {
                println!("ply: {}", comment);
            }
            for element in ply.descriptor.elements.values() {
                for comment in &element.comments {
                    println!("{}: {}", &element.name, comment);
                }
                for property in element.properties.values() {
                    match property {
                        PropertyDescriptor::Scalar {
                            ref name, ref comments, ..
                        }
                        | PropertyDescriptor::List {
                            ref name, ref comments, ..
                        } => {
                            for comment in comments {
                                println!("{}.{}: {}", &element.name, name, comment);
                            }
                        }
                    }
                }
            }
        }

        if matches.obj_infos {
            for obj_info in &ply.descriptor.obj_info {
                println!("ply: {}", obj_info);
            }
            for element in ply.descriptor.elements.values() {
                for obj_info in &element.obj_info {
                    println!("{}: {}", &element.name, obj_info);
                }
                for property in element.properties.values() {
                    match property {
                        PropertyDescriptor::Scalar {
                            ref name, ref obj_info, ..
                        }
                        | PropertyDescriptor::List {
                            ref name, ref obj_info, ..
                        } => {
                            for oinf in obj_info {
                                println!("{}.{}: {}", &element.name, name, oinf);
                            }
                        }
                    }
                }
            }
        }

        if let Some(save) = &matches.save {
            let output_path = save.join(
                path.file_name()
                    .ok_or_else(|| anyhow::anyhow!("Cannot obtain file name of path: {}", path.display()))?,
            );
            let mut ply = ply;
            ply.descriptor.format_type = matches.format;
            save_ply(&ply, output_path)?;
        }
    }

    Ok(())
}
