use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::fs::File;
use std::collections::HashSet;

use tomson::Toml;
use handlebars::Handlebars;
use rustc_serialize::json::{self, Json};

fn generate_code(mut toml: String) -> String {
    // turn the toml string into json for compatibility with handlebars
    let mut json = Toml::as_json(&mut toml).unwrap();

    let mut void_fields = HashSet::new();

    for (id, field) in json.as_object_mut().unwrap().get_mut("field").unwrap().as_object_mut().unwrap().iter_mut() {
        let field_obj = field.as_object_mut().unwrap();
        let field_type = field_obj.get("type").unwrap().as_string().unwrap().to_string();

        let field_name = id.clone();

        if field_type == "void" {
            void_fields.insert(field_name);
            continue;
        }

        let component_name = if let Some(component_json) = field_obj.get("component") {
            component_json.as_string().unwrap().to_string()
        } else {
            id.clone()
        };

        let component_has_type = if let Some(component_has_type_json) = field_obj.get("component_has_type") {
            component_has_type_json.as_boolean().unwrap()
        } else {
            false
        };

        field_obj.insert("struct_field_name".to_string(), Json::String(field_name.clone()));
        field_obj.insert("getter_name".to_string(), Json::String(id.clone()));
        field_obj.insert("component_name".to_string(), Json::String(component_name.clone()));
        field_obj.insert(field_type.clone(), Json::Boolean(true));

        if component_has_type {
            field_obj.insert("component_has_type".to_string(), Json::Boolean(true));
        }

        match field_type.as_ref() {
            "sum_f64" => {
                field_obj.insert("struct_field_type".to_string(), Json::String("f64".to_string()));
                field_obj.insert("struct_field_cons".to_string(), Json::String("0.0".to_string()));
                field_obj.insert("getter_type".to_string(), Json::String("f64".to_string()));
                field_obj.insert("getter_expr".to_string(), Json::String(format!("self.{}", field_name)));
                field_obj.insert("remove_entity".to_string(), Json::String(format!("if let Some(v) = entity.copy_{}() {{ cell.{} -= v; }}",
                                                                                   component_name, field_name)));
                field_obj.insert("insert_entity".to_string(), Json::String(format!("if let Some(v) = entity.copy_{}() {{ cell.{} += v; }}",
                                                                                   component_name, field_name)));
            }
            "count_bool" => {
                field_obj.insert("struct_field_type".to_string(), Json::String("usize".to_string()));
                field_obj.insert("struct_field_cons".to_string(), Json::String("0".to_string()));
                field_obj.insert("getter_type".to_string(), Json::String("bool".to_string()));
                field_obj.insert("getter_expr".to_string(), Json::String(format!("self.{} != 0", field_name)));
                field_obj.insert("remove_entity".to_string(), Json::String(format!("if entity.contains_{}() {{ cell.{} -= 1; }}",
                                                                                   component_name, field_name)));
                field_obj.insert("insert_entity".to_string(), Json::String(format!("if entity.contains_{}() {{ cell.{} += 1; }}",
                                                                                   component_name, field_name)));
            }
            "any" => {
                field_obj.insert("struct_field_type".to_string(), Json::String("AnySet<EntityId>".to_string()));
                field_obj.insert("struct_field_cons".to_string(), Json::String("AnySet::new()".to_string()));
                field_obj.insert("getter_type".to_string(), Json::String("bool".to_string()));
                field_obj.insert("getter_expr".to_string(), Json::String(format!("!self.{}.is_empty()", field_name)));
                field_obj.insert("any_name".to_string(), Json::String(format!("any_{}", field_name)));
                field_obj.insert("any_type".to_string(), Json::String("Option<EntityId>".to_string()));
                field_obj.insert("any_expr".to_string(), Json::String(format!("self.{}.any()", field_name)));
                field_obj.insert("remove_entity".to_string(), Json::String(format!("if entity.contains_{}() {{ cell.{}.remove(entity.id()); }}",
                                                                                   component_name, field_name)));
                field_obj.insert("insert_entity".to_string(), Json::String(format!("if entity.contains_{}() {{ cell.{}.insert(entity.id()); }}",
                                                                                   component_name, field_name)));
                field_obj.insert("is_set_type".to_string(), Json::Boolean(true));
            }
            "set" => {
                field_obj.insert("struct_field_type".to_string(), Json::String("EntitySet".to_string()));
                field_obj.insert("struct_field_cons".to_string(), Json::String("EntitySet::new()".to_string()));
                field_obj.insert("getter_type".to_string(), Json::String("bool".to_string()));
                field_obj.insert("getter_expr".to_string(), Json::String(format!("!self.{}.is_empty()", field_name)));
                field_obj.insert("iter_name".to_string(), Json::String(format!("iter_{}", field_name)));
                field_obj.insert("iter_type".to_string(), Json::String("EntitySetIter".to_string()));
                field_obj.insert("iter_expr".to_string(), Json::String(format!("self.{}.iter()", field_name)));
                field_obj.insert("remove_entity".to_string(), Json::String(format!("if entity.contains_{}() {{ cell.{}.remove(entity.id()); }}",
                                                                                   component_name, field_name)));
                field_obj.insert("insert_entity".to_string(), Json::String(format!("if entity.contains_{}() {{ cell.{}.insert(entity.id()); }}",
                                                                                   component_name, field_name)));
                field_obj.insert("is_set_type".to_string(), Json::Boolean(true));
            }
            other => panic!("unknown field type {}", other),
        }
    }

    {
        let mut json_obj = json.as_object_mut().unwrap();
        json_obj.insert("void".to_string(), Json::Object(json::Object::new()));
        for field in void_fields.iter() {
            let void_json = json_obj.get_mut("field").unwrap().as_object_mut().unwrap().remove(field).unwrap();
            let void_obj = void_json.into_object().unwrap();
            json_obj.get_mut("void").unwrap().as_object_mut().unwrap().insert(field.to_string(), Json::Object(void_obj));
        }
    }

    let mut handlebars = Handlebars::new();

    // prevent xml escaping
    handlebars.register_escape_fn(|input| input.to_string());
    handlebars.template_render(TEMPLATE, &json).unwrap()

}

