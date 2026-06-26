use std::io::{self, BufRead};

fn main() {
    // Clear screen
    print!("\x1B[2J\x1B[1;1H");

    println!("\x1B[1;33m");
    println!("          _....._");
    println!("       .-'       '-.");
    println!("     .'  _     _   '.      \x1B[1;31m_");
    println!("    \x1B[1;33m/   / \\   / \\    \\    \x1B[1;31m(_)");
    println!("   \x1B[1;33m|   | (o) (o) |    |    \x1B[1;33m|");
    println!("   |   |   | |   |    |    |");
    println!("    \\   \\  \\_/  /    /    /");
    println!("     '.  '-----'   .'  .-'");
    println!("       \x1B[1;34m'-._______.-'-'  /");
    println!("         /   |   \\    /");
    println!("        /    |    \\  /");
    println!("       /     |     \\/");
    println!("\x1B[0m");

    println!("\x1B[1;32m=== UBERMETROID SECURITY CONSOLE ===\x1B[0m");
    println!("\x1B[1;31mSystem Alert: Console Access is UNAUTHORIZED.\x1B[0m");
    println!("This application is running inside a secure, read-only Nix container.");
    println!("Direct shell access is disabled for environment isolation and security.");
    println!("\nPress \x1B[1;37m[Enter]\x1B[0m to close connection...");

    let stdin = io::stdin();
    let mut buffer = String::new();
    let _ = stdin.lock().read_line(&mut buffer);
}
