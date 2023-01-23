use std::{vec, cell};

//external crates
use noise::{NoiseFn, Perlin, Seedable};
use nalgebra_glm as glm;
use rand::{Rng, seq::SliceRandom};

//internal crates
use crate::graphics::shapes;
use super::{GenInfo, SimInfo};

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

//get connections of every cell
pub fn indices_to_connections(indices: &Vec<u32>)->Vec<Vec<usize>>{
    //TODO: FIND MORE EFFICIENT WAY TO DO THIS, IM SURE THERE IS ONE
    //iterate through indices, for every index, store other two in triangle
    let mut connections:Vec::<Vec<usize>> = vec![Vec::with_capacity(6);indices.len()/3];
    
    indices.chunks(3)
        .for_each(|x|//for each triangle
            {
                //adds connections of each vert in triangle
                for i in 0..3{
                    connections[x[i] as usize].push(x[(i+1)%3] as usize);
                }
            }
        );
    connections
}

//gets all edges, like the above but can be used more efficiently i think
pub fn indices_to_edges(indices: &Vec<u32>)->Vec<(usize,usize)>{
    //TODO: FIND MORE EFFICIENT WAY TO DO THIS, IM SURE THERE IS ONE
    //iterate through indices, for every index, store other two in triangle
    let mut edges:Vec::<(usize,usize)> = Vec::with_capacity(indices.len());
    
    indices.chunks(3)
        .for_each(|x|//for each triangle
            {
                //adds connections of each vert in triangle
                for i in 0..3{
                    let edge = (x[i] as usize,x[(i+1)%3] as usize);
                    //since a cell can never connect to itself and out of the two duplicates of an edge only one will ever be ordered, 
                    //checking if ordered before doing anything will ensure the edge is unique
                    if edge.0<edge.1 {
                        edges.push(edge);
                    }
                }
            }
        );
    edges
}

//gets length of an edge
pub fn edge_length(cells: &Vec<Cell>, edge:&(usize,usize))->f32{
    (cells[edge.0].position - 
        cells[edge.1].position)
        .magnitude()
}

//data for each cell on the planet, for rendering
#[derive(Copy, Clone)]
pub struct CellData {
    //position in space of cell
    pub position: [f32;3],
    //height of land in cell, in m
    pub height: f32,
    //absolute humidity, as g/m^3
    pub humidity: f32,
    //temperature, in degrees C
    pub temperature: f32
}
glium::implement_vertex!(CellData,position,height,humidity,temperature);

//data for every plate
pub struct Plate{
    axis: glm::Vec3,
    density: f32,
    speed: f32,//cm per year, avg is 5-15, earth rad = 6,371km,
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
    pub fn new(pos:glm::Vec3)->Cell{
        Cell { 
            contents: CellData { 
                position: pos.into(),
                height: 0.0,
                humidity: 0.0,
                temperature: 0.0
            },
            position:pos,
            plate: None
        }
    }
}