fn read_file_to_string<P: AsRef<Path>>(path: P) -> String {
    let mut file = File::open(path).unwrap();
    let mut string = String::new();
    file.read_to_string(&mut string).unwrap();

    string
}

pub fn generate_spatial_hash<P: AsRef<Path>, Q: AsRef<Path>>(in_path: P, out_path: Q) {
    let string = read_file_to_string(in_path);

    let output_string = generate_code(string);

    let mut outfile = File::create(out_path).unwrap();
    write!(outfile, "{}", output_string).unwrap();
}

const TEMPLATE: &'static str = r#"// Automatically generated. Do not edit.
#![allow(unused_imports)]
use ecs::*;
use coord::Coord;
use grid::{Grid, StaticGrid, DefaultGrid, IterGrid, CoordIterGrid};
use util::AnySet;

pub type SpatialHashCellIter<'a> = <StaticGrid<SpatialHashCell> as IterGrid<'a>>::Iter;
pub type SpatialHashCoordIter = <StaticGrid<SpatialHashCell> as CoordIterGrid>::CoordIter;

#[derive(Serialize, Deserialize)]
pub struct SpatialHashCell {

{{#each field}}
    {{ struct_field_name }}: {{ struct_field_type }},
{{/each}}

    // set of entities currently in this cell
    entities: EntitySet,

    // action on which this cell was last updated
    last_updated: u64,
}

impl SpatialHashCell {
    pub fn new() -> Self {
        SpatialHashCell {
{{#each field}}
            {{ struct_field_name }}: {{ struct_field_cons }},
{{/each}}

            entities: EntitySet::new(),
            last_updated: 0,
        }
    }

    pub fn last_updated(&self) -> u64 {
        self.last_updated
    }

    pub fn entity_ids(&self) -> &EntitySet {
        &self.entities
    }

    pub fn entity_id_iter(&self) -> EntitySetIter {
        self.entities.iter()
    }

{{#each field}}
    pub fn {{ getter_name }}(&self) -> {{ getter_type }} {
        {{ getter_expr }}
    }
    {{#if iter_name}}
    pub fn {{ iter_name }}(&self) -> {{ iter_type }} {
        {{ iter_expr }}
    }
    {{/if}}
    {{#if any_name}}
    pub fn {{ any_name }}(&self) -> {{ any_type }} {
        {{ any_expr }}
    }
    {{/if}}
{{/each}}
}

impl Default for SpatialHashCell {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize)]
pub struct SpatialHashTable {
    grid: StaticGrid<SpatialHashCell>,
    empty: SpatialHashCell,
}

impl SpatialHashTable {
    pub fn new(width: usize, height: usize) -> Self {
        SpatialHashTable {
            grid: StaticGrid::new_default(width, height),
            empty: SpatialHashCell::new(),
        }
    }

    pub fn clear(&mut self) {
        self.grid.reset_all();
    }

    pub fn cell_iter(&self) -> SpatialHashCellIter {
        self.grid.iter()
    }

    pub fn coord_iter(&self) -> SpatialHashCoordIter {
        self.grid.coord_iter()
    }

    pub fn get(&self, coord: Coord) -> &SpatialHashCell {
        self.grid.get(coord).unwrap_or(&self.empty)
    }

    pub fn limits_min(&self) -> Coord {
        self.grid.limits_min()
    }

    pub fn limits_max(&self) -> Coord {
        self.grid.limits_max()
    }

    pub fn is_valid_coord(&self, coord: Coord) -> bool {
        self.grid.is_valid_coord(coord)
    }

    pub fn width(&self) -> usize {
        self.grid.width()
    }

    pub fn height(&self) -> usize {
        self.grid.height()
    }

    fn get_mut(&mut self, coord: Coord) -> &mut SpatialHashCell {
        unsafe { self.grid.get_unchecked_mut(coord) }
    }

    fn change_entity_position(&mut self, entity: EntityRef, current_position: Coord, new_position: Coord, action_id: u64) {
        self.remove_entity_position(entity, current_position, action_id);
        self.add_entity_position(entity, new_position, action_id);
    }

    fn remove_entity_position(&mut self, entity: EntityRef, position: Coord, action_id: u64) {
        let mut cell = self.get_mut(position);

{{#each field}}
        {{ remove_entity }}
{{/each}}

        cell.entities.remove(entity.id());
        cell.last_updated = action_id;
    }

    fn add_entity_position(&mut self, entity: EntityRef, position: Coord, action_id: u64) {
        let mut cell = self.get_mut(position);

{{#each field}}
        {{ insert_entity }}
{{/each}}

        cell.entities.insert(entity.id());
        cell.last_updated = action_id;
    }

    pub fn update(&mut self, ecs: &EcsCtx, action: &EcsAction, action_id: u64) {

        for (entity_id, new_position) in action.positive_copy_iter_position(ecs) {
            let entity = ecs.entity(entity_id);
            // Add and remove tracked components based on the current data stored about the
            // entity, ignoring any component changes in the current action. These will be
            // applied later.
            if let Some(current_position) = entity.copy_position() {
                // the entity is changing position
                self.change_entity_position(entity, current_position, new_position, action_id);
            } else {
                // the entity is gaining a position
                self.add_entity_position(entity, new_position, action_id);
            }
        }

        for entity_id in action.negative_iter_position(ecs) {
            let entity = ecs.entity(entity_id);
            if let Some(position) = entity.copy_position() {
                self.remove_entity_position(entity, position, action_id);
            }
        }

{{#each field}}
        self.update_{{ struct_field_name }}(ecs, action, action_id);
{{/each}}
{{#each void}}
        self.update_{{ @key }}(ecs, action, action_id);
{{/each}}
    }

{{#each field}}
    fn update_{{ struct_field_name }}(&mut self, ecs: &EcsCtx, action: &EcsAction, action_id: u64) {
    {{#if count_bool}}
        for entity_id in action.positive_iter_{{ component_name }}(ecs) {
            let entity = ecs.post_entity(action, entity_id);
            if let Some(position) = entity.copy_position() {
                if !entity.current_contains_{{ component_name }}() {
                    let cell = self.get_mut(position);
                    cell.{{ struct_field_name }} += 1;
                    cell.last_updated = action_id;
                }
            }
        }

        for entity_id in action.negative_iter_{{ component_name }}(ecs) {
            let entity = ecs.post_entity(action, entity_id);
            if let Some(position) = entity.copy_position() {
                if entity.current_contains_{{ component_name }}() {
                    let cell = self.get_mut(position);
                    cell.{{ struct_field_name }} -= 1;
                    cell.last_updated = action_id;
                }
            }
        }
    {{/if}}
    {{#if sum_f64}}
        for (entity_id, new) in action.positive_iter_{{ component_name }}(ecs) {
            let entity = ecs.post_entity(action, entity_id);
            if let Some(position) = entity.copy_position() {
                let current = entity.current_copy_{{ component_name }}().unwrap_or(0.0);
                let increase = new - current;
                let cell = self.get_mut(position);
                cell.{{ struct_field_name }} += increase;
                cell.last_updated = action_id;
            }
        }
        for entity_id in action.negative_iter_{{ component_name }}(ecs) {
            let entity = ecs.post_entity(action, entity_id);
            if let Some(position) = entity.copy_position() {
                if let Some(value) = entity.current_copy_{{ component_name }}() {
                    let cell = self.get_mut(position);
                    cell.{{ struct_field_name }} -= value;
                    cell.last_updated = action_id;
                }
            }
        }
    {{/if}}
    {{#if is_set_type}}
        {{#if component_has_type}}
        for (entity_id, _) in action.positive_iter_{{ component_name }}(ecs) {
        {{else}}
        for entity_id in action.positive_iter_{{ component_name }}(ecs) {
        {{/if}}
            let entity = ecs.post_entity(action, entity_id);
            if let Some(position) = entity.copy_position() {
                let cell = self.get_mut(position);
        {{#if component_has_type}}
                if entity.current_{{ component_name }}().is_none() {
        {{else}}
                if !entity.current_contains_{{ component_name }}() {
        {{/if}}
                    cell.{{ struct_field_name }}.insert(entity_id);
                }
                cell.last_updated = action_id;
            }
        }
        for entity_id in action.negative_iter_{{ component_name }}(ecs) {
            let entity = ecs.post_entity(action, entity_id);
            if let Some(position) = entity.copy_position() {
        {{#if component_has_type}}
                if entity.current_{{ component_name }}().is_some() {
        {{else}}
                if entity.current_contains_{{ component_name }}() {
        {{/if}}
                    let cell = self.get_mut(position);
                    cell.{{ struct_field_name }}.remove(entity_id);
                    cell.last_updated = action_id;
                }
            }
        }
    {{/if}}
    }
{{/each}}
{{#each void}}
    fn update_{{ @key }}(&mut self, ecs: &EcsCtx, action: &EcsAction, action_id: u64) {
    {{#if component_has_type}}
        for (entity_id, _) in action.positive_iter_{{ @key }}(ecs) {
    {{else}}
        for entity_id in action.positive_iter_{{ @key }}(ecs) {
    {{/if}}

            let entity = ecs.post_entity(action, entity_id);
            if let Some(position) = entity.copy_position() {
                self.get_mut(position).last_updated = action_id;
            }
        }

        for entity_id in action.negative_iter_{{ @key }}(ecs) {
            let entity = ecs.post_entity(action, entity_id);
            if let Some(position) = entity.copy_position() {
                self.get_mut(position).last_updated = action_id;
            }
        }
    }
{{/each}}
}
"#;

