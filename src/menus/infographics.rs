
use core::ops::Range;
use std::f64;
//external crates used
use egui::{Context,plot::{Line, Plot, PlotPoints,VLine, Polygon, Legend, Corner}, Color32};
use nalgebra_glm as glm;

use crate::planet::GenInfo;

//UTILITY FUNCTIONS FOR INFO MENUS

//creates lines representing a subdivided triangle, 
//much more efficient than rendering large amount of polygons as have tested previously
//as the look of the triangles is just a result of overlapping lines, no actual internal points are specified, much better method
fn subdivided_tri_lines(iterations:u8)->Vec<Line>{
    //the percentage of the original line each point will be apart form eachother
    let step_size = 1.0/f32::powf(2.0, iterations as f32);
    //number of points that will be in the new lines
    let point_number = usize::pow(2, iterations as u32)+1;
    //starts with original clockwise triangle lines
    let mut tri_lines = vec![
        vec![glm::vec2(-1.0, -1.0),glm::Vec2::y()],//bottom right to top
        vec![glm::Vec2::y(),glm::vec2(1.0, -1.0)],//top to bottom left
        vec![glm::vec2(1.0, -1.0),glm::vec2(-1.0, -1.0)]//bottom left to bottom right
    ];
    //loop through each line, turn into lines with specified amount of points
    tri_lines = tri_lines
        .into_iter()
        .map(|line| {
            let new_line:Vec<glm::Vec2> =(0..point_number)
                .into_iter()
                .map(|point|{
                    //percentage along line the point being added is
                    let line_percentage = step_size*point as f32;
                    //interpolate the two original values in line with this percentage to get new point
                    glm::lerp(&line[0], &line[1], line_percentage)
                }).collect();
            new_line
        }).collect();

    //vec for lines about to be created
    let mut plot_lines:Vec<Line> = Vec::with_capacity(3*point_number);
    //loop through lines, connecting point in one line to the point in the next clockwise line at its index mirrored
    //e.g. if lines are 10 long, a point at 1 is connected to point 8 on the next line
    //last point in each current line is ignored, as it would connect to the start of the next line which would be at its own pos
    for line_no in 0..tri_lines.len(){
        //loop through points in lines, create line from that point to its inverse in the next line
        for point_no in 0..point_number{
            //get both points in line
            let a = tri_lines[line_no][point_no];
            let b = tri_lines[(line_no+1)%3][point_number-point_no-1];
            //turn the two points into appropriate data, use to make a line
            let line = Line::new( vec![
                [a.x as f64,a.y as f64],
                [b.x as f64,b.y as f64]]);
            //add line to vec
            plot_lines.push(line);
        }
    }
    //return lines to be plotted
    plot_lines
}

//creates a line for a plot from a given closure that returns an f64 
fn plot_func<G>(mut input_range:Range<i32>,sample_rate:u32,func:G)->Line
where G: Fn(f64)->f64{//type G is a function
    //extend range to required res
    input_range.start *= sample_rate as i32;
    input_range.end *= sample_rate as i32;

    //sample points at scale
    let points: PlotPoints = input_range
        .map(|i| {
            //scale back into original range
            [i as f64/sample_rate as f64, func(i as f64/sample_rate as f64)]
        }).collect();
    Line::new(points)
}

//plots an arc between two angles on the circumfrence of a circle
fn plot_arc(pos:[f64;2],radius:f64,start:f64,end:f64)->Line{
    let step_size = std::f64::consts::PI/200.0;
    let range = if start<end{
        (start*200.0) as i32..=(end*200.0) as i32
    }else{
        (end*200.0) as i32..=(start*200.0) as i32
    };
    let arc:PlotPoints = range
        .map(|x| {
            let t = x as f64 * step_size;
            [radius*(f64::cos(t)+pos[0]),radius*(f64::sin(t)+pos[1])]
        }).collect();
    Line::new(arc)
}

//creates line rotated about a point
fn plot_line(pos:[f64;2],size:f64,rotation:f64){
    
}

//creates circle given pos and radius
fn plot_circle(pos:[f64;2],radius:f64)->Polygon{
    let step_size = std::f64::consts::PI/100.0;
    let circle:PlotPoints = (-100..100)
        .map(|x| {
            let t = x as f64 * step_size;
            [radius*(f64::cos(t)+pos[0]),radius*(f64::sin(t)+pos[1])]
        }).collect();
    Polygon::new(circle)
}

//ACTUALL MENUS DISPLAYING THE INFORMATION

//diplays general information about the sim
pub fn intro_info(egui_ctx: &Context){
    egui::CentralPanel::default()
    .show(egui_ctx, |ui| {
        ui.heading("Untitled Planet Sim");
        ui.separator();
        ui.heading("Controls");
        ui.label("To move the camera you can use both mouse and keyboard.\nTo move the camera you can click and drag with middle or right click, or use wasd.\nTo zoom you can scroll or press e and q to zoom in and out respectivly.");
    });
}

