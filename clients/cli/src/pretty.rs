use crate::ui::splash::LOGO_NAME;

macro_rules! print_cmd_error {
    ($tt:tt) => {
        println!("\x1b[1;31m[ERRROR!!!] {}\x1b[0m", $tt);
        println!("\t\tCheck stderr for raw error content.");
    };
    ($tt:tt, $($tts:tt)+) => {
        println!("\x1b[1;31m[ERRROR!!!] {}\x1b[0m", $tt);
        println!("\t\t{}", core::format_args!($($tts)*));
        println!("\t\tCheck stderr for raw error content.");
    }
}

macro_rules! handle_cmd_error {
    ($err:tt, $tt:tt) => {
        {
            print_cmd_error!($tt);
            $err
        }
    }
}

macro_rules! print_cmd_info {
    ($tt:tt, $($tts:tt)*) => {
        println!("\x1b[1;33m[INFO!!!] {}\x1b[0m", $tt);
        println!("\t\t{}", core::format_args!($($tts)*));
    }
}

pub(crate) fn print_friendly_error_header() {
    // RGB: FF = 255, AA = 170, 00 = 0
    println!("\x1b[38;2;255;170;0m{}\x1b[0m", LOGO_NAME);
    println!("\x1b[38;2;255;170;0mWe'll be back shortly\x1b[0m");
    println!(
        "The Prover network’s orchestrater is under unprecedented traffic. Team has been notified. Thank you for your patience while issue is resolved.\n"
    );
}

pub(crate) use print_cmd_error;
pub(crate) use handle_cmd_error;
pub(crate) use print_cmd_info;
