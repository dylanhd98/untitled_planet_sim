use std::ops::Range;

//external crates
use egui::{Context,plot::{Line, Plot, PlotPoints,VLine,Bar, BarChart}};
use glium::{Display,DrawParameters};
use nalgebra_glm as glm;

//internal modules
use crate::{GenInfo, GameState,planet, graphics};

//functions used by menus
//amount of triangles multiplies by 4 for each iteration, starting at 20 at 0 iters
fn tri_count_at_n(n:u32)->u32{
    20*u32::pow(4, n)
}
//amount of vertices can be found by halving tri_no and adding 2
fn vert_count_from_tri(tris:u32)->u32{
    tris/2+2
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


//displays a graph showing the increase in triangles and vertices for each iteration, and a triangle sudivided according to iterations
fn iteration_info(egui_ctx: &Context,gen_info: &GenInfo){
    //amount of triangles multiplies by 4 for each iteration, starting at 20 at 0 iters
    let tri_no = 20*u32::pow(4, gen_info.iterations as u32);

    egui::CentralPanel::default()
        .show(egui_ctx,|ui| {
            ui.heading("Shape Subdivision Info");
            ui.label("Triangle Count");
            ui.label(format!("{}",tri_no));
            ui.label("Vertex Count");
            //amount of vertices can be found by halving tri_no and adding 2
            ui.label(format!("{}",tri_no/2+2));
            
            let tri_line = plot_func(-1..(1+gen_info.iterations).into(), 50, |x| 20.0*f64::powf(4.0, x));

            let vert_line = plot_func(-1..(1+gen_info.iterations).into(), 50, |x| (20.0*f64::powf(4.0, x))/2.0+2.0);

            let little_line = plot_func(-1..1, 50, |x| x);

            let current = VLine::new(gen_info.iterations as f64);

            ui.horizontal(|ui|{
                Plot::new("subdivision graph")
                    .allow_scroll(false)
                    .allow_zoom(false)
                    .allow_drag(false)
                    .allow_boxed_zoom(false)
                    //.show_background(false)
                    .show(ui, |plot_ui| {
                        plot_ui.line(vert_line);
                        plot_ui.line(tri_line);
                        plot_ui.vline(current);
                    } );

                Plot::new("triangle graph")
                    .allow_scroll(false)
                    .allow_zoom(false)
                    .allow_drag(false)
                    .allow_boxed_zoom(false)
                    .show_background(false)
                    .show(ui, |plot_ui| {
                        plot_ui.line(little_line);
                    } );
            }); 

        });
}

//menu for planet creation
pub fn planet_create(egui_ctx: &Context,display: &Display,game_state: &mut GameState){

    let mut new_planet = false;
    let gen_info;

    //makes sure game state is the intended one for this menu
    if let GameState::Generate(ref mut gen)=  game_state{
        gen_info=gen;
    }else{
        return;
    }

    egui::SidePanel::left("gen panel")
        .show(egui_ctx,|ui| {
            ui.label("Shape Subdivisions");
            ui.add(egui::Slider::new(&mut gen_info.iterations, 0..=7));

            ui.label("Plate Amount");
            ui.add(egui::Slider::new(&mut gen_info.plate_no, 0..=50));

            ui.label("Axial Tilt");
            ui.add(egui::Slider::new(&mut gen_info.axial_tilt, 0.0..=(2.0*3.141592653)));

            ui.label("Lapse Rate");
            ui.add(egui::Slider::new(&mut gen_info.lapse_rate, 0.0..=25.0));

            ui.label("Greenhouse Effect");
            ui.add(egui::Slider::new(&mut gen_info.greenhouse_effect, 0.0..=1.0));

            ui.label("Seed");
            ui.add(egui::DragValue::new(&mut gen_info.seed).speed(0));
            
            if ui.button("CREATE PLANET").clicked(){
                new_planet = true;
            }
        });
        
    iteration_info(egui_ctx, gen_info);

    if new_planet{
        //creates new camera
        let dimensions = display.get_framebuffer_dimensions();
        let cam = graphics::camera::Camera::new(dimensions.0 as f32/dimensions.1 as f32, 
            glm::vec3(0.0,0.0,5.0), 
            glm::Vec3::zeros(),
            glm::Vec3::y());

        //creates new planet with set perameters
        let planet = planet::Planet::new(&display, &gen_info);
        *game_state= GameState::Playing(planet, cam);
    }
}

//menus for during the simulation
pub fn playing(egui_ctx: &Context,params: &mut DrawParameters,planet:&mut planet::Planet){
    //left side panel for controls
    egui::SidePanel::left("Left Panel").resizable(false)
    .show(egui_ctx,|ui| {
        ui.label("Years Per Second");
        ui.add(egui::Slider::new(&mut planet.sim_info.years_per_second, 0.0..=1000000.0).logarithmic(true));

        ui.label("Terrain Scaling");
        ui.add(egui::Slider::new(&mut planet.render_data.scale, 0.0..=0.05));

        ui.label("Lapse Rate");
        ui.add(egui::Slider::new(&mut planet.sim_info.lapse_rate, 0.0..=25.0));

        ui.label("Greenhouse Effect");
        ui.add(egui::Slider::new(&mut planet.sim_info.greenhouse_effect, 0.0..=1.0));

        ui.label("Light Source");
        egui::ComboBox::from_id_source("lighting")
            .selected_text(format!("{:?}", planet.render_data.light_pos))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut planet.render_data.light_pos, planet::LightPosition::Sun, "Sun");
                ui.selectable_value(&mut planet.render_data.light_pos, planet::LightPosition::Camera, "Camera");
                ui.selectable_value(&mut planet.render_data.light_pos, planet::LightPosition::Fixed, "Fixed");
            }
        );

        ui.label("Polygon Mode");
        egui::ComboBox::from_id_source("polygon")
            .selected_text(format!("{:?}", params.polygon_mode))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut params.polygon_mode, glium::PolygonMode::Fill, "Fill");
                ui.selectable_value(&mut params.polygon_mode, glium::PolygonMode::Line, "Line");
                ui.selectable_value(&mut params.polygon_mode, glium::PolygonMode::Point, "Point");
            }
        );

        ui.label("Map Mode");
        egui::ComboBox::from_id_source("map_mode")
            .selected_text(format!("{:?}", planet.render_data.map_mode))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut planet.render_data.map_mode, planet::MapMode::Natural, "Natural");
                ui.selectable_value(&mut planet.render_data.map_mode, planet::MapMode::Height, "Height");
                ui.selectable_value(&mut planet.render_data.map_mode, planet::MapMode::Temperature, "Temperature");
                ui.selectable_value(&mut planet.render_data.map_mode, planet::MapMode::Humidity, "Humidity");
                ui.selectable_value(&mut planet.render_data.map_mode, planet::MapMode::Relief, "Relief");
                ui.selectable_value(&mut planet.render_data.map_mode, planet::MapMode::Normals, "Normals");
            }
        );
    });
}