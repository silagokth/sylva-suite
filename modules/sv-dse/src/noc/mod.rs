use sv_lib::model::{DataBase, Coordinate};
use log::{info, warn, debug, error};
use std::collections::{HashMap};
use regex::Regex;
use plotters::prelude::*;
use plotters::element::PointCollection;

mod insert;


fn to_wire_type(
    block: &HashMap<&str, Option<&str>>
) -> String {
    let mut label = String::from("wire_");
    
    for &dir_in in ["e", "n", "w", "s"].iter() {
        if let Some(Some(dir_out)) = block.get(dir_in) {
            label.push_str(dir_in);
            label.push_str(dir_out);
        }
    }

    if let Some(Some(_)) = block.get("reg") {
        label.push('r');
    }

    if let Some(Some(_)) = block.get("buffer") {
        label.push('b');
    }

    label
}

fn from_wire_type(
    wire: &String,
) -> Result<HashMap<&str, Option<&str>>, Box<dyn std::error::Error>> {
    let pattern = r"\bwire_?([enws]{2,})?(r)?(b)?\b";
    let re = Regex::new(pattern)?;

    let mut block = HashMap::from([
        ("e", None),
        ("n", None),
        ("w", None),
        ("s", None),
        ("reg", None),
        ("buffer", None),
    ]);
 
    if let Some(matches) = re.captures(wire) {
        let dir_str = matches.get(1).map_or("", |m| m.as_str());
        let split_dirs: Vec<&str> =  dir_str
            .as_bytes()
            .chunks(2)
            .map(|c| std::str::from_utf8(c).unwrap())
            .collect();
            
        for dir in split_dirs {
            let input = &dir[0..1];
            let output = &dir[1..2];
            block.insert(input, Some(output));
        }

        if matches.get(2).is_some() {
            block.insert("reg", Some("true"));
        }

        if matches.get(3).is_some() {
            block.insert("buffer", Some("true"));
        }
    }

    Ok(block)
}



fn noc_identification(
    db: &mut DataBase,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut assignment = HashMap::new();
    let template = HashMap::from([
        ("e", None),
        ("n", None),
        ("w", None),
        ("s", None),
        ("reg", None),
        ("buffer", None),
    ]);

    for routing_path in &db.synthesized_information.routing_paths {
       for node in &routing_path.path {
            let label = format!("{}_{}", node.x, node.y);
            if !assignment.contains_key(&label) {
                assignment.insert(label, template.clone());
            }
       }

       for i in 0..routing_path.path.len() {
            let coord = &routing_path.path[i];
            let coord_prev = if i == 0 {
                &Coordinate {x: coord.x, y: coord.y - 1}
            } else {
                &routing_path.path[i - 1]
            };
            let coord_next = if i == routing_path.path.len() - 1 {
                &Coordinate {x: coord.x, y: coord.y + 1}
            } else {
                &routing_path.path[i + 1]
            };

            let dir_in = match (coord_prev.x, coord_prev.y) {
                (p_x, p_y) if p_x == coord.x && p_y == coord.y - 1 => "s",
                (p_x, p_y) if p_x == coord.x && p_y == coord.y + 1 => "n",
                (p_x, p_y) if p_x == coord.x - 1 && p_y == coord.y => "w",
                (p_x, p_y) if p_x == coord.x + 1 && p_y == coord.y => "e",
                _ => return Err(format!("path is not continuous").into()),
            };

            let dir_out = match (coord_next.x, coord_next.y) {
                (n_x, n_y) if n_x == coord.x && n_y == coord.y - 1 => "s",
                (n_x, n_y) if n_x == coord.x && n_y == coord.y + 1 => "n",
                (n_x, n_y) if n_x == coord.x - 1 && n_y == coord.y => "w",
                (n_x, n_y) if n_x == coord.x + 1 && n_y == coord.y => "e",
                _ => return Err(format!("path is not continuous").into()),
            };
            
            if dir_in == dir_out {
                return Err(format!("self loop path detected").into());
            }

            // check availability
            let label = format!("{}_{}", coord.x, coord.y);
            let block = assignment.get_mut(&label).ok_or_else(|| {
                format!("label '{}' not found in assignment", label)
            })?;

            // 1. Check if incoming direction is already assigned
            if let Some(Some(_)) = block.get(dir_in) {
                return Err(format!("detected a conflict at block ({})", label).into());
            }
            // 2. check if dir_out is already used as any direction's value
            for &dir in ["e", "n", "w", "s"].iter() {
                if let Some(Some(value)) = block.get(dir) {
                    if *value == dir_out {
                        return Err(format!("detected a conflict at block ({})", label).into());
                    }
                }
            }
            
            // update assignment
            block.insert(dir_in, Some(dir_out));
       }
    }

    // update synthesized information
    for (label, block) in &assignment {
        db.synthesized_information.wire_assignment.insert(label.clone(), to_wire_type(block));
    }

    Ok(())
}



