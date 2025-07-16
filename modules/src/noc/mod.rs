use crate::model::{DataBase};
use log::{info, error};

mod insert;


pub fn run(
    db: &mut DataBase, 
    dir: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Start: noc synthesis");
    let module_dir = format!("{}route", dir);
    match std::fs::create_dir_all(&module_dir) {
        Ok(_) => (),
        Err(e) => {
            error!("Failed to create {} with {}", module_dir, e);
            return Err(Box::new(e));
        },
    };

    info!("Stage 1: noc block identification");

    info!("Stage 2: noc insertion");

    info!("Stage 3: update and generate result");

    info!("Finish: noc synthesis");
    Ok(())
}