//contains all data for the surface of the planet
pub struct Surface{
    //every cell
    pub cells: Vec<Cell>,
    //triangle data, u32 as thats whats needed for passing to gpu
    pub triangles: Vec<u32>,
    //contains indices of all cells not in use
    pub bank: Vec<usize>,
    //distnace used for cell collisions, absolute closest one can be to aother before one gets destroyed
    pub cell_distance: f32,
    //all tectonic plates on the surface
    pub plates: Vec<Plate>,
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
                        height: octive_noise(perlin, &pos, 2.5, 7, 0.6, 2.5),
                        humidity: (octive_noise(perlin, &(pos+glm::vec3(0.0,100.0,0.0)), 2.25, 5, 0.55, 2.5)+1.0)*0.5,
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
        let cell_distance = (cells[edges[0].0].position - cells[edges[0].1].position).magnitude()*0.6;

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
                    density: rng.gen_range(0.0..10.0),
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
            bank: Vec::with_capacity(shape.vertices.len()/2),
            cell_distance,
            plates,
        }
    }

    //handles tempereture updating
    pub fn temperature(&mut self,years:f32,sim_info: &SimInfo){
        //latitude that gets maximum sunlight from the sun
        let sun_max = glm::dot(&sim_info.to_sun, &sim_info.axis);

        //updates temp for each
        for cell in self.cells.iter_mut(){
            //adds energy from sun correctly according to latitude
            cell.contents.temperature += (1.0-cell.contents.height)* 
            glm::max2_scalar(1.0-f32::abs(sun_max- glm::dot(&cell.position,&sim_info.axis)), 0.0);

            //cell then loses a percentage to space
            cell.contents.temperature*=sim_info.greenhouse_effect;
        }
    }

    //handles the teconics on the planets surface
    pub fn tectonics(&mut self,years:f32){
        //for every cell with plate info, move according to plate
        for cell in self.cells.iter_mut().filter(|c|c.plate.is_some()){
            //plate cell belongs too
            let plate = &self.plates[cell.plate.unwrap()];
            //translate according to plate
            cell.position= glm::rotate_vec3(&cell.position, plate.speed*years, &plate.axis);
            //put cell pos into cell data
            cell.contents.position=cell.position.into();
        }
        
        //the behaviours, divergent, convergent, transform
        //diverge -> new cell at lengthened tris, center
        //converge -> densest of two close cells destroyed, other get higher besed
        //transform -> search connections of too far cell for closer one, closer replaces old in connections
        
        //there is a threshhold for connection length
        //for each connection in cell, if connection too long, search that second cells connections
        //and select any within threshhold for use as new connection

        let edges = indices_to_edges(&self.triangles);

        //filter edges to get only ones on plate boundries, then test for the collisions
        let plate_boundries:Vec<&(usize,usize)> = edges.iter()
            .filter(|e| 
                &self.cells[e.0].plate != &self.cells[e.1].plate)
                .collect();

        for edge in plate_boundries{
            //self.cells[edge.0].contents.height =5.0;
            //self.cells[edge.1].contents.height =5.0;

            /* 
            let edge_length = edge_length(&self.cells, edge);
            //if cells collide
            if edge_length < self.cell_distance{
                //when two collide remove the denser one, as it subducts
                if self.plates[self.cells[edge.0].plate.unwrap()].density<self.plates[self.cells[edge.1].plate.unwrap()].density{
                    //if edge.0 is less dense, edge.1 is destroyed and subducts
                    self.remove_cell(edge.1,edge.0);
                    self.cells[edge.0].contents.height +=1.0
                }else{
                    //otherwise inverse happens
                    self.remove_cell(edge.0,edge.1);
                    self.cells[edge.1].contents.height +=1.0
                }
            //if cells split too far, spawn new one at midpoint
            }else if edge_length > self.cell_distance*2.0{
                //when two collide remove the denser one, as it subducts
                if self.plates[self.cells[edge.0].plate.unwrap()].density<self.plates[self.cells[edge.1].plate.unwrap()].density{
                    //if edge.0 is less dense, edge.1 is destroyed and subducts
                    self.remove_cell(edge.1,edge.0);
                    self.cells[edge.0].contents.height +=1.0
                }else{
                    //otherwise inverse happens
                    self.remove_cell(edge.0,edge.1);
                    self.cells[edge.1].contents.height +=1.0
                }
            }*/
        }
    }

    //remove cell
    pub fn remove_cell(&mut self,cell: usize,replacement: usize){
        //copies the triangles that contain the cell to be removed, and doesnt contain the cell it is to be replaced with
        let tri_cells:Vec<u32> = self.triangles.chunks(3)
            .filter(|chunk| chunk.contains(&(cell as u32))&& !chunk.contains(&(replacement as u32)))//get only the triangles which do not contain the target cells
            .flatten()
            .map(|n|*n)
            .collect();

        //filter out triangles that contain cell
        self.triangles = self.triangles.chunks(3)
            .filter(|chunk| !chunk.contains(&(cell as u32)))//get only the triangles which do not contain the target cell
            .flatten()
            .map(|n|*n)
            .collect();
            
        //then marks cell as unused by pushing to the cell bank    
        self.bank.push(cell);
    }

    //adds a new cell to the planet between two other cells
    pub fn add_cell(&mut self, parents:(u32,u32)){
        //gets index of new cell to be used if avaliable from bank
        let cell = match self.bank.pop(){
            Some(c) => c,
            None => return, //if no avaliable cells in bank, does nothing
        };

        //removes any triangle containing both parent cells, as these are the ones which will obstruct the new cell
        self.triangles = self.triangles.chunks(3)
            .filter(|chunk| !(chunk.contains(&parents.0)&&chunk.contains(&parents.1)))//get only the triangles which do not contain the target cells
            .flatten()
            .map(|n|*n)
            .collect();

        //gets other two cells which are needed for triangulation
        

        //get midpoint between the two parent cells
        let mid = (self.cells[parents.0 as usize].position+self.cells[parents.1 as usize].position)*0.5;

        //use cell from bank as new cell between the parent cells
        self.cells[cell as usize] = Cell::new(glm::normalize(&mid));
    }
}