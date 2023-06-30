use getopts;
use getopts::Options as GetOptOptions;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut opts = GetOptOptions::new();
    
    opts.optflag("h", "help", "print this help menu");
    opts.optopt("p", "path", "set path to log folder", "PATH");
    opts.optflag("a", "all", "print all issues from logs");
    opts.optflag("e", "error", "print main.ERROR from log");
    opts.optflag("w", "warning", "print main.WARNING from log");
    opts.optflag("c", "critical", "print main.CRITICAL from log");
    

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(_) => { panic!("no arguments") }
    };

    if matches.opt_present("h") {
        print_usage(opts);
        return;
    }

    print_usage(opts);

    let path_opt = matches.opt_str("p");
    let path = match path_opt {
        Some(x) => {x}
        None => { panic!("path not set") }
    };

    let mut magelog = magelogs::MageLog::new();

    magelog
        .set_path(path)
        .set_is_all(matches.opt_present("a"))
        .set_critical_issue(matches.opt_present("c"))
        .set_error_issue(matches.opt_present("e"))
        .set_warning_issue(matches.opt_present("w"))
        .watch();
}

fn print_usage(opts: GetOptOptions) {
    print!("{}", opts.usage("Usage: FILE [options]"));
}
