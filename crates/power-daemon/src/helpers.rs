use std::{
    io::{Read, Write},
    process::{Child, Command, Stdio},
};

static mut SHELL_INSTANCE: Option<Child> = None;

pub fn run_command(command: &str) {
    unsafe {
        if SHELL_INSTANCE.is_none() {
            SHELL_INSTANCE = Some(
                Command::new("sh")
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()
                    .expect("Could not spawn shell process"),
            );
        }

        let stdin = SHELL_INSTANCE
            .as_mut()
            .unwrap()
            .stdin
            .as_mut()
            .expect("Could not open attached shell process stdin");

        let stdout = SHELL_INSTANCE
            .as_mut()
            .unwrap()
            .stdout
            .as_mut()
            .expect("Could not open attached shell process stdin");

        writeln!(stdin, "{command}").expect("Could not write to attached shell stdin");

        let mut output = String::new();
        stdout
            .read_to_string(&mut output)
            .expect("Could not read attached shell stdout");
        print!("{}", output);
    }
}
