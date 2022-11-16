//data for each cell on the planet, can be written directly to the planetbuffer, although only the neccisary parts are
#[derive(Copy, Clone)]
pub struct PlanetCell {
    pub latitude:f32,
    pub height: f32,
    pub humidity: f32,
    pub temperature: f32
}
glium::implement_vertex!(PlanetCell,height,humidity,temperature);