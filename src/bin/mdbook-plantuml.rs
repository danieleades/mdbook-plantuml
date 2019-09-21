extern crate clap;
extern crate mdbook;
extern crate mdbook_plantuml;
#[macro_use]
extern crate log;
extern crate log4rs;

use clap::{App, Arg, ArgMatches, SubCommand};
use mdbook::errors::Error as MDBookError;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor};
use mdbook_plantuml::PlantUMLPreprocessor;
use std::io;
use std::process;
use std::error::Error;

pub fn make_app() -> App<'static, 'static> {
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    App::new("mdBook PlantUML preprocessor")
        .version(VERSION)
        .author("Sytse Reitsma")
        .about("A mdbook preprocessor which renders PlantUML code blocks to inline SVG diagrams")
        .subcommand(
            SubCommand::with_name("supports")
                .arg(Arg::with_name("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
}

fn main() {
    let matches = make_app().get_matches();

    let preprocessor = PlantUMLPreprocessor;
    if let Some(sub_args) = matches.subcommand_matches("supports") {
        handle_supports(&preprocessor, sub_args);
    }
    else {
        if let Err(e) = setup_logging() {
           eprintln!("{}", e);
           process::exit(2);
        }

        if let Err(e) = handle_preprocessing(&preprocessor) {
           eprintln!("{}", e);
           process::exit(1);
        }
    }
}

fn handle_preprocessing(pre: &dyn Preprocessor) -> Result<(), MDBookError> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    if ctx.mdbook_version != mdbook::MDBOOK_VERSION {
        // We should probably use the `semver` crate to check compatibility
        // here...
        eprintln!(
            "Warning: The {} plugin was built against version {} of mdbook, \
             but we're being called from version {}",
            pre.name(),
            mdbook::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processed_book = pre.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;
    Ok(())
}

fn handle_supports(pre: &dyn Preprocessor, sub_args: &ArgMatches) -> ! {
    let renderer = sub_args.value_of("renderer").expect("Required argument");
    let supported = pre.supports_renderer(&renderer);

    // Signal whether the renderer is supported by exiting with 1 or 0.
    if supported {
        process::exit(0);
    } else {
        process::exit(1);
    }
}

fn setup_logging() -> Result<(), Box<Error>> {
    use log::LevelFilter;
    use log4rs::append::file::FileAppender;
    use log4rs::encode::pattern::PatternEncoder;
    use log4rs::config::{Appender, Config, Root};

    let logfile = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
            .build("D:\\Projects\\mdBook-plantuml\\output.log")?;

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder()
                   .appender("logfile")
                   .build(LevelFilter::Info))?;

    log4rs::init_config(config)?;

    info!("--- Started preprocessor ---");

    Ok(())
}