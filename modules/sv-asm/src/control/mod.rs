use sv_lib::model::{DataBase}; 
//use sv_lib::file_handler;
use log::{info, error};
//use std::collections::{HashMap};
use std::process::Command;
use std::path::{Path, PathBuf};
use std::fs;


//pub mod utils;
mod alimp_syn;
//mod glocal_syn;


fn runc(
    cmd: &mut Command
) -> Result<(), Box<dyn std::error::Error>> {
    let status = cmd.status()?;
    if !status.success() {
        return Err(format!("Command failed: {:?}", cmd).into());
    }
    Ok(())
}

fn copy_dir_all(
    src: impl AsRef<Path>, 
    dst: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(&dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dest_path = dst.as_ref().join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), dest_path)?;
        }
    }

    Ok(())
}

fn get_alimp_sys(
    dir: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("GITHUB_TOKEN").map_err(|e| {
        error!("GITHUB_TOKEN is not present or invalid: {}", e);
        e
    })?;

    let repo_url = format!(
        "https://{}@github.com/silagokth/sylva-components.git",
        token
    );

    let temp_dir = format!("{}/temp_repo", dir);
    let target_subdir = "integration/alimp";
    let output_dir = format!("{}/alimp", dir);

    // 1. Clean existing directories (ignore if they don’t exist)
    if Path::new(&temp_dir).exists() {
        fs::remove_dir_all(&temp_dir)?;
    }

    if Path::new(&output_dir).exists() {
        fs::remove_dir_all(&output_dir)?;
    }

    // 2. Clone (no checkout)
    runc(Command::new("git").args([
        "clone",
        "--filter=blob:none",
        "--no-checkout",
        &repo_url,
        &temp_dir,
    ]))?;

    // 3. Sparse checkout setup
    runc(Command::new("git")
        .current_dir(&temp_dir)
        .args(["sparse-checkout", "init", "--cone"]))?;

    runc(Command::new("git")
        .current_dir(&temp_dir)
        .args(["sparse-checkout", "set", target_subdir]))?;

    runc(Command::new("git")
        .current_dir(&temp_dir)
        .arg("checkout"))?;

    // 4. Copy only desired directory
    let src_path = PathBuf::from(&temp_dir).join(target_subdir);
    copy_dir_all(&src_path, &output_dir)?;

    // 5. Cleanup temp repo
    fs::remove_dir_all(&temp_dir)?;


    Ok(())
}


#[allow(unused_variables)]
pub fn run(
    db: &mut DataBase, 
    dir: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Start: control");
    let module_dir = format!("{}/control", dir);
    match std::fs::create_dir_all(&module_dir) {
        Ok(_) => (),
        Err(e) => {
            error!("Failed to create {} with {}", module_dir, e);
            return Err(Box::new(e));
        },
    };
   
    /* Create an Alimp system environment */
    //get_alimp_sys(&module_dir)?;
    info!("Complete creating AlImp system");

    
    /* AlImp local control synthesis */
    let node_ids: Vec<String> = db.synthesized_information.alimp_bindings
        .iter()
        .map(|b| b.app_node_id.clone())
        .collect();

    for node_id in node_ids {
        info!("running AlImp Synthesis for {}", node_id);
        alimp_syn::main(db, &node_id, &module_dir)?;
    }
/*
    /* Glocal Control synthesis */
    info!("running global control synthesis");
    global_syn::main(db, module_dir);
*/
    info!("Finish: control");
    Ok(())
}

