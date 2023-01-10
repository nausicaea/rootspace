use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "loader", author, version, about = "Loads files as Stanford PLY model files", long_about = None)]
struct Args {
    #[arg(short, long, help = "Increases the output of the program", action = clap::ArgAction::Count)]
    verbose: u8,
    #[arg(short, long, help = "Print only the names of elements", action = clap::ArgAction::SetTrue)]
    element_names: bool,
    #[arg(short, long, help = "Print the names of properties for each element", action = clap::ArgAction::SetTrue)]
    property_names: bool,
    #[arg(short, long, help = "Print the face primitive type (triangles, quads)", action = clap::ArgAction::SetTrue)]
    face_types: bool,
    #[arg(help = "Specify the path(s) of the file to be parsed", action = clap::ArgAction::Append)]
    paths: Vec<std::path::PathBuf>,
}

fn main() {
    let matches = Args::parse();

    for path in &matches.paths {
        if matches.verbose > 0 {
            println!("{}:", path.display());
        }
        match plyers::load_ply(path) {
            Err(e) => panic!("{}", e),
            Ok(ply) => {
                if matches.verbose > 1 {
                    println!("{:?}", ply)
                }

                if matches.element_names {
                    for element in &ply.descriptor.elements {
                        println!("{}", &element.name);
                    }
                }

                if matches.property_names {
                    for element in &ply.descriptor.elements {
                        for property in &element.properties {
                            println!("{}.{}", &element.name, &property.name);
                        }
                        for list_property in &element.list_properties {
                            println!("{}.{}", &element.name, &list_property.name);
                        }
                    }
                }

                if matches.face_types {
                    let ft = ply.face_type();
                    println!("{:?}", ft);
                }
            }
        }
    }
}
