use clap::Parser;
use rootspace::plyers::load_ply;
use rootspace::plyers::types::PropertyDescriptor;

#[derive(Debug, Parser)]
#[command(name = "loader", author, version, about = "Loads files as Stanford PLY model files", long_about = None)]
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
    #[arg(help = "Specify the path(s) of the file to be parsed", action = clap::ArgAction::Append)]
    paths: Vec<std::path::PathBuf>,
}

fn main() {
    let matches = Args::parse();

    for path in &matches.paths {
        if matches.verbose > 0 {
            println!("{}:", path.display());
        }
        match load_ply(path) {
            Err(e) => panic!("{}", e),
            Ok(ply) => {
                if matches.verbose > 1 {
                    println!("{:?}", ply)
                }

                if matches.element_names {
                    for (_, element) in &ply.descriptor.elements {
                        println!("{}", &element.name);
                    }
                }

                if matches.property_names {
                    for (_, element) in &ply.descriptor.elements {
                        for (_, property) in &element.properties {
                            match property {
                                PropertyDescriptor::Scalar { ref name, .. }
                                | PropertyDescriptor::List { ref name, .. } => println!("{}.{}", &element.name, name),
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
                    for (_, element) in &ply.descriptor.elements {
                        for comment in &element.comments {
                            println!("{}: {}", &element.name, comment);
                        }
                        for (_, property) in &element.properties {
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
                    for (_, element) in &ply.descriptor.elements {
                        for obj_info in &element.obj_info {
                            println!("{}: {}", &element.name, obj_info);
                        }
                        for (_, property) in &element.properties {
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
            }
        }
    }
}
