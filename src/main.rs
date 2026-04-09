mod app;
mod args;
mod cmds;
mod config;
mod domain;
mod errors;
mod ops;

fn main() {
    let result = app::run();

    if let Err(error) = &result {
        eprintln!("Error: {:#}", error);

        if let Some(follow_up) = error.follow_up() {
            eprint!("{follow_up}");
        }

        std::process::exit(1);
    }
}
