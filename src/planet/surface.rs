use std::vec;

//external crates
use noise::{NoiseFn, Perlin, Seedable};
use nalgebra_glm as glm;

//internal crates
use crate::graphics::shapes;

//data for each cell on the planet
#[derive(Copy, Clone)]
pub struct CellData {
    pub position: [f32;3],
    pub height: f32,
    pub humidity: f32,
    pub temperature: f32
}
glium::implement_vertex!(CellData,position,height,humidity,temperature);


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
    pub plates: Vec<Plate>
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
            .map(|v|
                CellData{
                    position: [v.x,v.y,v.z],
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
                        position: cell.1,
                    }
                )
                .collect()
        };

        //creates plates for surface
        let plates = vec![
            Plate{
                axis: glm::vec3(0.0,0.0,1.0),
                density: 0.5,
                speed: 0.005,
            },
            Plate{
                axis: glm::vec3(0.0,0.0,1.0),
                density: 0.5,
                speed: -0.005,
            }
        ];

        Surface{
            cells,
            plates
        }
    }

    pub fn tectonics(&mut self,years:f32){
        for cell in self.cells.iter_mut(){
            //translate according to plate
            cell.position= 
            if cell.position.x>0.0{
                glm::rotate_y_vec3(&cell.position,years)
            }else{
                glm::rotate_y_vec3(&cell.position,-years)
            };
            //put cell pos into cell data
            cell.contents.position=[cell.position.x,cell.position.y,cell.position.z];
        }
        
        //the behaviours, divergent, convergent, transform
        //diverge -> new cell at lengthened tris, center
        //converge -> densest of two close cells destroyed, other get higher besed
        //transform -> search connections of too far cell for closer one, closer replaces old in connections
        
        //there is a threshhold for connection length
        //for each connection in cell, if connection too long, search that second cells connections
        //and select any within threshhold for use as new connection
        

    }

    pub fn update(&mut self,years:f32){
        self.tectonics(years);
        //for every cell translate pos, compare translation with neighbors pos
        //closest neighbor to translated pos is selected
        //copy selected data into cell
        /* 
        //translate all pos->get all data at translated point-> copy new data into cells
        let new_cell_data:Vec<f32> = self.cells.iter()
            .map(|a|
                {
                    //get new pos
                    let new_pos = if a.position.y>0.0{
                        glm::rotate_y_vec3(&a.position,0.01)
                    }else{
                        glm::rotate_y_vec3(&a.position,0.01)
                    };

                    //find distance of connection at new pos
                    let mut distances:Vec<(&usize,f32)> = a.connections.iter()
                        //map to iter that contains (neighboring cell, distance to new pos)
                        .map(|con| (con,glm::magnitude(&(glm::normalize(&new_pos)-self.cells[*con].position))))
                        //make into list
                        .collect();

                    //sorts distances to cell pos
                    distances.sort_by(|a,b| a.1.partial_cmp(&b.1).unwrap());

                    //first two in distances are the other two verts of triangle
                    let b = &self.cells[*distances[0].0];
                    let c = &self.cells[*distances[1].0];

                    //calculating barycentric coords of point
                    
                    //vectors for calculating areas
                    let atob = b.position-a.position;
                    let atoc = c.position-a.position;
                    let atonew = new_pos-a.position;
                    //area of main triangle
                    let tri_area = glm::cross(&atob, &atoc).magnitude()*0.5;
                    let area_opposite_b = glm::cross(&atonew, &atob).magnitude()*0.5;
                    let area_opposite_c = glm::cross(&atonew, &atoc).magnitude()*0.5;

                    let beta:f32 = area_opposite_b/tri_area;//weight for b
                    let gamma:f32 = area_opposite_c/tri_area;//weight for  c
                    let alpha:f32 = 1.0-beta-gamma;//weight for a

                    //interpolate height with those coords
                    b.contents.height*beta + c.contents.height*gamma + a.contents.height*alpha
                }
            )
            .collect();

        //add new cell data to all cells
        for (cell,new_height) in self.cells.iter_mut().zip(new_cell_data.into_iter()){
            cell.contents.height += (new_height-cell.contents.height)*(years/10.0);
            //cell.contents.height = new_height;
        }*/
    }
}