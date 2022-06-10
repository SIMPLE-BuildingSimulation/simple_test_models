/*
MIT License
Copyright (c) 2021 Germán Molina
Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:
The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.
THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/


/// The kind of Floating point number used in the
/// library... the `"float"` feature means it becomes `f32`
/// and `f64` is used otherwise.
#[cfg(feature = "float")]
type Float = f32;

#[cfg(not(feature = "float"))]
type Float = f64;

use geometry3d::{Loop3D, Point3D, Polygon3D};
use std::rc::Rc;

use simple_model::{
    hvac::ElectricHeater, substance::Normal as NormalSubstance, Boundary, Construction,
    Fenestration, FenestrationPositions, FenestrationType, Infiltration, Luminaire, Material,
    SimpleModel, SimulationStateHeader, Space, Surface,
};

/// The test material
pub enum TestMat {
    /// A Concrete with a certain `Float` thickness   
    /// 
    /// # Properties
    /// * density: 1700.
    /// * Specific heat: 800.
    /// * Thermal Cond.: 0.816
    /// * Emmisivity: From `options.emmisivity`
    Concrete(Float), 

    /// A Polyurethane with a certain `Float` thickness
    /// 
    /// # Properties
    /// * density: 17.5
    /// * Specific heat: 2400.
    /// * Thermal Cond.: 0.0252
    /// * Emmisivity: From `options.emmisivity
    Polyurethane(Float)
}

pub struct SingleZoneTestBuildingOptions {
    pub zone_volume: Float,
    pub construction: Vec<TestMat>, // Explicitly mentioned
    pub surface_area: Float,
    pub window_area: Float,
    pub heating_power: Float,
    pub lighting_power: Float,
    pub infiltration_rate: Float,
    pub emmisivity: Float,
}

impl Default for SingleZoneTestBuildingOptions {
    fn default() -> SingleZoneTestBuildingOptions {
        SingleZoneTestBuildingOptions {
            zone_volume: -1., // Will be checked... negative numbers panic
            construction: Vec::with_capacity(0),
            surface_area: -1., // Will be checked... negative numbers panic
            window_area: 0.,
            heating_power: 0.,
            lighting_power: 0.,
            infiltration_rate: 0.,
            emmisivity: 0.84,
        }
    }
}

pub fn add_luminaire(
    model: &mut SimpleModel,
    options: &SingleZoneTestBuildingOptions,
    header: &mut SimulationStateHeader,
) {
    let power = options.lighting_power;
    assert!(power > 0.);
    let mut luminaire = Luminaire::new("the luminaire".to_string());
    luminaire.set_max_power(power);
    luminaire.set_target_space(Rc::clone(&model.spaces[0]));
    model.add_luminaire(luminaire, header);
}

pub fn add_heater(
    model: &mut SimpleModel,
    options: &SingleZoneTestBuildingOptions,
    header: &mut SimulationStateHeader,
) {
    let power = options.heating_power;
    assert!(power > 0.);
    let mut hvac = ElectricHeater::new("some hvac".to_string());
    hvac.set_target_space(Rc::clone(&model.spaces[0]));
    model.add_hvac(hvac.wrap(), header);
}

/// A single space model with a single surface (optionally) one operable window that has the same construction
/// as the rest of the walls. Thw front of the surface faces South.
///
/// The surface_area includes the window; the window_area is cut down from it.
pub fn get_single_zone_test_building(
    options: &SingleZoneTestBuildingOptions,
) -> (SimpleModel, SimulationStateHeader) {
    let mut model = SimpleModel::new("The SimpleModel".to_string());
    let mut header = SimulationStateHeader::new();

    /*************** */
    /* ADD THE SPACE */
    /*************** */
    let zone_volume = options.zone_volume;
    if zone_volume <= 0.0 {
        panic!("A positive zone_volume parameter is required (Float)");
    }

    let mut space = Space::new("Some space".to_string());
    space.set_volume(zone_volume);

    /*********************** */
    /* ADD INFILTRATION, IF NEEDED */
    /*********************** */
    if options.infiltration_rate > 0.0 {
        let infiltration_rate = options.infiltration_rate;
        assert!(infiltration_rate > 0.);
        let infiltration = Infiltration::Constant(infiltration_rate);
        space.set_infiltration(infiltration);
    }

    // .set_importance(Box::new(ScheduleConstant::new(1.0)));
    let space = model.add_space(space);

    /******************* */
    /* ADD THE SUBSTANCE */
    /******************* */

    // Add both substances
    let mut concrete = NormalSubstance::new("concrete".to_string());
    concrete.set_density(1700.)
            .set_specific_heat_capacity(800.)
            .set_thermal_conductivity(0.816)
            .set_thermal_absorbtance(options.emmisivity);
    let concrete = model.add_substance(concrete.wrap());

    let mut polyurethane = NormalSubstance::new("polyurethane".to_string());
    polyurethane.set_density(17.5)
        .set_specific_heat_capacity(2400.)
        .set_thermal_conductivity(0.0252)
        .set_thermal_absorbtance(options.emmisivity);
    let polyurethane = model.add_substance(polyurethane.wrap());


    /*********************************** */
    /* ADD THE MATERIAL AND CONSTRUCTION */
    /*********************************** */
    let mut construction = Construction::new("the construction".to_string());
    for (i,c) in options.construction.iter().enumerate(){
        let material = match c{
            TestMat::Concrete(thickness)=>{
                Material::new(format!("Material {}", i), concrete.clone(), *thickness)
            }
            TestMat::Polyurethane(thickness)=>{
                Material::new(format!("Material {}", i), polyurethane.clone(), *thickness)
            }
        };
        let material = model.add_material(material);
        construction.materials.push(material);
    }
    let construction = model.add_construction(construction);
    
    /****************** */
    /* SURFACE GEOMETRY */
    /****************** */
    // Wall
    let surface_area = options.surface_area;
    if surface_area <= 0.0 {
        panic!("A positive surface_area option is needed (Float)");
    }

    let l = (surface_area / 4.).sqrt();
    let mut the_loop = Loop3D::new();
    the_loop.push(Point3D::new(-l, 0., 0.)).unwrap();
    the_loop.push(Point3D::new(l, 0., 0.)).unwrap();
    the_loop.push(Point3D::new(l, 0., l * 2.)).unwrap();
    the_loop.push(Point3D::new(-l, 0., l * 2.)).unwrap();
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
        the_inner_loop.push(Point3D::new(-l, 0., l / 2.)).unwrap();
        the_inner_loop.push(Point3D::new(l, 0., l / 2.)).unwrap();
        the_inner_loop
            .push(Point3D::new(l, 0., 3. * l / 2.))
            .unwrap();
        the_inner_loop
            .push(Point3D::new(-l, 0., 3. * l / 2.))
            .unwrap();
        the_inner_loop.close().unwrap();
        p.cut_hole(the_inner_loop.clone()).unwrap();
        window_polygon = Some(Polygon3D::new(the_inner_loop).unwrap());
    }

    /***************** */
    /* ACTUAL SURFACES */
    /***************** */
    // Add surface
    let mut surface = Surface::new("Surface".to_string(), p, Rc::clone(&construction));
    surface.set_front_boundary(Boundary::Space(Rc::clone(&space)));
    model.add_surface(surface);

    // Add window.
    if let Some(window_polygon) = window_polygon {
        let mut fenestration = Fenestration::new(
            "window one".to_string(),
            window_polygon,
            construction,
            FenestrationPositions::Binary,
            FenestrationType::Window,
        );

        fenestration.set_front_boundary(Boundary::Space(Rc::clone(&space)));
        model.add_fenestration(fenestration, &mut header);
    }

    /*********************** */
    /* ADD HEATER, IF NEEDED */
    /*********************** */
    if options.heating_power > 0.0 {
        add_heater(&mut model, options, &mut header);
    }

    /*********************** */
    /* ADD LIGHTS, IF NEEDED */
    /*********************** */
    if options.lighting_power > 0.0 {
        add_luminaire(&mut model, options, &mut header);
    }

    // Return
    (model, header)
}

#[cfg(test)]
mod testing {

    use super::*;

    #[test]
    fn test_with_window() {
        let surface_area = 4.;
        let window_area = 1.;
        let zone_volume = 40.;

        let (_simple_model, _state_header) = get_single_zone_test_building(
            // &mut state,
            &SingleZoneTestBuildingOptions {
                zone_volume,
                surface_area,
                window_area,
                construction: vec![TestMat::Concrete(0.2)],
                ..Default::default()
            },
        );
    }
}
