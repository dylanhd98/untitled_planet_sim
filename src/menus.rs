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

//displays a graph showing the increase in triangles and vertices for each iteration
fn iteration_graph(){

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
    
    //amount of triangles multiplies by 4 for each iteration, starting at 20 at 0 iters
    let tri_no = 20*u32::pow(4, gen_info.iterations as u32);

    egui::CentralPanel::default()
        .show(egui_ctx,|ui| {
            ui.heading("Gen Info");
            ui.label("Vertex Count");
            //amount of vertices can be found by halving tri_no and adding 2
            ui.label(format!("{}",tri_no/2+2));
            ui.label("Triangle Count");
            ui.label(format!("{}",tri_no));

            let tri_counts: PlotPoints = (-10..=(1+gen_info.iterations as i8)*10).map(|i| {
                let i = i as f64/10.0;
                //let x = (20*u32::pow(4, i as u32))/2+2;
                let x = (20.0*f64::powf(4.0, i));
                [i, x]
            }).collect();
            let tri_line = Line::new(tri_counts);

            let vert_counts: PlotPoints = (-10..=(1+gen_info.iterations as i8)*10).map(|i| {
                let i = i as f64/10.0;
                let x = (20.0*f64::powf(4.0, i))/2.0+2.0;
                [i, x]
            }).collect();
            let vert_line = Line::new(vert_counts);

            let current = VLine::new(gen_info.iterations as f64);

            Plot::new("my_plot")
                .width(500.0)
                .view_aspect(0.75)
                .allow_scroll(false)
                .allow_zoom(false)
                .allow_drag(false)
                //.allow_boxed_zoom(false)
                //.show_background(false)
                .show(ui, |plot_ui| {
                    plot_ui.line(vert_line);
                    plot_ui.line(tri_line);
                    plot_ui.vline(current)
                } );
        });

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