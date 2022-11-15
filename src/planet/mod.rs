//external crates
use glium::{Surface,glutin};
use nalgebra_glm as glm;
use noise::{NoiseFn, Perlin, Seedable};

//other internal modules
use crate::graphics;

//child modules


//handles perlin noise for generating base
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

//buffer containing all thigs needed for rendering
struct PlanetBuffers{
    shape_data :glium::VertexBuffer<graphics::VertexPos>,
    planet_data: glium::VertexBuffer<PlanetCell>,
    indices: glium::IndexBuffer<u32>,
}

//data for each cell on the planet, can be written directly to the planetbuffer, although only the neccisary parts are
#[derive(Copy, Clone)]
pub struct PlanetCell {
    pub latitude:f32,
    pub height: f32,
    pub humidity: f32,
    pub temperature: f32
}
glium::implement_vertex!(PlanetCell,height,humidity,temperature);

//data for every plate
pub struct Plate{
    axis: glm::Vec3,
    density: f32,
    speed: f32,
}


pub struct Planet{
    texture: glium::texture::SrgbTexture2d,

    buffers: PlanetBuffers,

    cells: Vec<PlanetCell>,

    axis: glm::Vec3,

    to_sun: glm::Vec3
}
impl Planet{
    pub fn new(display:&glium::Display, texture:glium::texture::SrgbTexture2d, iterations :u8)->Planet{
        let perlin = Perlin::new(1);

        let axis = glm::vec3(0.0,1.0,0.25).normalize();
        //generates base shape
        let base_shape = graphics::shapes::Shape::icosahedron()
            .subdivide(iterations)
            .normalize();

        //generates cells
        let mut cells:Vec<PlanetCell> = base_shape.vertices.iter()
            .map(|v|
                PlanetCell{
                    latitude: glm::dot(v,&axis),
                    height: octive_noise(perlin, &v, 2.5, 7, 0.6, 2.5),
                    humidity: octive_noise(perlin, &(v+glm::vec3(0.0,100.0,0.0)), 2.25, 5, 0.55, 2.5),
                    temperature: 0.5,
                }
            )
            .collect();


        //generates mesh vertices from base shape
        let planet_vertices:Vec<graphics::VertexPos> = base_shape.vertices
            .iter()
            .map(|v| graphics::VertexPos{
                position:[v.x,v.y,v.z],
             })
            .collect();

        Planet{
            texture,

            buffers: 
            PlanetBuffers{
                //buffer containing base shape of planet, most likely the sphere
                shape_data: glium::VertexBuffer::new(display,&planet_vertices).unwrap(),
                //buffer containing cell data needed for rendering, dynamic as this will change frequently
                planet_data: glium::VertexBuffer::dynamic(display, &cells).unwrap(),
                //indices, define triangles of planet
                indices: glium::IndexBuffer::new(display,glium::index::PrimitiveType::TrianglesList, &base_shape.indices).unwrap(),
            },

            cells: cells,

            axis: axis,

            to_sun: glm::vec3(1.0,0.0,0.0)
        }
    }

    pub fn update(&mut self, days: f32){
        //calc tempteretures 
        //latitude that gets maximum sunlight from the sun
        let sun_max = glm::dot(&self.to_sun, &self.axis);
        self.cells.iter_mut()
            .for_each(|c| 
                c.temperature = (1.0-c.height)* 
                glm::max2_scalar(1.0-f32::abs(sun_max-c.latitude), 0.0));

        //one year is 360 days here for simplicity, therefore number of days is converted to radians
        self.to_sun= glm::rotate_y_vec3(&self.to_sun, days*(std::f32::consts::PI/180.0));
        

        self.buffers.planet_data.write(&self.cells);
    }

    pub fn draw(&self, target:&mut glium::Frame, program:&glium::Program, params:&glium::DrawParameters,cam:&graphics::camera::Camera){
        //let translation:[[f32;4];4] =  glm::translation(&glm::vec3(0.0,0.0,0.0)).into();
        let pers:[[f32;4];4] = cam.perspective.into();
        let view:[[f32;4];4] = cam.view.into();
        let uniform = glium::uniform! {
            perspective:pers,
            view: view,
            //world: translation,

            tex: glium::uniforms::Sampler::new(&self.texture).wrap_function(glium::uniforms::SamplerWrapFunction::Clamp),
            to_light: [self.to_sun.x,self.to_sun.y,self.to_sun.z]
        };

        target.draw((&self.buffers.shape_data,&self.buffers.planet_data),&self.buffers.indices,program,&uniform,params).unwrap();
    }
}