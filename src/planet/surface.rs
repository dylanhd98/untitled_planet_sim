
//external crates
use std::{vec,collections::HashMap};
use noise::{NoiseFn, Perlin, Seedable};
use nalgebra_glm as glm;
use rand::{Rng, seq::SliceRandom, rngs::ThreadRng};

//internal crates
use crate::graphics::shapes;
use super::{GenInfo, SimInfo,utils::*};


//data for each cell on the planet, for rendering
#[derive(Copy, Clone)]
pub struct CellData {
    //position in space of cell, should be normalized
    pub position: [f32;3],
    //height of land in cell from sea level, in km, should be in range -10km to 10-km
    pub height: f32,
    //absolute humidity, as g/m^3, range of 0 to 100
    pub humidity: f32,
    //temperature, in degrees C, range of -50 to 50
    pub temperature: f32
}
glium::implement_vertex!(CellData,position,height,humidity,temperature);

//data for every plate
pub struct Plate{
    //axis around which the plate rotates
    axis: glm::Vec3,
    //density of plates determines which will overlap another and nature of collision
    density: f32,
    //cm per year, avg is 5-15, earth rad = 6,371km,
    speed: f32,
}

//data relating to the cell
pub struct Cell{
    //what is contained within the cell
    pub contents: CellData,
    //physical position of cell
    pub position: glm::Vec3,
    //plate that the cell belongs too
    pub plate: Option<usize>
}
impl Cell{
    //creates effectivly blank cell at pos
    pub fn new(pos:glm::Vec3,plate: Option<usize>)->Cell{
        Cell { 
            contents: CellData { 
                position: pos.into(),
                height: -10.0,
                humidity: 0.0,
                temperature: 0.0
            },
            position:pos,
            plate: plate
        }
    }
}

//contains all data for the surface of the planet
pub struct Surface{
    //every cell
    pub cells: Vec<Cell>,
    //triangle data, u32 as thats whats needed for passing to gpu
    pub triangles: Vec<u32>,
    //all tectonic plates on the surface
    pub plates: Vec<Plate>,
    //contains indices of all cells not in use
    pub bank: Vec<usize>,
    //distace used for cell collisions, absolute closest one can be to aother before one gets destroyed
    cell_distance: f32,
    //time passed since last triangulation
    since_triangulation:f32,
    //random generator for surface
    rng: ThreadRng,
}
impl Surface{
    pub fn new(shape: shapes::Shape,gen: &GenInfo)->Surface{
        let mut rng = rand::thread_rng();
        //creates cells for surface
        let mut cells:Vec<Cell> = {
            let perlin = Perlin::new(gen.seed);
            //generates cells with perlin noise
            shape.vertices.clone().into_iter()
            .map(|pos|
                Cell{
                    contents: CellData{
                        position: pos.into(),
                        //multiplied by 10 to get hight in the -10km to 10km range
                        height: octive_noise(perlin, &pos, 2.5, 7, 0.6, 2.5)*10.0,
                        //humidity to be in range 0 to 100, so (perlin+1)*50
                        humidity: (octive_noise(perlin, &(pos+glm::vec3(0.0,100.0,0.0)), 2.25, 5, 0.55, 2.5)+1.0)*50.0,
                        //temp set to zero bc its raised almost immedietely in the sim
                        temperature: 0.0,
                    },
                    position: pos,
                    plate: None
                }
            )
            .collect()
        };

        let edges = indices_to_edges(&shape.indices);

        //length of edge
        let cell_distance = (cells[edges[0].0].position - cells[edges[0].1].position).magnitude();

        //creates randomized plates for surface
        let mut plates:Vec<Plate> = (0..gen.plate_no)
            .map(|_|{
                //randomized axis the plate moves around
                let rand_axis = {
                    let x:f32 = rng.gen_range(0.0..=glm::two_pi());
                    let y:f32 = rng.gen_range(0.0..=glm::two_pi());
                    glm::rotate_y_vec3(
                        &glm::rotate_x_vec3(&glm::vec3(0.0,1.0,0.0),x), 
                        y)
                };

                Plate {
                    axis: rand_axis,
                    density: rng.gen_range(0.0..10.0),//not representative of real value, just used to compare plates
                    speed: rng.gen_range(0.00..0.2)/6371000.0, //done in meters per second, 6371000 is earths radius
                }
            })
            .collect();

        if !plates.is_empty(){
            //place seed cells randomly for each plate for each to spread out from
            for plate in 0..plates.len(){
                let target = rng.gen_range(0..cells.len());
                cells[target].plate = Some(plate);
            }

            //fill planet with the cells via random fill, if there are plates to even fill with
            while cells.iter().any(|c| !c.plate.is_some()){
                //get all plate boundries
                let plate_boundries:Vec<&(usize,usize)> = edges.iter()
                    .filter(|e| 
                    &cells[e.0].plate != &cells[e.1].plate)
                    .collect();

                for _ in 0..16{
                    //extend plate across random boundries 16 times
                    let edge = plate_boundries.choose(&mut rng).unwrap();
                    if cells[edge.0].plate == None{
                        cells[edge.0].plate = cells[edge.1].plate;
                    }else if cells[edge.1].plate == None{
                        cells[edge.1].plate = cells[edge.0].plate;
                    }
                }
            }
        }
        
        Surface{
            cells,
            triangles: shape.indices,
            plates,
            bank: Vec::with_capacity(shape.vertices.len()/2),
            cell_distance,
            since_triangulation:0.0,
            rng,
        }
    }

