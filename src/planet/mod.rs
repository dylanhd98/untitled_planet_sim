use glium::{Surface,glutin};
use nalgebra_glm as glm;
use rand::Rng;
use noise::{NoiseFn, Perlin, Seedable};

use crate::graphics;

#[derive(Copy, Clone)]
pub struct CellData {
    pub height: f32,
    pub humidity: f32,
    pub temperature: f32
}
glium::implement_vertex!(CellData,height,humidity,temperature);

struct PlanetBuffers{
    shape_data :glium::VertexBuffer<graphics::PosNorm>,
    planet_data: glium::VertexBuffer<CellData>,
    indices: glium::IndexBuffer<u32>,
}

fn octive_noise(perlin: Perlin, pos:&glm::Vec3, scale:f32, octives:u8, persistance:f32, lacunarity:f32)->f32{
    let mut noise_value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;

    for _o in 0..octives{
        let perlin_value = perlin.get([
            (pos[0]/scale * frequency) as f64,
            (pos[1]/scale *frequency) as f64,
            (pos[2]/scale *frequency) as f64
        ]) as f32;

        noise_value += perlin_value*amplitude;
        amplitude *= persistance;
        frequency *= lacunarity;
    }
    noise_value
}

//instead of having each surface cell be a struct, each part of the cells are stored in there own collection, making accessing memory quicker
//this is vital for extra proformance
struct PlanetCells{
    //contains index of every node connected to current nodes
   //connections:Vec<Vec<usize>>,
    height:Vec<f32>,
    temperature:Vec<f32>,
    humidity:Vec<f32>
}
impl PlanetCells{
    fn new(vertices:&Vec<glm::Vec3>)->PlanetCells{
        let perlin = Perlin::new(1);

        //generate heights of planet start
        let height = 
        vertices.iter()
            .map(
                |v|
                octive_noise(perlin, &v, 2.5, 7, 0.6, 2.5)
            )
            .collect();

        let humid:Vec<f32> = 
        vertices.iter()
            .map(
                |h|
                octive_noise(perlin, &(h+glm::vec3(0.0,100.0,0.0)), 2.25, 5, 0.55, 2.5)
            )
            .collect();

        PlanetCells{
            height,
            temperature: vec![0.5;vertices.len()],
            humidity: humid
        }
    }
    //puts cell data into format usable for buffer
    fn get_cell_data(&self)->Vec<CellData>{
        //data in form usable for buffer
        let mut celldata= Vec::with_capacity(self.height.len());
        
        for i in 0..self.height.len(){
            celldata.push(
                CellData{
                    height: self.height[i],
                    humidity: self.humidity[i],
                    temperature: self.temperature[i]
                }
            );
        }
        celldata
    }
}

pub struct Planet{
    texture: glium::texture::SrgbTexture2d,

    mesh: PlanetBuffers,

    cells: PlanetCells,

    axis: glm::Vec3,

    to_sun: glm::Vec3
}
impl Planet{
    pub fn new(display:&glium::Display, texture:glium::texture::SrgbTexture2d, iterations :u8)->Planet{

        //generates base shape
        let base_shape = graphics::Shape::icosahedron()
            .subdivide(iterations)
            .normalize();

        //generates cells
        let cells = PlanetCells::new(&base_shape.vertices);

        //generates mesh vertices from base shape
        let planet_vertices:Vec<graphics::PosNorm> = base_shape.vertices
            .iter()
            .map(|v| graphics::PosNorm{
                position:[v.x,v.y,v.z],
                normal: [v.x,v.y,v.z]
             })
            .collect();

        Planet{
            texture,

            mesh: 
            PlanetBuffers{
                //buffer containing base shape of planet, most likely the sphere
                shape_data: glium::VertexBuffer::new(display,&planet_vertices).unwrap(),
                //buffer containing cell data needed for rendering, dynamic as this will change frequently
                planet_data: glium::VertexBuffer::dynamic(display, &cells.get_cell_data()).unwrap(),
                //indices, define triangles of planet
                indices: glium::IndexBuffer::new(display,glium::index::PrimitiveType::TrianglesList, &base_shape.indices).unwrap(),
            },

            cells: cells,

            axis: glm::vec3(0.0,1.0,1.0).normalize(),

            to_sun: glm::vec3(1.0,0.0,0.0)
        }
    }

    pub fn update(&mut self, days: f32){
        //calc tempteretures 
        //latitude that gets maximum sunlight from the sun
        let sun_max = glm::dot(&self.to_sun, &self.axis);
        self.cells.temperature = self.cells.temperature.iter()
            .map(|t| t-0.001)
            .collect();

        self.to_sun= glm::rotate_y_vec3(&self.to_sun, days);
        

        self.mesh.planet_data.write(&self.cells.get_cell_data());
        //self.mesh.vertices.write();
    }

    pub fn draw(&self, target:&mut glium::Frame, program:&glium::Program, params:&glium::DrawParameters,cam:&graphics::Camera){
        //let translation:[[f32;4];4] =  glm::translation(&glm::vec3(0.0,0.0,0.0)).into();
        let pers:[[f32;4];4] = cam.perspective.into();
        let view:[[f32;4];4] = cam.view.into();
        let uniform = glium::uniform! {
            perspective:pers,
            view: view,
            //world: translation,

            tex: &self.texture,
            to_light: [self.to_sun.x,self.to_sun.y,self.to_sun.z]
        };

        target.draw((&self.mesh.shape_data,&self.mesh.planet_data),&self.mesh.indices,program,&uniform,params).unwrap();
    }
}