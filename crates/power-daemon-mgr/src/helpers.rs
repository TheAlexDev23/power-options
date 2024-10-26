use std::io::{self, Write};

pub fn yn_prompt(prompt: &str) -> bool {
    loop {
        print!("{} [Y/n]: ", prompt);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let trimmed_input = input.trim().to_lowercase();
        match trimmed_input.as_str() {
            "y" | "yes" | "" => return true,
            "n" | "no" => return false,
            _ => println!("Invalid input. Please enter 'y' or 'n'."),
        }
    }
}