    //handles tempereture updating
    pub fn temperature(&mut self,years:f32,sim_info: &SimInfo){
        //latitude that gets maximum sunlight from the sun
        let sun_max = glm::dot(&sim_info.to_sun, &sim_info.axis);

        //updates temp for each
        for cell in self.cells.iter_mut(){
            //amount of light recieved as percentage compared to ideal
            //calculates latitude and gets its distance from the ideal/max 
            let light_angle_multiplier = glm::max2_scalar(1.0-f32::abs(sun_max- glm::dot(&cell.position,&sim_info.axis)), 0.0);
            //multiplies ideal temp by angle, then takes lapse rate*height away if above sea level
            cell.contents.temperature = ((40.0*light_angle_multiplier)-(glm::max2_scalar(cell.contents.height,0.0)*sim_info.lapse_rate))*sim_info.greenhouse_effect;
        }
    }

    //handles the teconics on the planets surface
    pub fn tectonics(&mut self,years:f32,sim_info: &mut SimInfo){
        //for every cell with plate info, move according to plate, but also try to stay as far as possible from neighbors of same plate
        for cell in self.cells.iter_mut().filter(|c|c.plate.is_some()){
            //plate cell belongs too
            let plate = &self.plates[cell.plate.unwrap()];
            //translate according to plate
            cell.position= glm::rotate_vec3(&cell.position, plate.speed*years, &plate.axis);
            //put cell pos into cell data
            cell.contents.position=cell.position.into();
        }

        //update counter, check if exceeds interval
        if sim_info.triangulation_interval > self.since_triangulation{
            self.since_triangulation +=years;
            return;
        }
        self.since_triangulation = 0.0;
        
        //get edges from triangles
        let edges = indices_to_edges(&self.triangles);

        //filter edges to get only ones on plate boundries, then test for the collisions
        let plate_boundries:Vec<&(usize,usize)> = edges.iter()
            .filter(|e| 
                &self.cells[e.0].plate != &self.cells[e.1].plate)
                .collect();

        //for each edge along the plate boundry
        for edge in plate_boundries{
            //get edge length
            let edge_length = edge_length(&self.cells, edge);
            //if cells collide
            if edge_length < self.cell_distance{
                //when two collide remove the denser one, as it subducts
                //sort by density
                let dense_sorted =
                if self.plates[self.cells[edge.0].plate.unwrap()].density<self.plates[self.cells[edge.1].plate.unwrap()].density{
                    (edge.0,edge.1)
                }else{
                    (edge.1,edge.0)
                };

                self.remove_cell(dense_sorted.0);
                self.cells[dense_sorted.1].contents.height +=1.0
            }
            //if cells split too far, spawn new one at midpoint
            else if edge_length > self.cell_distance*1.25{
                self.add_cell(*edge);
            }
        }

        //retriangulate mesh
        //project points stereographic
        let mut all_points:Vec<glm::Vec3> = self.cells.iter()
            .map(|cell| stereographic(cell.position))
            .collect();
        //get indices of all points excluding those in bank
        let to_triangulate:Vec<u32> = (0..all_points.len()).into_iter()
            .filter(|x| !self.bank.contains(x))
            .map(|x| x as u32)
            .collect();

        self.triangles = bowyer_watson(&mut all_points, &to_triangulate);
    }

    //remove cell
    pub fn remove_cell(&mut self,cell: usize){
        //hashset to ensure surrounding points are unique
/*         let mut surrounding_points:HashSet<u32> = HashSet::with_capacity(6);

        //filter out triangles that contain cell, record all other points if they do
        self.triangles = self.triangles.chunks(3)
            .filter(|chunk| {
                //get only the triangles which do not contain the target cell
                if !chunk.contains(&(cell as u32)){
                    true
                }else{
                    //remove cell from triangles
                    let mut tri:Vec<u32> = chunk.iter()
                        .filter(|x| **x != cell as u32)//make sure ther original cell is excluded
                        .map(|x| {
                            surrounding_points.insert(*x);//insert points into hashset
                            *x
                        }) 
                        .collect();
                    false
                }
            })
            .flatten()
            .map(|n|*n)
            .collect();

        let mut all_points:Vec<glm::Vec3> = self.cells.iter()
            .map(|cell| stereographic(cell.position))
            .collect();

        //gets new triangulation 
        let mut triangulation = bowyer_watson(&mut all_points,&mut Vec::from_iter(surrounding_points));
        //adds new triangles
        self.triangles.append(&mut triangulation);*/
            
        //then marks cell as unused by pushing to the cell bank    
        self.bank.push(cell);
    }

    //adds a new cell to the planet between two other cells
    pub fn add_cell(&mut self, parents:(usize,usize)){
        //gets index of new cell to be used if avaliable from bank
        let cell = match self.bank.pop(){
            Some(c) => c,
            None => return, //if no avaliable cells in bank, does nothing
        };

        /* 
        //vec to store surrounding triangles
        let mut tris = Vec::with_capacity(6);

        //removes any triangle containing both parent cells, as these are the ones which will obstruct the new cell, they are also stored for later
        self.triangles = self.triangles.chunks(3)
            .filter(|chunk| {
                //if triangle contains both parents, appends triangle
                if chunk.contains(&(parents.0 as u32))&&chunk.contains(&(parents.1 as u32)){
                    tris.append(&mut vec![chunk[0],chunk[1],chunk[2]]);
                    false
                }else{
                    true
                }
            })
            .flatten()
            .map(|n|*n)
            .collect();*/

        //get midpoint between the two parent cells
        let mid = (self.cells[parents.0 as usize].position+self.cells[parents.1 as usize].position)*0.5;
        //select random plate of the two to make the new plate belong to
        let plate = if self.rng.gen(){
            self.cells[parents.0 as usize].plate
        }else{
            self.cells[parents.1 as usize].plate
        };
        //use cell from bank as new cell between the parent cells
        self.cells[cell as usize] = Cell::new(glm::normalize(&mid),plate);
    }
}