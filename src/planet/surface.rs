//external crates
use noise::{NoiseFn, Perlin, Seedable};
use nalgebra_glm as glm;

//internal crates
use crate::graphics::shapes;


//data for each cell on the planet, can be written directly to the planetbuffer
#[derive(Copy, Clone)]
pub struct CellData {
    pub height: f32,
    pub humidity: f32,
    pub temperature: f32
}
glium::implement_vertex!(CellData,height,humidity,temperature);


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

//data for every plate
pub struct Plate{
    axis: glm::Vec3,
    density: f32,
    speed: f32,
}

//data relating to the cell
pub struct Cell{
    //what is contained within the cell
    pub contents: CellData,
    //all other cells the cell is connected too
    pub connections: Vec<usize>,
    //physical position of cell
    pub position: glm::Vec3, 
}

//contanes all data for the surface of the planet
pub struct Surface{
    pub cells: Vec<Cell>,
}
impl Surface{
    pub fn new(shape: &shapes::Shape, seed:u32)->Surface{
        
        //creates cells for surface
        let cells:Vec<Cell> = {
            let perlin = Perlin::new(seed);
            //gets connections
            let connections = shape.get_connections();
            //generates cell data
            let cells:Vec<CellData> = shape.vertices.iter()
            .map(|v
                |
                CellData{
                    height: octive_noise(perlin, &v, 2.5, 7, 0.6, 2.5),
                    humidity: octive_noise(perlin, &(v+glm::vec3(0.0,100.0,0.0)), 2.25, 5, 0.55, 2.5),
                    temperature: 0.5,
                }
            )
            .collect();
            
            cells.into_iter()
                .zip(connections.into_iter())
                .zip(shape.vertices.clone().into_iter())
                .map(|cell|
                    Cell{
                        contents: cell.0.0,
                        connections: cell.0.1,
                        position: cell.1
                    }
                )
                .collect()
        };

        //creates plates for surface
        let plates = vec![
            Plate{
                axis: glm::vec3(0.0,0.0,1.0),
                density: 0.5,
                speed: 1.0,
            }
        ];

        Surface{
            cells,
        }
    }

    pub fn update(&mut self,years:f32){
        for cell in self.cells.iter_mut(){
            let surrounding_avg ={
                cell.connections.iter()
                .map(|conn|
                    self.cells[*conn].contents.height)
                .sum()
            };

            cell.contents.height = surrounding_avg;
        }
    }
}