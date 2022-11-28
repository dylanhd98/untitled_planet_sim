use std::vec;

//external crates
use noise::{NoiseFn, Perlin, Seedable};
use nalgebra_glm as glm;

//internal crates
use crate::graphics::shapes;


//data for each cell on the planet
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
        //for every cell translate pos, compare translation with neighbors pos
        //closest neighbor to translated pos is selected
        //copy selected data into cell

        //translate all pos->get all data at translated point-> copy new data into cells
        let new_cell_data:Vec<f32> = self.cells.iter()
            .map(|c|
                {
                    //get new pos
                    let mut new_pos = glm::rotate_y_vec3(&c.position,0.5);

                    //find interpolated data at pos 
                    let mut distnaces:Vec<(&usize,f32)> = c.connections.iter()
                        //map to iter that contains (neighboring cell, distance to new pos)
                        .map(|c| (c,glm::magnitude(&(glm::normalize(&new_pos)-self.cells[*c].position))))
                        //make into list
                        .collect();
                    //sorts ddistances to cell pos
                    distnaces.sort_by(|a,b| a.1.partial_cmp(&b.1).unwrap());
                    //first two pos are the other two verts of triangle
                    //given all 3 points in triangle, interpolate to find value of new pos

                    //calculate barycentric coords of point
                    let w1:f32 = 0.75;//weight for a to b
                    let w2:f32 = 0.25;//weight for a to c



                    //interpolate height with those coords
                    let atob = glm::lerp_scalar(c.contents.height,self.cells[*distnaces[0].0].contents.height , w1);
                    let abtoc = glm::lerp_scalar(atob,self.cells[*distnaces[1].0].contents.height , w2);
                    abtoc
                }
            )
            .collect();

        //add new cell data to all cells
        for cell in self.cells.iter_mut().zip(new_cell_data.into_iter()){
            cell.0.contents.height += (cell.1-cell.0.contents.height);
        }
    }
}