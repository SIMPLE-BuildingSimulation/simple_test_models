use std::rc::Rc;
use geometry3d::loop3d::Loop3D;
use geometry3d::point3d::Point3D;
use geometry3d::polygon3d::Polygon3D;
// use schedule::constant::ScheduleConstant;

use building_model::boundary::Boundary;
use building_model::building::Building;
use building_model::fenestration::*;
use building_model::space::Space;
use building_model::heating_cooling::{HeatingCoolingKind, HeaterCooler};
use building_model::simulation_state::SimulationState;
use building_model::substance::Substance;
use building_model::material::Material;
use building_model::construction::Construction;
use building_model::surface::Surface;
use building_model::luminaire::Luminaire;

pub struct SingleZoneTestBuildingOptions {
    pub zone_volume: f64,
    pub material_is_massive: Option<bool>, // Explicitly mentioned
    pub surface_area: f64,
    pub window_area: f64,
    pub heating_power: f64,
    pub lighting_power: f64,
    pub infiltration_rate: f64,
}

impl Default for SingleZoneTestBuildingOptions {
    fn default() -> SingleZoneTestBuildingOptions {
        SingleZoneTestBuildingOptions {
            zone_volume: -1., // Will be checked... negative numbers panic
            material_is_massive: None,
            surface_area: -1., // Will be checked... negative numbers panic
            window_area: 0.,
            heating_power: 0.,
            lighting_power: 0.,
            infiltration_rate: 0.,
        }
    }
}

pub fn add_luminaire(building: &mut Building, state: &mut SimulationState, options: &SingleZoneTestBuildingOptions) {
    
    let power = options.lighting_power;
    assert!(power > 0.);
    let mut luminaire = Luminaire::new("the luminaire".to_string());
    luminaire.set_max_power(power);
    luminaire.set_target_space(Rc::clone(&building.spaces[0]));
    building.add_luminaire(luminaire, state);    
}

pub fn add_heater(building: &mut Building, state: &mut SimulationState, options: &SingleZoneTestBuildingOptions) {
    let power = options.heating_power;
    assert!(power > 0.);
    let mut hvac = HeaterCooler::new(
        "some hvac".to_string(),
        HeatingCoolingKind::ElectricHeating
    );
    hvac.set_max_heating_power(power);
    hvac.push_target_space(Rc::clone(&building.spaces[0])).unwrap();

    building.add_hvac(hvac, state);
    
}

/// A single space building with a single surface (optionally) one operable window that has the same construction
/// as the rest of the walls.
///
/// The surface_area includes the window; the window_area is cut down from it.
pub fn get_single_zone_test_building(state: &mut SimulationState, options: &SingleZoneTestBuildingOptions) -> Building {
    let mut building = Building::new("The Building".to_string());

    /*************** */
    /* ADD THE SPACE */
    /*************** */
    let zone_volume = options.zone_volume;
    if zone_volume <= 0.0 {
        panic!("A positive zone_volume parameter is required (f64)");
    }

    let mut space = Space::new("Some space".to_string());
    space.set_volume(zone_volume);
        // .set_importance(Box::new(ScheduleConstant::new(1.0)));
    building.add_space(space);

    

    /******************* */
    /* ADD THE SUBSTANCE */
    /******************* */
    
    let substance : Rc<Substance>;
    let thickness: f64;

    let is_massive = options.material_is_massive.expect("material_is_massive option required (bool)");
    if is_massive {
        // Massive material
        let mut sub = Substance::new("the substance".to_string());
        sub .set_density(1700.)
            .set_specific_heat_capacity(800.)
            .set_thermal_conductivity(0.816);
        substance = building.add_substance(sub);

        thickness = 200. / 1000.;
    } else {
        let mut sub = Substance::new("the substance".to_string());
        sub .set_density(17.5)
            .set_specific_heat_capacity(2400.)
            .set_thermal_conductivity(0.0252);
        substance = building.add_substance(sub);

        thickness = 20. / 1000.;        
    }

    
    /****************** */
    /* ADD THE MATERIAL */
    /****************** */
    let material = Material::new("the material".to_string(), substance, thickness);
    let material = building.add_material(material);
    

    /********************** */
    /* ADD THE CONSTRUCTION */
    /********************** */
    let mut construction = Construction::new("the construction".to_string());
    construction.layers.push(material);
    let construction = building.add_construction(construction);
    

    /****************** */
    /* SURFACE GEOMETRY */
    /****************** */
    // Wall
    let surface_area = options.surface_area;
    if surface_area <= 0.0 {
        panic!("A positive surface_area option is needed (f64)");
    }

    let l = (surface_area / 4.).sqrt();
    let mut the_loop = Loop3D::new();
    the_loop.push(Point3D::new(-l, -l, 0.)).unwrap();
    the_loop.push(Point3D::new(l, -l, 0.)).unwrap();
    the_loop.push(Point3D::new(l, l, 0.)).unwrap();
    the_loop.push(Point3D::new(-l, l, 0.)).unwrap();
    the_loop.close().unwrap();

    let mut p = Polygon3D::new(the_loop).unwrap();

    // Window... if there is any
    let mut window_polygon: Option<Polygon3D> = None;
    if options.window_area > 0.0 {
        if options.window_area >= surface_area {
            panic!("Win_area >= Surface_area")
        }
        let l = (options.window_area / 4.).sqrt();
        let mut the_inner_loop = Loop3D::new();
        the_inner_loop.push(Point3D::new(-l, -l, 0.)).unwrap();
        the_inner_loop.push(Point3D::new(l, -l, 0.)).unwrap();
        the_inner_loop.push(Point3D::new(l, l, 0.)).unwrap();
        the_inner_loop.push(Point3D::new(-l, l, 0.)).unwrap();
        the_inner_loop.close().unwrap();
        p.cut_hole(the_inner_loop.clone()).unwrap();
        window_polygon = Some(Polygon3D::new(the_inner_loop).unwrap());
    }

    /***************** */
    /* ACTUAL SURFACES */
    /***************** */
    // Add surface
    let space_index = 0; // there is only one space
    let mut surface = Surface::new("Surface".to_string(), p, Rc::clone(&construction));
    surface.set_front_boundary(Boundary::Space(space_index));
    building.add_surface(surface);

    

    // Add window.
    if let Some(window_polygon) = window_polygon {
        let mut fenestration = Fenestration::new(
            "window one".to_string(), 
            window_polygon, 
            construction, 
            FenestrationPositions::Binary,
            FenestrationType::Window
        );
        
        fenestration.set_front_boundary(Boundary::Space(space_index));
        building.add_fenestration(fenestration);
    }

    
    /*********************** */
    /* ADD HEATER, IF NEEDED */
    /*********************** */
    if options.heating_power > 0.0 {
        add_heater(&mut building, state, options);
    }

    /*********************** */
    /* ADD INFILTRATION, IF NEEDED */
    /*********************** */
    if options.infiltration_rate > 0.0 {
        unimplemented!()
    }

    /*********************** */
    /* ADD LIGHTS, IF NEEDED */
    /*********************** */
    if options.lighting_power > 0.0 {
        add_luminaire(&mut building, state, options);
    }

    // Return
    building
}
