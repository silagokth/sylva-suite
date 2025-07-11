use crate::model::{DataBase, FloorPlan, RectangleShape, RectanglePosition, Placement};
use log::{info, warn, error, debug};

mod graph;

pub fn run(
    db: &mut DataBase, 
    dir: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Start: routing");
    let module_dir = format!("{}route", dir);
    match std::fs::create_dir_all(&module_dir) {
        Ok(_) => (),
        Err(e) => {
            error!("Failed to create {} with {}", module_dir, e);
            return Err(Box::new(e));
        },
    };

    info!("Stege 1: creating floor plan");
    //let mut route_graph = create_routing_graph(db)?;

    info!("Finish: routing");
    Ok(())
}