fn noc_synthesis(
    db: &mut DataBase,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut assignment = HashMap::new();
    let wire_assignment = db.synthesized_information.wire_assignment.clone();
    for (label, wire_type) in &wire_assignment {
        assignment.insert(label, from_wire_type(wire_type)?);
    }

    let mut all_paths = String::new();
    for routing_path in &mut db.synthesized_information.routing_paths {
        let design = insert::insert_main(
            &db.technology_constraint,
            routing_path.path.len() as i32,
        )?;

        if design.is_empty() {
            error!("fail to find a NoC solution ");
            return Err(format!("noc solution is not found for {}", routing_path.app_edge_id).into());
        }
        all_paths += &format!("\npath: {} - {}", routing_path.app_edge_id, design);
    
        // marks buffers and registers according to the design 
        let mut delay = 0;

        for (node, kind) in routing_path.path.iter().zip(design.chars()) {
            let label = format!("{}_{}", node.x, node.y);

            if !assignment.contains_key(&label) {
                error!("Block {} is not found in wire assignment", label);
                return Err(format!("Block {} is not found in wire assignment", label).into());
            }

            let block = assignment.get_mut(&label).ok_or_else(|| {
                format!("label '{}' not found in assignment", label)
            })?; 
            
            match kind {
                'b' => {
                    block.insert("buffer", Some("true"));
                }
                'r' => {
                    block.insert("reg", Some("true"));
                    delay += 1
                }
                _ => {}
            }
        }
        routing_path.delay = delay;
    }

    for (label, block) in &assignment {
        db.synthesized_information.wire_assignment.insert(label.to_string(), to_wire_type(block));
    }

    debug!("all noc solutions {}", all_paths);
    Ok(())
}


