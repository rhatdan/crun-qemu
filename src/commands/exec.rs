// SPDX-License-Identifier: GPL-2.0-or-later

use std::error::Error;
use std::fs::File;

use crate::crun::crun_exec;

pub fn exec(
    global_args: &liboci_cli::GlobalOpts,
    args: &mut liboci_cli::Exec,
) -> Result<(), Box<dyn Error>> {
    assert!(args.command.is_empty());

    let process_config_path = args.process.as_ref().expect("process config");
    let mut process: oci_spec::runtime::Process =
        serde_json::from_reader(File::open(process_config_path)?)?;

    let command = process.args().as_ref().expect("command specified");

    let ssh_user = command
        .first()
        .expect("first command argument is user to ssh as into the vm");

    let mut new_command = vec![];
    if ssh_user != "-" {
        new_command.extend([
            "ssh".to_string(),
            "-o".to_string(),
            "LogLevel=ERROR".to_string(),
            "-o".to_string(),
            "StrictHostKeyChecking=no".to_string(),
            "-l".to_string(),
            ssh_user.clone(),
            "localhost".to_string(),
        ]);
    }
    new_command.extend(command.iter().skip(1).cloned());

    process.set_args(Some(new_command));

    serde_json::to_writer(File::create(process_config_path)?, &process)?;

    crun_exec(global_args, args)?;

    Ok(())
}