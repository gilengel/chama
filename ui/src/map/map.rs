use geo::intersects::Intersects;
use geo::prelude::{BoundingRect, Contains, EuclideanDistance};
use geo::{Coordinate, Line, LineString, MultiPolygon, Polygon, Rect};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rust_editor::gizmo::{GetPosition, Id};
use rust_editor::interactive_element::{InteractiveElement, InteractiveElementState};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

extern crate geo_booleanop;

use geo_booleanop::boolean::BooleanOp;

use std::cmp::Ordering;
use std::collections::hash_map::Keys;
use std::collections::HashMap;

use super::district::{District, House};
use super::house::generate_houses_from_polygon;
use super::intersection::Intersection;
use super::street::Street;


impl Serialize for Map {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        // 3 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("Map", 3)?;
        state.serialize_field("width", &self.width)?;
        state.serialize_field("height", &self.height)?;
        state.serialize_field("streets", &self.streets.values().cloned().collect::<Vec<Street>>())?;
        state.serialize_field("districts", &self.districts.values().cloned().collect::<Vec<District>>())?;

        state.end()
    }
}

#[derive(Deserialize, Clone)]
pub struct Map {
    width: u32,
    height: u32,

    pub(crate) street_polygon: MultiPolygon<f64>,
    pub(crate) district_polygons: Vec<Polygon<f64>>,

    pub(crate) streets: HashMap<Uuid, Street>,

    #[serde(skip_serializing)]
    pub(crate) intersections: HashMap<Uuid, Intersection>,

    districts: HashMap<Uuid, District>,

    #[serde(skip_serializing)]
    bounding_box: Rect<f64>,
}

impl Default for Map {
    fn default() -> Map {
        Map {
            width: 2560,
            height: 1440,
            streets: HashMap::new(),
            intersections: HashMap::new(),
            districts: HashMap::new(),

            street_polygon: MultiPolygon::new(vec![]),
            district_polygons: vec![],

            bounding_box: Rect::new(Coordinate { x: 0., y: 0. }, Coordinate { x: 0., y: 0. }),
        }
    }
}

impl From<&mut Map> for Polygon<f64> {
    fn from(map: &mut Map) -> Polygon<f64> {
        let v: Vec<Coordinate<f64>> = map
            .intersections()
            .into_iter()
            .map(|x| x.1.position())
            .collect();

        Polygon::new(LineString::from(v), vec![])
    }
}

impl geo::algorithm::concave_hull::ConcaveHull for Map {
    type Scalar = f64;

    fn concave_hull(&self, concavity: Self::Scalar) -> geo::Polygon<Self::Scalar> {
        let v: Vec<Coordinate<Self::Scalar>> = self
            .intersections()
            .into_iter()
            .map(|x| x.1.position())
            .collect();

        let polygon: Polygon<Self::Scalar> = Polygon::new(LineString::from(v), vec![]);

        polygon.concave_hull(concavity)
    }
}

impl Map {
    pub fn new(width: u32, height: u32) -> Self {
        Map {
            width,
            height,
            ..Default::default()
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// Removes all data (streets, districts, intersections) from the instance.
    /// Be aware that calling this is permanent and not unduable.
    pub fn clear(&mut self) {
        self.streets.clear();
        self.intersections.clear();
        self.districts.clear();
    }

    /// Returns `true` if `self` has no streets, no intersections and no districts
    pub fn is_empty(&self) -> bool {
        self.streets.is_empty() && self.intersections.is_empty() && self.districts.is_empty()
    }

    pub fn intersections(&self) -> &HashMap<Uuid, Intersection> {
        &self.intersections
    }

    pub fn intersections_mut(&mut self) -> &mut HashMap<Uuid, Intersection> {
        &mut self.intersections
    }

    pub fn intersections_keys<'a>(&'a self) -> Keys<'a, Uuid, Intersection> {
        self.intersections.keys()
    }

    pub fn districts(&self) -> &HashMap<Uuid, District> {
        &self.districts
    }

    pub fn districts_mut(&mut self) -> &mut HashMap<Uuid, District> {
        &mut self.districts
    }

    pub fn streets(&self) -> &HashMap<Uuid, Street> {
        &self.streets
    }

    pub fn streets_mut(&mut self) -> &mut HashMap<Uuid, Street> {
        &mut self.streets
    }

