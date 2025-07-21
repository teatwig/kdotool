use kdotool::{create_connection, help::*, run_script, scripting_proxy, Globals};
use lexopt::Parser;

fn main() -> anyhow::Result<()> {
    let mut context = Globals {
        cmdline: std::env::args().collect::<Vec<String>>().join(" "),
        ..Default::default()
    };

    let mut parser = Parser::from_env();

    if let Ok(version) = std::env::var("KDE_SESSION_VERSION") {
        if version == "5" {
            context.kde5 = true;
        }
    }

    // Parse global options
    let mut next_arg: Option<String> = None;
    let mut opt_help = false;
    let mut opt_version = false;
    let mut opt_dry_run = false;
    let mut opt_remove = false;

    while let Some(arg) = parser.next()? {
        use lexopt::prelude::*;
        match arg {
            Short('h') | Long("help") => {
                opt_help = true;
            }
            Short('v') | Long("version") => {
                opt_version = true;
            }
            Short('d') | Long("debug") => {
                context.debug = true;
            }
            Short('n') | Long("dry-run") => {
                opt_dry_run = true;
            }
            Long("shortcut") => {
                context.shortcut = parser.value()?.string()?;
            }
            Long("name") => {
                context.script_name = parser.value()?.string()?;
            }
            Long("remove") => {
                opt_remove = true;
                context.script_name = parser.value()?.string()?;
            }
            Value(os_string) => {
                next_arg = Some(os_string.string()?);
                break;
            }
            _ => {
                return Err(arg.unexpected().into());
            }
        }
    }

    if !opt_remove && next_arg.is_none() || opt_help {
        help();
        return Ok(());
    }

    if opt_version {
        print_version();
        return Ok(());
    }

    env_logger::Builder::from_default_env()
        .filter(
            Some("kdotool"),
            if context.debug {
                log::LevelFilter::Debug
            } else {
                log::LevelFilter::Info
            },
        )
        .init();

    let kwin_conn = create_connection()?;

    if opt_remove {
        let _: () = scripting_proxy(&kwin_conn).method_call(
            "org.kde.kwin.Scripting",
            "unloadScript",
            (&context.script_name,),
        )?;
        return Ok(());
    }

    let command = next_arg.unwrap();

    run_script(&kwin_conn, command, &mut context, parser, opt_dry_run)?;

    Ok(())
}
