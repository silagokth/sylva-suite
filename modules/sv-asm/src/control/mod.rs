use sv_lib::model::{DataBase}; 
//use sv_lib::file_handler;
use log::{info, error};
//use std::collections::{HashMap};
use std::path::{Path};


//pub mod utils;
mod alimp_syn;
//mod glocal_syn;


fn copy_dir_all(
    src: impl AsRef<Path>, 
    dst: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(&dst)?;

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dest_path = dst.as_ref().join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(entry.path(), &dest_path)?;
        } else {
            std::fs::copy(entry.path(), dest_path)?;
        }
    }

    Ok(())
}


fn verify_git_submodule(
    path: &str,
    expected_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {

    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(path)
        .arg("remote")
        .arg("get-url")
        .arg("origin")
        .output()?;

    if !output.status.success() {
        return Err(format!("{} is not a valid git repository", path).into());
    }

    let url = String::from_utf8(output.stdout)?.trim().to_string();

    if url != expected_url {
        return Err(format!(
            "Submodule mismatch:\n  expected: {}\n  got: {}",
            expected_url, url
        ).into());
    }

    Ok(())
}




fn get_rtl_framework(
    framework_dir: &String,
    dir: &String,
) -> Result<(), Box<dyn std::error::Error>> {

    let framework_path = Path::new(framework_dir);
    let work_dir = Path::new(dir).join("work");

    // ------------------ 1. verify repo ------------------
    verify_git_submodule(
        framework_dir,
        "https://github.com/silagokth/sylva-components.git"
    )?;   

    // ------------------ 2. clean work dir ------------------
    if work_dir.exists() {
        std::fs::remove_dir_all(&work_dir)?;
    }
    std::fs::create_dir_all(&work_dir)?;

    // ------------------ 3. copy folders ------------------
    let components_src = framework_path.join("components");
    let integration_src = framework_path.join("integration");

    let components_dst = work_dir.join("components");
    let integration_dst = work_dir.join("integration");

    if !components_src.exists() {
        return Err("Missing components directory".into());
    }

    if !integration_src.exists() {
        return Err("Missing integration directory".into());
    }

    copy_dir_all(&components_src, &components_dst)?;
    copy_dir_all(&integration_src, &integration_dst)?;

    Ok(())        
}


#[allow(unused_variables)]
pub fn run(
    db: &mut DataBase, 
    framework_dir: &String,
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
    get_rtl_framework(&framework_dir, &module_dir)?;
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

