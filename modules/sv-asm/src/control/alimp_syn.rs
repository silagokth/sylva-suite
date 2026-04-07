use sv_lib::model::{DataBase, Control}; 


pub fn main(
    db: &mut DataBase, 
    node_id: &String,
    dir: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    let module_dir = format!("{}/{}", dir, node_id);
    match std::fs::create_dir_all(&module_dir) {
        Ok(_) => (),
        Err(e) => {
            error!("Failed to create {} with {}", module_dir, e);
            return Err(Box::new(e));
        },
    };
    


    Ok(())
}

