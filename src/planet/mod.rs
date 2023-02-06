//external crates
use glium::{Surface,glutin};
use nalgebra_glm as glm;
use rand::rngs::ThreadRng;

//other internal modules
use crate::graphics;

use self::surface::CellData;

//child modules
pub mod surface;
pub mod utils;

#[derive(PartialEq)]
#[derive(Debug)]
pub enum LightPosition{
    Sun,
    Camera,
    Fixed
}

#[derive(PartialEq)]
#[derive(Clone, Copy)]
#[derive(Debug)]
pub enum MapMode{
    Natural,
    Height,
    Temperature,
    Humidity,
    Relief,
    Normals,
}

//info used for generating planet
pub struct GenInfo{
    pub iterations :u8,
    pub seed:u32,
    pub plate_no:u32,
    pub axial_tilt:f32,
    pub lapse_rate: f32,
    pub greenhouse_effect:f32
}

//information for the general running of the simulation, not specific to cells or surface
pub struct SimInfo{
    //years passing per second in the sim
    pub years_per_second: f32,
    //how long passes between updating the tectonics
    pub triangulation_interval: f32,
    //how much temp falls with altitude, C/km
    pub lapse_rate: f32,
    //percentage of energy retained from the sun
    pub greenhouse_effect:f32,
    //luminosity of the sun
    //pub solar_luminosity:f32,
    //planet's axis
    pub axis: glm::Vec3,
    //vector pointing to orbital center
    pub to_sun: glm::Vec3,
}

//struct containing all things needed passed to the gpu
pub struct RenderData{
    //buffer containing cell data needed for rendering
    planet_data: glium::VertexBuffer<surface::CellData>,
    //indices, define triangles of planet
    indices: glium::IndexBuffer<u32>,
    //how exagerated the planet surface will be
    pub scale: f32,
    //where the light source is
    pub light_pos: LightPosition,
    //map mode to use when displaying the planet
    pub map_mode: MapMode,
}

pub struct Planet{
    //data related to rendering the planet and for the shaders
    pub render_data: RenderData,
    //data describing the planets surface
    pub surface: surface::Surface,
    //infromation used for the general running of the simulation
    pub sim_info: SimInfo
}
impl Planet{
    pub fn new(display:&glium::Display, gen:&GenInfo)->Planet{
        //creats rng
        let mut rng = rand::thread_rng();
        //tilts planet axis as specified
        let axis = glm::rotate_z_vec3( &glm::vec3(0.0,1.0,0.0),gen.axial_tilt);

        //generates base shape
        let base_shape = graphics::shapes::Shape::icosahedron()
            .subdivide(gen.iterations)
            .normalize();
        
        //creates planet surface
        let mut surface = surface::Surface::new(base_shape,gen);

        //extract data for buffer
        let surface_contents:Vec<CellData> = surface.cells.iter()
            .map(|c|
            c.contents)
            .collect();

        Planet{
            render_data: 
            RenderData{
                //dynamic as this will change frequently
                planet_data: glium::VertexBuffer::dynamic(display, &surface_contents).unwrap(),
                
                indices: glium::IndexBuffer::new(display,glium::index::PrimitiveType::TrianglesList, &surface.triangles).unwrap(),

                scale: 0.01,

                light_pos: LightPosition::Fixed,

                map_mode: MapMode::Natural,
            },

            surface: surface,

            sim_info: 
            SimInfo { 
                years_per_second: 0.0, 
                triangulation_interval: 1_000_000.0,
                lapse_rate: gen.lapse_rate,
                //solar_luminosity: 1.0,
                greenhouse_effect: gen.greenhouse_effect, 
                axis: axis, 
                to_sun: glm::vec3(1.0,0.0,0.0),
            }
        }
    }

    pub fn update(&mut self, deltatime: f32,display:&glium::Display){
        let years_past = deltatime*self.sim_info.years_per_second;

        self.surface.tectonics(years_past,  &mut self.sim_info);
        self.surface.temperature(years_past, &self.sim_info);

        //one year is 360 days here for simplicity
        self.sim_info.to_sun= glm::rotate_y_vec3(&self.sim_info.to_sun, years_past*(std::f32::consts::PI*2.0));
        
        //extract data needed for rendering out
        let surface_contents:Vec<CellData> = self.surface.cells.iter()
            .map(|c|
                c.contents
            )
            .collect();

        //write surface contents to planet buffer
        self.render_data.planet_data.write(&surface_contents);
        //FIX THIS, WORKS BUT BAD
        self.render_data.indices= glium::IndexBuffer::new(display,glium::index::PrimitiveType::TrianglesList, &self.surface.triangles).unwrap();        ;
    }

    pub fn draw(&self, target:&mut glium::Frame, program:&glium::Program, params:&glium::DrawParameters,cam:&graphics::camera::Camera){
        let pers:[[f32;4];4] = cam.perspective.into();
        let view:[[f32;4];4] = cam.view.into();

        let to_light:[f32;3] = match self.render_data.light_pos{
            LightPosition::Sun=> self.sim_info.to_sun.into(),
            LightPosition::Camera=> cam.pos.normalize().into(),
            LightPosition::Fixed=> [0.0,0.0,1.0],
        };

        let uniform = glium::uniform!{
            perspective:pers,
            view: view,
            to_light: to_light,
            terra_scale: self.render_data.scale,
            map_mode: self.render_data.map_mode as i32,
        };

        target.draw(&self.render_data.planet_data,&self.render_data.indices,program,&uniform,params).unwrap();
    }
}