    pub fn intersections_by_ids<'a>(
        &'a self,
        ids: &'a Vec<Uuid>,
    ) -> impl Iterator<Item = &'a Intersection> {
        self.intersections
            .values()
            .filter(|intersection| ids.contains(&intersection.id()))
    }

    pub fn intersections_within_rectangle<'a>(
        &'a self,
        rect: &'a Rect<f64>,
    ) -> impl Iterator<Item = &'a Intersection> {
        self.intersections
            .values()
            .filter(|intersection| rect.contains(&intersection.position()))
    }

    pub fn intersections_within_rectangle_mut<'a>(
        &'a mut self,
        rect: &'a Rect<f64>,
    ) -> impl Iterator<Item = &'a mut Intersection> {
        self.intersections
            .values_mut()
            .filter(|intersection| rect.contains(&intersection.position()))
    }

    pub fn intersections_with_state<'a>(
        &'a self,
        state: InteractiveElementState,
    ) -> impl Iterator<Item = &'a Intersection> {
        self.intersections
            .values()
            .filter(move |intersection| intersection.state() == state)
    }

    pub fn intersections_with_state_mut<'a>(
        &'a mut self,
        state: InteractiveElementState,
    ) -> impl Iterator<Item = &'a mut Intersection> {
        self.intersections
            .values_mut()
            .filter(move |intersection| intersection.state() == state)
    }

    fn update_districts(&mut self) {
        self.street_polygon = MultiPolygon::new(vec![]);
        for (_, street) in &self.streets {
            self.street_polygon = self.street_polygon.union(street.polygon());
        }

        let mut district_polygons = vec![];
        for polygon in self.street_polygon.iter() {
            district_polygons.append(&mut polygon.interiors().to_vec());
        }

        self.districts_mut().clear();
        for i in 0..district_polygons.len() {
            let polygon = Polygon::new(district_polygons[i].clone(), vec![]);

            let seed = <ChaCha8Rng as SeedableRng>::Seed::default();
            let houses: Vec<House> = generate_houses_from_polygon(&polygon, 50., seed);

            let district = District {
                polygon: polygon.clone(),
                houses,
                minimum_house_side: 250.,
                ..District::default()
            };
            self.districts_mut().insert(district.id(), district);

            self.district_polygons.push(polygon);
        }
    }

    pub fn add_street(&mut self, street: &Street) -> Uuid {
        let id = street.id();
        self.streets.insert(id, street.clone());

        self.update_districts();

        id
    }

    pub fn remove_street(&mut self, street: &Street) {
        self.streets.remove(&street.id());

        self.update_districts();
    }

    pub fn add_district(&mut self, district: District) -> Uuid {
        let id = district.id();
        self.districts.insert(id, district);

        id
    }

    pub fn add_intersection(&mut self, intersection: Intersection) -> Uuid {
        let id = intersection.id();
        self.intersections.insert(id, intersection);

        self.update_bounding_box();

        id
    }

    pub fn get_intersection_at_position(
        &self,
        position: &Coordinate<f64>,
        offset: f64,
        ignored_intersections: &Vec<Uuid>,
    ) -> Option<Uuid> {
        for (id, intersection) in &self.intersections {
            if ignored_intersections.into_iter().any(|e| e == id) {
                continue;
            }

            if intersection.position().euclidean_distance(position) < offset {
                return Some(*id);
            }
        }

        None
    }

    pub fn line_intersection_with_intersections(
        &self,
        line: &Line<f64>,
    ) -> Vec<(Uuid, Coordinate<f64>)> {
        let mut intersections: Vec<(Uuid, Coordinate<f64>)> = Vec::new();

        for (_, intersection) in &self.intersections {
            if line.intersects(&intersection.position())
                && intersection.position() != line.start
                && intersection.position() != line.end
            {
                intersections.push((intersection.id(), intersection.position()));
            }
        }

        intersections.sort_by(|a, b| {
            let d1 = a.1.euclidean_distance(&line.start);
            let d2 = b.1.euclidean_distance(&line.start);

            if d1 < d2 {
                return Ordering::Less;
            }

            if d1 == d2 {
                return Ordering::Equal;
            }

            Ordering::Greater
        });

        intersections
    }

    pub fn get_street_at_position(
        &self,
        position: &Coordinate<f64>,
        ignored_streets: &Vec<Uuid>,
    ) -> Option<Uuid> {
        for (id, street) in &self.streets {
            if ignored_streets.contains(id) {
                continue;
            }

            if street.is_point_on_street(position) {
                return Some(*id);
            }
        }

        None
    }

    pub fn update_bounding_box(&mut self) {
        let polygon: Polygon<f64> = self.into();
        if let Some(bb) = polygon.bounding_rect() {
            self.bounding_box = bb;

            let offset = 20.0;
            self.bounding_box.set_min(
                self.bounding_box.min()
                    - Coordinate {
                        x: offset,
                        y: offset,
                    },
            );
            self.bounding_box.set_max(
                self.bounding_box.max()
                    + Coordinate {
                        x: offset,
                        y: offset,
                    },
            )
        }
    }

    pub fn street(&self, id: &Uuid) -> Option<&Street> {
        if self.streets.contains_key(id) {
            return Some(self.streets.get(id).unwrap());
        }

        None
    }

    pub fn street_mut(&mut self, id: &Uuid) -> Option<&mut Street> {
        if self.streets.contains_key(id) {
            return Some(self.streets.get_mut(id).unwrap());
        }

        None
    }

    pub fn intersection(&self, id: &Uuid) -> Option<&Intersection> {
        if self.intersections.contains_key(id) {
            return Some(self.intersections.get(id).unwrap());
        }

        None
    }

    pub fn intersection_mut(&mut self, id: &Uuid) -> Option<&mut Intersection> {
        if self.intersections.contains_key(id) {
            return Some(self.intersections.get_mut(id).unwrap());
        }

        None
    }

    pub fn district(&self, id: &Uuid) -> Option<&District> {
        if self.districts.contains_key(id) {
            return Some(self.districts.get(id).unwrap());
        }

        None
    }

    pub fn district_mut(&mut self, id: &Uuid) -> Option<&mut District> {
        if self.districts.contains_key(id) {
            return Some(self.districts.get_mut(id).unwrap());
        }

        None
    }

    pub fn get_district_at_position(&self, position: &Coordinate<f64>) -> Option<Uuid> {
        for (_, district) in &self.districts {
            if district.is_point_on_district(position) {
                return Some(district.id());
            }
        }

        None
    }

    pub fn remove_district(&mut self, id: &Uuid) {
        self.districts.remove(id);
    }
}
