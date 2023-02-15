
use core::ops::Range;
//external crates used
use egui::{Context,plot::{Line, Plot,PlotUi, PlotPoints,VLine,Bar, BarChart, Polygon}};
use nalgebra_glm as glm;

use crate::planet::GenInfo;

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

pub fn subdivision_info(egui_ctx: &Context, gen_info: &GenInfo){
        let tri_count = 20*u32::pow(4, gen_info.iterations as u32);
                
        let vert_count = tri_count/2+2;

        let tri_line = plot_func(-1..(1+gen_info.iterations).into(), 50, |x| 20.0*f64::powf(4.0, x));

        let vert_line = plot_func(-1..(1+gen_info.iterations).into(), 50, |x| (20.0*f64::powf(4.0, x))/2.0+2.0);

        let iter_line = VLine::new(gen_info.iterations);

        let triangle_lines:Vec<Line> = subdivided_tri_lines(gen_info.iterations);

        egui::CentralPanel::default()
        .show(egui_ctx,|ui| {
            ui.heading("Shape Subdivision Info");
            ui.label("Triangle Count");
            ui.label(format!("{}",tri_count));
            ui.label("Vertex Count");
            //amount of vertices can be found by halving tri_no and adding 2
            ui.label(format!("{}",vert_count));

            ui.horizontal(|ui|{
                Plot::new("subdivision graph")
                    .width(500.0)
                    .height(500.0)
                    .allow_scroll(false)
                    .allow_zoom(false)
                    .allow_drag(false)
                    .allow_boxed_zoom(false)
                    .show(ui, |plot_ui| {
                        plot_ui.line(tri_line);
                        plot_ui.line(vert_line);
                        plot_ui.vline(iter_line);
                    } );

                Plot::new("triangle graph")
                    .width(500.0)
                    .height(500.0)
                    .allow_scroll(false)
                    .allow_zoom(false)
                    .allow_drag(false)
                    .allow_boxed_zoom(false)
                    .show_axes([false;2])
                    .show_background(false)
                    .show(ui, |plot_ui| {
                        //plot the lines of the triangle
                        for line in triangle_lines{
                            plot_ui.line(line);
                        }
                    } );
            }); 

        });
}