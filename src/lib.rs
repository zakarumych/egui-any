use std::{fmt::Display, hash::Hash};

use egui::{Id, Response, Ui};
use egui_probe::{EguiProbe, Style};
use hashbrown::HashMap;

/// Top-level descriptio of a value.
#[derive(Clone, Debug, Default, PartialEq, EguiProbe)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Desc {
    /// A boolean value.
    #[default]
    Bool,

    /// An integer value.
    Int { min: Option<i64>, max: Option<i64> },

    /// A floating-point value.
    Float { min: Option<f64>, max: Option<f64> },

    /// A string value.
    String { variants: Option<Vec<String>> },

    /// A list of values.
    List {
        // The description of the values.
        elem_desc: Option<Box<Desc>>,
    },

    /// A map of key-value pairs.
    Map {
        // The description of the values.
        value_desc: Option<Box<Desc>>,
    },
}

impl Desc {
    pub fn default_value(&self) -> Value {
        match *self {
            Desc::Bool => Value::Bool(false),
            Desc::Int { min, .. } => Value::Int(min.unwrap_or(0)),
            Desc::Float { min, .. } => Value::Float(min.unwrap_or(0.0)),
            Desc::String { ref variants } => variants.as_ref().and_then(|v| v.first()).map_or_else(
                || Value::String(String::new()),
                |s| Value::String(s.clone()),
            ),
            Desc::List { .. } => Value::List(Vec::new()),
            Desc::Map { .. } => Value::Map(HashMap::new()),
        }
    }
}

impl Desc {
    pub fn kind(&self) -> &str {
        match self {
            Desc::Bool => "bool",
            Desc::Int { .. } => "int",
            Desc::Float { .. } => "float",
            Desc::String { .. } => "string",
            Desc::List { .. } => "list",
            Desc::Map { .. } => "map",
        }
    }
}

/// Top-level value.
#[derive(Clone)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    List(Vec<Value>),
    Map(HashMap<String, Value>),
}

impl Value {
    pub fn kind(&self) -> &str {
        match self {
            Value::Bool(_) => "bool",
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::List(_) => "list",
            Value::Map(_) => "map",
        }
    }
}

pub struct ValueProbe<'a> {
    desc: Option<&'a Desc>,
    mydesc: Desc,
    myid: Id,
    value: &'a mut Value,
    id_source: Id,
}

impl<'a> ValueProbe<'a> {
    pub fn new(desc: Option<&'a Desc>, value: &'a mut Value, id_source: impl Hash) -> Self {
        ValueProbe {
            desc,
            mydesc: Desc::Bool,
            myid: Id::NULL,
            value,
            id_source: Id::new(id_source),
        }
    }
}

