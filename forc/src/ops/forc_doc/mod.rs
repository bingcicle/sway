use std::{env, path::PathBuf};
mod html;

use crate::{
    cli::{BuildCommand, DocCommand},
    utils::{
        cli_error::CliError,
        helpers::{find_manifest_dir, get_sway_files, read_manifest},
    },
};

use super::forc_build;

pub fn doc(command: DocCommand) -> Result<(), CliError> {
    let build_command = BuildCommand {
        path: command.path.clone(),
        print_asm: false,
        binary_outfile: None,
        offline_mode: false,
    };

    match forc_build::build(build_command) {
        Ok(_) => {
            let project_dir = if let Some(path) = &command.path {
                PathBuf::from(path)
            } else {
                env::current_dir()?
            };

            match find_manifest_dir(&project_dir) {
                Some(manifest_dir) => {
                    let manifest = read_manifest(&manifest_dir)?;
                    let project_name = manifest.project.name;
                    let project_name_buff = html::build_static_files(&project_name)?;
                    let files = get_sway_files(manifest_dir)?;

                    // build index.html
                    // list all Structs, Traits, Enums etc
                    let mut page_types = vec![];

                    env::set_current_dir(project_name_buff)?;

                    for file in files {
                        if let Ok(file_content) = std::fs::read_to_string(&file) {
                            if let core_lang::CompileResult::Ok {
                                value,
                                warnings: _,
                                errors: _,
                            } = core_lang::parse(&file_content)
                            {
                                let res = html::get_page_types(value);
                                page_types.extend(res.clone());
                            }
                        }
                    }

                    let main_sidebar = html::build_main_sidebar(&project_name, &page_types);

                    for page in page_types {
                        html::build_page(&page, &main_sidebar)?;
                    }

                    Ok(())
                }
                None => Err(CliError::manifest_file_missing(project_dir)),
            }
        }
        Err(err) => Err(err.into()),
    }
}
