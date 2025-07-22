use crate::ui::splash::LOGO_NAME;

macro_rules! print_cmd_error {
    ($tt:tt) => {
        println!("\x1b[1;31m[ERROR!!!] {}\x1b[0m", $tt);
        println!("\x1b[1;31m[ERROR!!!]\x1b[0m Raw error being sent to stderr...\n");
    };
    ($tt:tt, $($tts:tt)+) => {
        println!("\x1b[1;31m[ERROR!!!] {}\x1b[0m", $tt);
        println!("\x1b[1;31m[ERROR!!!]\x1b[0m Raw error being sent to stderr...");
        println!("\x1b[1;31m[ERROR!!!]\x1b[0m Start details...");
        println!("{}", core::format_args!($($tts)*));
        println!("\x1b[1;31m[ERROR!!!]\x1b[0m End details.\n");
    }
}

macro_rules! handle_cmd_error {
    ($err:tt, $tt:tt) => {{
        print_cmd_error!($tt);
        format!("{}", $err)
    }};
}

macro_rules! print_cmd_info {
    ($tt:tt, $($tts:tt)*) => {
        println!("\x1b[1;33m[INFO!!!] {}\x1b[0m", $tt);
        println!("{}", core::format_args!($($tts)*));
    }
}

pub(crate) fn print_friendly_error_header() {
    // RGB: FF = 255, AA = 170, 00 = 0
    println!("\x1b[38;2;255;170;0m{}\x1b[0m", LOGO_NAME);
    println!("\x1b[38;2;255;170;0mWe'll be back shortly!\x1b[0m");
    println!(
        "The orchestrator of the prover network is under unprecedented traffic. The team has been notified. Thank you for your patience while the issue is resolved.\n"
    );
}

pub(crate) use handle_cmd_error;
pub(crate) use print_cmd_error;
pub(crate) use print_cmd_info;
