use geometry3d::loop3d::Loop3D;
use geometry3d::point3d::Point3D;
use geometry3d::polygon3d::Polygon3D;
use schedule::constant::ScheduleConstant;

use building_model::boundary::Boundary;
use building_model::building::Building;
use building_model::fenestration::*;
use building_model::heating_cooling::HeatingCoolingKind;
use building_model::material::MaterialProperties;
use building_model::substance::SubstanceProperties;

use simulation_state::simulation_state::SimulationState;

pub struct Options {
    pub zone_volume: f64,
    pub material_is_massive: Option<bool>, // Explicitly mentioned
    pub surface_area: f64,
    pub window_area: f64,
    pub heating_power: f64,
    pub lighting_power: f64,
}

impl Default for Options {
    fn default() -> Options {
        Options {
            zone_volume: -1., // Will be checked... negative numbers panic
            material_is_massive: None,
            surface_area: -1., // Will be checked... negative numbers panic
            window_area: 0.,
            heating_power: 0.,
            lighting_power: 0.,
        }
    }
}

pub fn add_luminaire(building: &mut Building, state: &mut SimulationState, options: &Options) {
    let power = options.lighting_power;
    assert!(power > 0.);
    building.add_luminaire_to_space(state, 0).unwrap();
    building.set_space_max_lighting_power(0, power).unwrap();
}

pub fn add_heater(building: &mut Building, state: &mut SimulationState, options: &Options) {
    let power = options.heating_power;
    assert!(power > 0.);
    //space.add_heating_cooling(system);
    building
        .add_heating_cooling_to_space(state, 0, HeatingCoolingKind::ElectricHeating)
        .unwrap();
    building.set_space_max_heating_power(0, power).unwrap();
}

/// A single space building with a single surface (optionally) one operable window that has the same construction
/// as the rest of the walls.
///
/// The surface_area includes the window; the window_area is cut down from it.
pub fn get_single_zone_test_building(state: &mut SimulationState, options: &Options) -> Building {
    let mut building = Building::new("The Building".to_string());

    /*************** */
    /* ADD THE SPACE */
    /*************** */
    let zone_volume = options.zone_volume;
    if zone_volume <= 0.0 {
        panic!("A positive zone_volume parameter is required (f64)");
    }
    let space_index = building.add_space("Some space".to_string());
    building.set_space_volume(space_index, zone_volume).unwrap();
    building
        .set_space_importance(space_index, Box::new(ScheduleConstant::new(1.0)))
        .unwrap();

    /******************* */
    /* ADD THE SUBSTANCE */
    /******************* */
    let (substance_properties, material_thickness) = if options
        .material_is_massive
        .expect("material_is_massive option required (bool)")
    {
        // Massive material
        (
            SubstanceProperties {
                density: 1700.,               // kg/m3... reverse engineered from paper
                specific_heat_capacity: 800., // J/kg.K
                thermal_conductivity: 0.816,  // W/m.K
            },
            200. / 1000.,
        ) // 200mm
    } else {
        // Lightweight material
        (
            SubstanceProperties {
                thermal_conductivity: 0.0252,  // W/m.K
                specific_heat_capacity: 2400., // J/kg.K
                density: 17.5,                 // kg/m3... reverse engineered from paper
            },
            20. / 1000.,
        ) // 20mm
    };

    let poly_index = building.add_substance("the_substance".to_string());
    building
        .set_substance_properties(poly_index, substance_properties)
        .unwrap();

    /****************** */
    /* ADD THE MATERIAL */
    /****************** */
    let mat_index = building.add_material("The_material".to_string());
    building
        .set_material_properties(
            mat_index,
            MaterialProperties {
                thickness: material_thickness,
            },
        )
        .unwrap();
    building
        .set_material_substance(mat_index, poly_index)
        .unwrap();

    /********************** */
    /* ADD THE CONSTRUCTION */
    /********************** */
    let c_index = building.add_construction("The construction".to_string());
    building
        .add_material_to_construction(c_index, mat_index)
        .unwrap();

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
    let surface_index = building.add_surface("Surface".to_string());
    building
        .set_surface_construction(surface_index, c_index)
        .unwrap();
    building.set_surface_polygon(surface_index, p).unwrap();

    building
        .set_surface_front_boundary(surface_index, Boundary::Space(space_index))
        .unwrap();

    // Add window.
    if let Some(window_polygon) = window_polygon {
        let window_index = building.add_fenestration(
            state,
            "Window One".to_string(),
            FenestrationPositions::Binary,
            FenestrationType::Window,
        );
        building
            .set_fenestration_construction(window_index, c_index)
            .unwrap();
        building
            .set_fenestration_polygon(window_index, window_polygon)
            .unwrap();
        building
            .set_fenestration_front_boundary(surface_index, Boundary::Space(space_index))
            .unwrap();
    }

    if let Ok(win) = building.get_fenestration(0){
        assert!((options.window_area - win.area().unwrap()).abs() < f64::EPSILON);
    }

    if let Ok(surf) = building.get_surface(surface_index) {
        // areas add up?
        assert!( ( options.surface_area - surf.area().unwrap() - options.window_area).abs() < f64::EPSILON );
        match surf.front_boundary() {
            Boundary::Space(s) => {
                assert_eq!(*s, space_index)
            }
            _ => assert!(false),
        }
    } else {
        assert!(false);
    }

    /*********************** */
    /* ADD HEATER, IF NEEDED */
    /*********************** */
    if options.heating_power > 0.0 {
        add_heater(&mut building, state, options);
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
