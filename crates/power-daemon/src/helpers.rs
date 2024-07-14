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

        writeln!(stdin, "{command}").expect("Could not write to attached shell stdin");
    }
}

pub fn run_command_with_output_unchecked(command: &str) -> (String, String) {
    let args = shell_words::split(command).unwrap();

    let mut args_iter = args.iter();

    let mut command_proc = Command::new(args_iter.next().unwrap());

    for arg in args_iter {
        command_proc.arg(arg);
    }

    let result = command_proc.output().unwrap();

    let stdout = String::from_utf8_lossy(&result.stdout).to_string();
    let stderr = String::from_utf8_lossy(&result.stderr).to_string();

    (stdout, stderr)
}
