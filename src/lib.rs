use geometry3d::polygon3d::Polygon3D;
use geometry3d::point3d::Point3D;
use geometry3d::loop3d::Loop3D;


use building_model::building::Building;
use building_model::substance::{SubstanceProperties};
use building_model::material::{MaterialProperties};
use building_model::boundary::Boundary;
use building_model::fenestration::*;
use building_model::heating_cooling::HeatingCoolingKind;

use simulation_state::simulation_state::SimulationState;

pub fn get_single_zone_building_with_heater(state: &mut SimulationState, power: f64)-> Building {
    let mut building = get_single_zone_building(state);

    //let mut space = building.get_space(0).unwrap();

    //let system = 

    //space.add_heating_cooling(system);
    building.add_heating_cooling_to_space(state,0, HeatingCoolingKind::ElectricHeating).unwrap();
    building.set_space_max_heating_power(0, power).unwrap();
    
    return building
}

/// A single space building with one operable window (it is opaque
/// at the moment, but I don't think it matters) and a 1500W
/// heater. The zone has only one Surface.
pub fn get_single_zone_building(state: &mut SimulationState)-> Building {
    
    let mut building = Building::new("The Building".to_string()); 

    // Add the space
    let zone_volume = 40.;
    let space_index = building.add_space("Some space".to_string(), state);
    building.set_space_volume(space_index,zone_volume).unwrap();

    //building.add_heating_cooling_to_space(state, space_index, HeatingCoolingKind::IdealHeaterCooler).unwrap();
    //building.set_space_max_heating_power(space_index, 1500.).unwrap();

    // Add substance
    let poly_index = building.add_substance("polyurethane".to_string());
    building.set_substance_properties(poly_index, SubstanceProperties{
        thermal_conductivity: 0.0252, // W/m.K            
        specific_heat_capacity: 2400., // J/kg.K
        density: 17.5, // kg/m3... reverse engineered from paper
    }).unwrap();

    // add material
    let mat_index = building.add_material("20mm Poly".to_string());
    building.set_material_properties(mat_index, MaterialProperties{
        thickness: 20./1000.
    }).unwrap();
    building.set_material_substance(mat_index,poly_index).unwrap();

    // Add construction
    let c_index = building.add_construction("The construction".to_string());
    building.add_material_to_construction(c_index, mat_index).unwrap();


    // Create surface geometry
    // Geometry
    let mut the_loop = Loop3D::new();
    let l = 1. as f64;
    the_loop.push( Point3D::new(-l, -l, 0.)).unwrap();
    the_loop.push( Point3D::new(l, -l, 0.)).unwrap();
    the_loop.push( Point3D::new(l, l, 0.)).unwrap();
    the_loop.push( Point3D::new(-l, l, 0.)).unwrap();
    the_loop.close().unwrap();
    
    let mut p = Polygon3D::new(the_loop).unwrap();


    let mut the_inner_loop = Loop3D::new();
    let l = 0.5 as f64;
    the_inner_loop.push( Point3D::new(-l, -l, 0.)).unwrap();
    the_inner_loop.push( Point3D::new(l, -l, 0.)).unwrap();
    the_inner_loop.push( Point3D::new(l, l, 0.)).unwrap();
    the_inner_loop.push( Point3D::new(-l, l, 0.)).unwrap();
    the_inner_loop.close().unwrap();
    p.cut_hole(the_inner_loop.clone()).unwrap();

    

    // Add surface
    let surface_index = building.add_surface("Surface".to_string());
    building.set_surface_construction(surface_index,c_index).unwrap();
    building.set_surface_polygon(surface_index, p).unwrap();
    
    building.set_surface_front_boundary(surface_index, Boundary::Space(space_index)).unwrap();

    // Add window.        
    let window_polygon = Polygon3D::new(the_inner_loop).unwrap();
    let window_index = building.add_fenestration(state, "Window One".to_string(), FenestrationPositions::Binary, FenestrationType::Window);
    building.set_fenestration_construction(window_index, c_index).unwrap();     
    building.set_fenestration_polygon(window_index, window_polygon).unwrap();
    building.set_fenestration_front_boundary(surface_index, Boundary::Space(space_index)).unwrap();

    
    

    // Finished building the Building

    return building;

}