//menu showing the user information about subdividing the mesh
pub fn subdivision_info(egui_ctx: &Context, gen_info: &GenInfo){
        //amount of triangles that will be generated at current iteration
        let tri_count = 20*u32::pow(4, gen_info.iterations as u32);
        //amount of vertices that will be generated at current iteration
        let vert_count = tri_count/2+2;
        //plot lines showing the tri count and vertex count at all iterations previous current and one above
        //intends to show the user why preformance may quickly drop with iterations
        let tri_line = plot_func(-1..(1+gen_info.iterations).into(), 50, |x| 20.0*f64::powf(4.0, x));
        let vert_line = plot_func(-1..(1+gen_info.iterations).into(), 50, |x| (20.0*f64::powf(4.0, x))/2.0+2.0);
        //plot vertical line at value current iteration amount selected, as reference for the user on a graph
        let iter_line = VLine::new(gen_info.iterations);
        //generates lines to be drawn that show what subdivisions on a triangle look like
        let triangle_lines:Vec<Line> = subdivided_tri_lines(gen_info.iterations);

        egui::CentralPanel::default()
        .show(egui_ctx,|ui| {
            //gets dimensons of central panel for later
            let panel_size = ui.available_size();
            ui.heading("Shape Subdivision Info");
            ui.separator();
            ui.label("The amount of times each triangle on the original icosahedron is subdivided, this is done to create a sphereical planet with similarly distanced points at a user specified resolution.");
            ui.separator();

            ui.horizontal(|ui|{
                //plot graph of amount of triangles and points
                Plot::new("subdivision graph")
                    .width(panel_size.x*0.5)
                    .height(panel_size.y*0.5)
                    .allow_scroll(false)
                    .allow_zoom(false)
                    .allow_drag(false)
                    .allow_boxed_zoom(false)
                    .legend(Legend::default().position(Corner::LeftTop).background_alpha(1.0))
                    .show(ui, |plot_ui| {
                        plot_ui.line(tri_line.name(format!("Triangle Count ({})",tri_count)));
                        plot_ui.line(vert_line.name(format!("Vetex Count  ({})",vert_count)));
                        plot_ui.vline(iter_line.name(format!("Current Iterations ({})",gen_info.iterations)));
                    } );

                //graph displaying subdivided triangle, meant to just look like still image
                Plot::new("triangle graph")
                    .width(panel_size.x*0.5)
                    .height(panel_size.y*0.5)
                    .allow_scroll(false)
                    .allow_zoom(false)
                    .allow_drag(false)
                    .allow_boxed_zoom(false)
                    .show_axes([false;2])
                    .show_background(false)
                    .show_y(false)
                    .show_x(false)
                    .show(ui, |plot_ui| {
                        //plot the lines of the triangle
                        for line in triangle_lines{
                            plot_ui.line(line);
                        }
                    } );
            }); 

        });
}

//displays circle and line at angle intersecting with it to represent axial tilt
pub fn axial_tilt_info(egui_ctx: &Context, gen_info: &GenInfo){
    egui::CentralPanel::default()
        .show(egui_ctx, |ui| {
            ui.heading("Axial Tilt Info");
            ui.separator();
            ui.label("The axial tilt of the planet, the angle between the pole of the planet and the normal of its orbital plane. Determines the seasons of the planet due to the angle the light hits different latitudes differing around  the year.");
            ui.separator();

            Plot::new("axial diagram")
            .data_aspect(1.0)
            .allow_scroll(false)
            .allow_zoom(false)
            .allow_drag(false)
            .allow_boxed_zoom(false)
            .show_axes([false;2])
            .show_background(false)
            .show_y(false)
            .show_x(false)
            .legend(Legend::default().position(Corner::LeftTop))
            .show(ui, |plot_ui| {
                //plot unit circle
                plot_ui.polygon(plot_circle([0.0,0.0], 1.0)
                    .color(Color32::LIGHT_BLUE));
                //plot orbital axis and plane
                let points = vec![
                    [0.0,1.5],
                    [0.0,-1.5]
                ];
                plot_ui.line(Line::new(points)
                    .color(Color32::GRAY)
                    .name("Orbital Axis"));
                
                //plot rotational axis
                let rotation = (gen_info.axial_tilt+0.5)*std::f32::consts::PI;
                //creates line at angle of axis
                let axis_point = 1.5*glm::vec2(f32::cos(rotation), f32::sin(rotation));
                let points = vec![
                    [axis_point.x as f64,axis_point.y as f64],
                    [-axis_point.x as f64,-axis_point.y as f64]
                ];
                plot_ui.line(Line::new(points)
                    .color(Color32::LIGHT_RED)
                    .name("Rotational Axis"));
                
                //plot axial tilt
                plot_ui.line(plot_arc([0.0,0.0], 1.25, 0.5, gen_info.axial_tilt as f64 + 0.5)
                    .name(format!("Axial Tilt\nRadians ({}π)\nDegrees ({}°)",gen_info.axial_tilt,gen_info.axial_tilt*180.0)));

                //now create equator line
                let rotation = (gen_info.axial_tilt)*std::f32::consts::PI;
                let axis_point = glm::vec2(f32::cos(rotation), f32::sin(rotation));
                let points = vec![
                    [axis_point.x as f64,axis_point.y as f64],
                    [-axis_point.x as f64,-axis_point.y as f64]
                ];
                plot_ui.line(Line::new(points)
                    .color(Color32::LIGHT_GREEN)
                    .style(egui::plot::LineStyle::dashed_dense())
                    .name("Equator"));
            });
                
        });
}