fn plot_graph(
    db: &DataBase,
    module_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
     
    let max_x = db.synthesized_information.max_width.clone();
    let max_y = db.synthesized_information.max_height.clone();
    let step_x_label = (max_x / 20) + 1;
    let step_y_label = (max_y / 20) + 1;
    let resolution = match max_x * max_y {
        a if a > 100_000 => 10000,
        a if a > 50_000 => 4000,
        a if a > 10_000 => 2000,
        a if a > 5_000 => 1000,
        _ => 800,
    } as u32;

    if resolution >= 10000 {
        warn!("the floorplan is too large to be presented in the graph!");
        return Ok(())
    }
    
    let output_file = format!("{}/layout_graph.png", module_dir);
    let root = BitMapBackend::new(&output_file, (resolution, resolution)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .margin(10)
        .caption("NoC Layout", ("sans-serif", 30))
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(
            -0.5f64..(max_x as f64 - 0.5),
            -0.5f64..(max_y as f64 - 0.5),
        )?;

    chart.configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .x_labels(((max_x / step_x_label) + 1) as usize)
        .y_labels(((max_y / step_y_label) + 1) as usize)
        .draw()?;

    // draw routing paths
    for path in &db.synthesized_information.routing_paths {
        for coord in &path.path {
            let x = coord.x as f64;
            let y = coord.y as f64;

            chart.plotting_area().draw(&Rectangle::new(
                [(x - 0.5, y - 0.5), (x + 0.5, y + 0.5)],
                RGBColor(173, 216, 230).filled(), // light blue
            ))?;

            let label = format!("{}_{}", coord.x, coord.y);
            let wire_type = db.synthesized_information.wire_assignment.get(&label).ok_or_else(|| {
                format!("label '{}' not found in assignment", label)
            })?;
            let block = from_wire_type(&wire_type)?;

            // register (diagonal red line)
            if let Some(Some(_)) = block.get("reg") {
                chart.plotting_area().draw(&PathElement::new(
                    vec![(x - 0.4, y - 0.4), (x + 0.4, y + 0.4)],
                    &RED,
                ))?;
            }

            // buffer (small red dot)
            if let Some(Some(_)) = block.get("buffer") {
                chart.plotting_area().draw(&Circle::new((x - 0.3, y + 0.3), 2, RED.filled()))?;
            }

            fn draw_array<T: DrawingBackend, C: CoordTranslate>(
                area: &DrawingArea<T, C>,
                start_x: f64,
                start_y: f64,
                end_x: f64,
                end_y: f64,
            ) -> Result<(), Box<dyn std::error::Error>> 
            where
                C::From: Into<(f64, f64)>,
                for<'a> &'a PathElement<(f64, f64)>: PointCollection<'a, <C as CoordTranslate>::From>,
                for<'a> &'a Polygon<(f64, f64)>: PointCollection<'a, <C as CoordTranslate>::From>,
                <T as DrawingBackend>::ErrorType: 'static,
            {
                area.draw(&PathElement::new(
                    vec![(start_x, start_y), (end_x, end_y)], 
                    ShapeStyle::from(&BLACK).stroke_width(2),
                ))?;
                        
                let arrow_head_size = 0.2; // Adjust size of the arrowhead
                let arrow_angle = std::f64::consts::PI / 6.0; // 30 degrees for arrow head

                let dx = end_x - start_x;
                let dy = end_y - start_y;
                let angle = dy.atan2(dx); // Angle of the line

                // calculate points for the two "wings" of the arrowhead
                let x1 = end_x - arrow_head_size * (angle - arrow_angle).cos();
                let y1 = end_y - arrow_head_size * (angle - arrow_angle).sin();

                let x2 = end_x - arrow_head_size * (angle + arrow_angle).cos();
                let y2 = end_y - arrow_head_size * (angle + arrow_angle).sin();
                
                // draw arrowhead
                area.draw(&PathElement::new(
                    vec![(end_x, end_y), (x1, y1)],
                    ShapeStyle::from(&BLACK).stroke_width(2),
                ))?;
                area.draw(&PathElement::new(
                    vec![(end_x, end_y), (x2, y2)],
                    ShapeStyle::from(&BLACK).stroke_width(2),
                ))?;

                area.draw(&Polygon::new(
                    vec![(end_x, end_y), (x1, y1), (x2, y2)],
                    BLACK.filled(),
                ))?;
            
                Ok(())
            }

            // directional arrows
            match block.get("n") {
                Some(Some("e")) => draw_array(chart.plotting_area(), x, y + 0.5, x + 0.5, y)?,
                Some(Some("w")) => draw_array(chart.plotting_area(), x, y + 0.5, x - 0.5, y)?,
                Some(Some("s")) => draw_array(chart.plotting_area(), x, y + 0.5, x, y - 0.5)?,
                _ => {}
            }
            match block.get("s") {
                Some(Some("e")) => draw_array(chart.plotting_area(), x, y - 0.5, x + 0.5, y)?,
                Some(Some("w")) => draw_array(chart.plotting_area(), x, y - 0.5, x - 0.5, y)?,
                Some(Some("n")) => draw_array(chart.plotting_area(), x, y - 0.5, x, y + 0.5)?,
                _ => {}
            }
            match block.get("e") {
                Some(Some("w")) => draw_array(chart.plotting_area(), x + 0.5, y, x - 0.5, y)?,
                Some(Some("s")) => draw_array(chart.plotting_area(), x + 0.5, y, x, y - 0.5)?,
                Some(Some("n")) => draw_array(chart.plotting_area(), x + 0.5, y, x, y + 0.5)?,
                _ => {}
            }
            match block.get("w") {
                Some(Some("s")) => draw_array(chart.plotting_area(), x - 0.5, y, x, y - 0.5)?,
                Some(Some("n")) => draw_array(chart.plotting_area(), x - 0.5, y, x, y + 0.5)?,
                Some(Some("e")) => draw_array(chart.plotting_area(), x - 0.5, y, x + 0.5, y)?,
                _ => {}
            }
        }
    }

    // draw grid
    for i in 0..max_x {
        chart.draw_series(LineSeries::new(
            vec![(i as f64 - 0.5, -0.5), (i as f64 - 0.5, max_y as f64 - 0.5)],
            ShapeStyle::from(&RGBColor(200, 200, 200)).stroke_width(1),
        ))?;
    }
    for i in 0..max_y {
        chart.draw_series(LineSeries::new(
            vec![(-0.5, i as f64 - 0.5), (max_x as f64 - 0.5, i as f64 - 0.5)],
            ShapeStyle::from(&RGBColor(200, 200, 200)).stroke_width(1),
        ))?;
    }

    // draw nodes and buffers 
    for node in &db.app_graph.nodes {
        let (mut x, mut y, mut w, mut h) = (-1, -1, -1, -1);

        for placement in &db.synthesized_information.placements {
            if placement.app_node_id == node.id {
                x = placement.x;
                y = placement.y;
                break;
            }
        }

        for binding in &db.synthesized_information.alimp_bindings {
            if binding.app_node_id == node.id {
                w = binding.alimp_instance.width * db.technology_constraint.width_ratio;
                h = binding.alimp_instance.height * db.technology_constraint.height_ratio;
                break;
            }
        }

        if x == -1 || y == -1 || w == -1 || h == -1 {
            return Err(format!("Missing placement/binding for node {}", node.id).into());
        }

        let (step_x, step_y) = (db.technology_constraint.width_ratio, db.technology_constraint.height_ratio);
        let (step_x_f, step_y_f) = (step_x as f64, step_y as f64);

        // Node rectangle
        // Orange: Main node
        for i in (x..(x + w)).step_by(step_x as usize) {
            for j in (y..(y + h)).step_by(step_y as usize) {
                let (i_f, j_f) = (i as f64, j as f64);
                chart.draw_series(std::iter::once(Rectangle::new(
                    [(i_f - 0.30, j_f - 0.30), (i_f - 0.70 + step_x_f, j_f - 0.70 + step_y_f)],
                    RGBColor(255, 165, 0).filled(), // Orange
                )))?;
            }
        }


        if node.input_ports.len() > 0 {
            // Input buffer (purple, below node)
            for i in (x..(x + w)).step_by(step_x as usize) {
                for j in ((y - step_y)..y).step_by(step_y as usize) {
                    let (i_f, j_f) = (i as f64, j as f64);
                    chart.draw_series(std::iter::once(Rectangle::new(
                        [(i_f - 0.30, j_f - 0.30), (i_f - 0.70 + step_x_f, j_f - 0.70 + step_y_f)],
                        RGBColor(160, 32, 240).filled(), // Purple
                    )))?;
                }
            }
        }
        
        if node.output_ports.len() > 0 {
            // Output buffer (red, above node)
            for i in (x..(x + w)).step_by(step_x as usize) {
                for j in ((y + h)..(y + h + step_y)).step_by(step_y as usize) {
                    let (i_f, j_f) = (i as f64, j as f64);
                    chart.draw_series(std::iter::once(Rectangle::new(
                        [(i_f - 0.30, j_f - 0.30), (i_f - 0.70 + step_x_f, j_f - 0.70 + step_y_f)],
                        RED.filled(),
                    )))?;
                }
            }

            // Transporter (green, above buffer)
            for i in (x..(x + w)).step_by(step_x as usize) {
                for j in ((y + h + step_y)..(y + h + (2 * step_y))).step_by(step_y as usize) {
                    let (i_f, j_f) = (i as f64, j as f64);
                    chart.draw_series(std::iter::once(Rectangle::new(
                        [(i_f - 0.30, j_f - 0.30), (i_f - 0.70 + step_x_f, j_f - 0.70 + step_y_f)],
                        GREEN.filled(),
                    )))?;
                }
            }
        }
    }

    // add node labels 
    for node in &db.app_graph.nodes {
        let (mut x, mut y, mut w, mut h) = (-1, -1, -1, -1);

        for placement in &db.synthesized_information.placements {
            if placement.app_node_id == node.id {
                x = placement.x;
                y = placement.y;
                break;
            }
        }

        for binding in &db.synthesized_information.alimp_bindings {
            if binding.app_node_id == node.id {
                w = binding.alimp_instance.width * db.technology_constraint.width_ratio;
                h = binding.alimp_instance.height * db.technology_constraint.height_ratio;
                break;
            }
        }

        if x == -1 || y == -1 || w == -1 || h == -1 {
            return Err(format!("Missing placement/binding for node {}", node.id).into());
        }

        let x_f = x as f64;
        let y_f = y as f64;
        let w_f = w as f64;
        let h_f = h as f64;

        // Label
        chart.plotting_area().draw(&Text::new(
            &node.id[..],
            (x_f + (w_f - 1.0) / 5.0, y_f + h_f / 2.0),
            ("sans-serif", 50.0).into_font().color(&BLACK),
        ))?;
    }

    root.present()?;
    info!("Saved layout to: {}", output_file);
    Ok(())
}



#[allow(unused_variables)]
pub fn run(
    db: &mut DataBase, 
    dir: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Start: noc synthesis");
    let module_dir = format!("{}noc", dir);
    match std::fs::create_dir_all(&module_dir) {
        Ok(_) => (),
        Err(e) => {
            error!("Failed to create {} with {}", module_dir, e);
            return Err(Box::new(e));
        },
    };

    info!("Stage 1: noc block identification");
    noc_identification(db)?;
    
    info!("Stage 2: noc synthesis and update information");
    noc_synthesis(db)?;

    info!("Stage 3: generate graph");
    plot_graph(&db, &module_dir)?; 

    info!("Finish: noc synthesis");
    Ok(())
}