impl EguiProbe for ValueProbe<'_> {
    fn probe(&mut self, ui: &mut Ui, style: &Style) -> Response {
        match self.desc {
            None => {
                let id = ui.make_persistent_id(self.id_source);
                self.mydesc = ui
                    .ctx()
                    .data(|d| d.get_temp::<Desc>(id))
                    .unwrap_or_default();
                let r = self.mydesc.probe(ui, style);
                ui.ctx()
                    .data_mut(|d| d.insert_temp(id, self.mydesc.clone()));
                r
            }
            Some(Desc::Bool) => match self.value {
                Value::Bool(value) => value.probe(ui, style),
                _ => {
                    ui.horizontal(|ui| {
                        ui.strong(format!(
                            "Expected boolean, but is {} instead",
                            self.value.kind()
                        ));
                        if ui.small_button("Reset to false").clicked() {
                            *self.value = Value::Bool(false);
                        }
                        ui.strong("?");
                    })
                    .response
                }
            },
            Some(&Desc::Int { min, max }) => {
                let reset_to = match (min, max) {
                    (None, None) => 0,
                    (Some(min), None) => min.max(0),
                    (None, Some(max)) => max.min(0),
                    (Some(min), Some(max)) if min <= max => 0i64.clamp(min, max),
                    (Some(min), Some(max)) => {
                        return invalid_range(ui, min, max);
                    }
                };

                match self.value {
                    Value::Int(value) => match (min, max) {
                        (None, None) => value.probe(ui, style),
                        (Some(min), None) => {
                            egui_probe::customize::probe_range(min.., value).probe(ui, style)
                        }
                        (None, Some(max)) => {
                            egui_probe::customize::probe_range(..=max, value).probe(ui, style)
                        }
                        (Some(min), Some(max)) => {
                            egui_probe::customize::probe_range(min..=max, value).probe(ui, style)
                        }
                    },
                    Value::Float(value) => {
                        let f = *value as i64;
                        let x = match (min, max) {
                            (None, None) => f,
                            (Some(min), None) => min.max(f),
                            (None, Some(max)) => max.min(f),
                            (Some(min), Some(max)) => f.clamp(min, max),
                        };

                        ui.horizontal(|ui| {
                            ui.strong(format!(
                                "Expected integer, but is {} instead",
                                self.value.kind()
                            ));

                            if ui.small_button(format!("Convert to {x}")).clicked() {
                                *self.value = Value::Int(x);
                            }

                            ui.strong("?");
                        })
                        .response
                    }
                    _ => {
                        ui.horizontal(|ui| {
                            ui.strong(format!(
                                "Expected integer, but is {} instead",
                                self.value.kind()
                            ));
                            if ui.small_button(format!("Reset to {reset_to}")).clicked() {
                                *self.value = Value::Int(reset_to);
                            }
                            ui.strong("?");
                        })
                        .response
                    }
                }
            }
            Some(&Desc::Float { min, max }) => {
                let reset_to = match (min, max) {
                    (None, None) => 0.0,
                    (Some(min), None) => min.max(0.0),
                    (None, Some(max)) => max.min(0.0),
                    (Some(min), Some(max)) if min <= max => 0f64.clamp(min, max),
                    (Some(min), Some(max)) => {
                        return invalid_range(ui, min, max);
                    }
                };

                match self.value {
                    Value::Float(value) => match (min, max) {
                        (None, None) => value.probe(ui, style),
                        (Some(min), None) => {
                            egui_probe::customize::probe_range(min.., value).probe(ui, style)
                        }
                        (None, Some(max)) => {
                            egui_probe::customize::probe_range(..=max, value).probe(ui, style)
                        }
                        (Some(min), Some(max)) => {
                            egui_probe::customize::probe_range(min..=max, value).probe(ui, style)
                        }
                    },
                    Value::Int(value) => {
                        let f = *value as f64;
                        let x = match (min, max) {
                            (None, None) => f,
                            (Some(min), None) => min.max(f),
                            (None, Some(max)) => max.min(f),
                            (Some(min), Some(max)) => f.clamp(min, max),
                        };

                        ui.horizontal(|ui| {
                            ui.strong(format!(
                                "Expected integer, but is {} instead",
                                self.value.kind()
                            ));

                            if ui.small_button(format!("Convert to {x:0.1}")).clicked() {
                                *self.value = Value::Float(x);
                            }

                            ui.strong("?");
                        })
                        .response
                    }
                    _ => {
                        ui.horizontal(|ui| {
                            ui.strong(format!(
                                "Expected integer, but is {} instead",
                                self.value.kind()
                            ));
                            if ui.small_button(format!("Reset to {reset_to}")).clicked() {
                                *self.value = Value::Float(reset_to);
                            }
                            ui.strong("?");
                        })
                        .response
                    }
                }
            }
            Some(&Desc::String { ref variants }) => match self.value {
                Value::String(value) => match variants {
                    None => value.probe(ui, style),
                    Some(variants) => {
                        let cbox =
                            egui::ComboBox::from_id_salt(self.id_source).selected_text(&**value);

                        cbox.show_ui(ui, |ui| {
                            for variant in variants.iter() {
                                if ui.selectable_label(value == variant, variant).clicked() {
                                    *value = variant.clone();
                                }
                            }
                        })
                        .response
                    }
                },
                Value::Bool(value) if variants.is_none() => {
                    let (r, s) = convert_to_string(ui, value, "bool");
                    if let Some(s) = s {
                        *self.value = Value::String(s);
                    }
                    r
                }
                Value::Int(value) if variants.is_none() => {
                    let (r, s) = convert_to_string(ui, value, "int");
                    if let Some(s) = s {
                        *self.value = Value::String(s);
                    }
                    r
                }
                Value::Float(value) if variants.is_none() => {
                    let (r, s) = convert_to_string(ui, value, "float");
                    if let Some(s) = s {
                        *self.value = Value::String(s);
                    }
                    r
                }
                _ if variants.is_none() => {
                    ui.horizontal(|ui| {
                        ui.strong(format!(
                            "Expected string, but is {} instead",
                            self.value.kind()
                        ));
                        if ui.small_button("Reset to empty string").clicked() {
                            *self.value = Value::String(String::new());
                        }
                        ui.strong("?");
                    })
                    .response
                }
                _ => {
                    ui.horizontal(|ui| {
                        ui.strong(format!(
                            "Expected string, but is {} instead",
                            self.value.kind()
                        ));
                        if ui.small_button("Reset to default value").clicked() {
                            *self.value = Value::String(
                                variants
                                    .as_ref()
                                    .unwrap()
                                    .first()
                                    .map_or(String::new(), |s| s.clone()),
                            );
                        }
                        ui.strong("?");
                    })
                    .response
                }
            },
            Some(&Desc::List {
                elem_desc: ref elem,
            }) => match self.value {
                Value::List(elems) => match elem {
                    None => {
                        self.myid = ui.make_persistent_id(self.id_source.with("List"));
                        self.mydesc = ui
                            .ctx()
                            .data(|d| d.get_temp::<Desc>(self.myid))
                            .unwrap_or_default();

                        let r = ui
                            .horizontal(|ui| {
                                self.mydesc.probe(ui, style);

                                let r = ui.small_button(style.add_button_text());
                                if r.clicked() {
                                    elems.push(self.mydesc.default_value());
                                }
                            })
                            .response;

                        ui.ctx()
                            .data_mut(|d| d.insert_temp(self.myid, self.mydesc.clone()));
                        r
                    }
                    Some(elem) => {
                        ui.horizontal(|ui| {
                            ui.weak(elem.kind());

                            let r = ui.small_button(style.add_button_text());
                            if r.clicked() {
                                elems.push(elem.default_value());
                            }
                        })
                        .response
                    }
                },
                _ => {
                    ui.horizontal(|ui| {
                        ui.strong(format!(
                            "Expected list, but is {} instead",
                            self.value.kind()
                        ));
                        if ui.small_button("Reset to empty list").clicked() {
                            *self.value = Value::List(Vec::new());
                        }
                        ui.strong("?");
                    })
                    .response
                }
            },
            Some(&Desc::Map {
                value_desc: ref value,
            }) => match self.value {
                Value::Map(values) => {
                    #[derive(Clone)]
                    struct NewKey(String);

                    self.myid = ui.make_persistent_id(self.id_source.with("Map"));

                    let mut new_key = ui
                        .ctx()
                        .data(|d| d.get_temp::<NewKey>(self.myid))
                        .unwrap_or(NewKey(String::new()));

                    let r = match value {
                        None => {
                            self.mydesc = ui
                                .ctx()
                                .data(|d| d.get_temp::<Desc>(self.myid))
                                .unwrap_or_default();

                            let r = ui
                                .horizontal(|ui| {
                                    self.mydesc.probe(ui, style);

                                    ui.text_edit_singleline(&mut new_key.0);

                                    let r = ui.small_button(style.add_button_text());
                                    if r.clicked() {
                                        values.insert(
                                            std::mem::take(&mut new_key.0),
                                            self.mydesc.default_value(),
                                        );
                                    }
                                })
                                .response;

                            ui.ctx()
                                .data_mut(|d| d.insert_temp(self.myid, self.mydesc.clone()));
                            r
                        }
                        Some(elem) => {
                            ui.horizontal(|ui| {
                                ui.weak(elem.kind());

                                ui.text_edit_singleline(&mut new_key.0);

                                let r = ui.small_button(style.add_button_text());
                                if r.clicked() {
                                    values.insert(
                                        std::mem::take(&mut new_key.0),
                                        elem.default_value(),
                                    );
                                }
                            })
                            .response
                        }
                    };

                    ui.ctx().data_mut(|d| d.insert_temp(self.myid, new_key));
                    r
                }
                _ => {
                    ui.horizontal(|ui| {
                        ui.strong(format!(
                            "Expected list, but is {} instead",
                            self.value.kind()
                        ));
                        if ui.small_button("Reset to empty map").clicked() {
                            *self.value = Value::Map(HashMap::new());
                        }
                        ui.strong("?");
                    })
                    .response
                }
            },
        }
    }

    fn iterate_inner(&mut self, ui: &mut Ui, f: &mut dyn FnMut(&str, &mut Ui, &mut dyn EguiProbe)) {
        match self.desc {
            None => {
                let mut probe = ValueProbe::new(Some(&self.mydesc), self.value, self.id_source);
                f("value", ui, &mut probe);
            }
            Some(Desc::Bool) => {}
            Some(Desc::Int { .. }) => {}
            Some(Desc::Float { .. }) => {}
            Some(Desc::String { .. }) => {}
            Some(Desc::List { elem_desc: elem }) => {
                let elem = match elem {
                    None => {
                        self.mydesc.iterate_inner(ui, f);
                        ui.ctx()
                            .data_mut(|d| d.insert_temp(self.myid, self.mydesc.clone()));

                        &self.mydesc
                    }
                    Some(elem) => &**elem,
                };

                match self.value {
                    Value::List(elems) => {
                        let id = self.id_source.with("List");

                        let mut idx = 0;
                        elems.retain_mut(|value| {
                            let mut probe = ValueProbe::new(Some(elem), value, id.with(idx));
                            let mut item = DeleteMe {
                                value: &mut probe,
                                delete: false,
                            };
                            f(&format!("[{idx}]"), ui, &mut item);
                            idx += 1;
                            !item.delete
                        });
                    }
                    _ => {}
                }
            }
            Some(Desc::Map { value_desc: value }) => {
                let desc = match value {
                    None => {
                        self.mydesc.iterate_inner(ui, f);
                        ui.ctx()
                            .data_mut(|d| d.insert_temp(self.myid, self.mydesc.clone()));

                        &self.mydesc
                    }
                    Some(value) => &**value,
                };

                match self.value {
                    Value::Map(values) => {
                        let id: Id = self.id_source.with("List");

                        let mut idx = 0;
                        values.retain(|key, value| {
                            let mut probe = ValueProbe::new(Some(desc), value, id.with(idx));
                            let mut item = DeleteMe {
                                value: &mut probe,
                                delete: false,
                            };
                            f(key, ui, &mut item);
                            idx += 1;
                            !item.delete
                        });
                    }
                    _ => {}
                }
            }
        }
    }
}

