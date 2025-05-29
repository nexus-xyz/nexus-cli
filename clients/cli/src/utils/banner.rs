use crate::utils::system::measure_gflops;
use colored::Colorize;

pub const LOGO_NAME: &str = r#"
  ███╗   ██╗  ███████╗  ██╗  ██╗  ██╗   ██╗  ███████╗
  ████╗  ██║  ██╔════╝  ╚██╗██╔╝  ██║   ██║  ██╔════╝
  ██╔██╗ ██║  █████╗     ╚███╔╝   ██║   ██║  ███████╗
  ██║╚██╗██║  ██╔══╝     ██╔██╗   ██║   ██║  ╚════██║
  ██║ ╚████║  ███████╗  ██╔╝ ██╗  ╚██████╔╝  ███████║
  ╚═╝  ╚═══╝  ╚══════╝  ╚═╝  ╚═╝   ╚═════╝   ╚══════╝
"#;

#[allow(unused)]
pub fn print_banner(environment: &crate::environment::Environment) {
    // Split the logo into lines and color them differently
    let logo_lines: Vec<&str> = LOGO_NAME.lines().collect();
    for line in logo_lines {
        let mut colored_line = String::new();
        for c in line.chars() {
            if c == '█' {
                colored_line.push_str(&format!("{}", "█".bright_white()));
            } else {
                colored_line.push_str(&format!("{}", c.to_string().cyan()));
            }
        }
        println!("{}", colored_line);
    }

    let version = match option_env!("CARGO_PKG_VERSION") {
        Some(v) => format!("v{}", v),
        None => "(unknown version)".into(),
    };
    println!(
        "{} {} {}\n",
        "  Welcome to the".bright_white(),
        "Nexus Network CLI".bright_cyan().bold(),
        version.bright_white()
    );
    println!(
        "{}",
        "  Use the CLI to contribute to the massively-parallelized Nexus proof network."
            .bright_white()
    );
    println!();
    println!(
        "{}: {}",
        "Computational capacity of this node".bold(),
        format!("{:.2} GFLOPS", measure_gflops()).bright_cyan()
    );
    println!(
        "{}: {}",
        "Environment".bold(),
        environment.to_string().bright_cyan()
    );
}
