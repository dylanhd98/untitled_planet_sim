use std::fmt::format;

use egui::Context;
use glium::{Display,DrawParameters};
use nalgebra_glm as glm;

use crate::{GenInfo, GameState,planet, graphics};

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

    egui::CentralPanel::default()
        .show(egui_ctx,|ui| {
            ui.label("Shape Subdivision Iterations");
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
        }
    );

    if new_planet{
        //creates new camera
        let dimensions = display.get_framebuffer_dimensions();
        let mut cam = graphics::camera::Camera::new(dimensions.0 as f32/dimensions.1 as f32, 
            glm::vec3(0.0,0.0,5.0), 
            glm::vec3(0.0,0.0,0.0),
            glm::vec3(0.0,1.0,0.0));

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