fn invalid_range<T: Display>(ui: &mut Ui, min: T, max: T) -> Response {
    ui.strong(format!(
        "Invalid range. `min = {}` must be not greater than `max = {}`.",
        min, max
    ))
}

fn convert_to_string<T: ToString>(
    ui: &mut Ui,
    value: &T,
    kind: &str,
) -> (Response, Option<String>) {
    let mut convert = false;
    let s = value.to_string();

    let r = ui
        .horizontal(|ui| {
            ui.strong(format!("Expected string, but is {} instead", kind));
            if ui.small_button(format!("Convert to {s:?}")).clicked() {
                convert = true;
            }
            ui.strong("?");
        })
        .response;

    (r, if convert { Some(s) } else { None })
}

/// Modifier to add a delete button to an item probe UI.
pub struct DeleteMe<'a, T> {
    pub value: &'a mut T,
    pub delete: bool,
}

impl<T> EguiProbe for DeleteMe<'_, T>
where
    T: EguiProbe,
{
    fn probe(&mut self, ui: &mut egui::Ui, style: &Style) -> egui::Response {
        ui.horizontal(|ui| {
            self.value.probe(ui, style);
            ui.add_space(ui.spacing().item_spacing.x);
            if ui.small_button(style.remove_button_text()).clicked() {
                self.delete = true;
            };
        })
        .response
    }

    fn iterate_inner(&mut self, ui: &mut Ui, f: &mut dyn FnMut(&str, &mut Ui, &mut dyn EguiProbe)) {
        if !self.delete {
            self.value.iterate_inner(ui, f);
        }
    }
}
