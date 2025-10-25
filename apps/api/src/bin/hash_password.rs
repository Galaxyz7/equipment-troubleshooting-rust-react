use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <password>", args[0]);
        eprintln!("\nExample:");
        eprintln!("  cargo run --bin hash_password mySecurePassword123");
        std::process::exit(1);
    }

    let password = &args[1];
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    match argon2.hash_password(password.as_bytes(), &salt) {
        Ok(hash) => {
            println!("\n✅ Password hashed successfully!");
            println!("\nCopy this hash to your .env file as ADMIN_PASSWORD_HASH:\n");
            println!("{}", hash);
            println!();
        }
        Err(e) => {
            eprintln!("❌ Error hashing password: {}", e);
            std::process::exit(1);
        }
    }
}
