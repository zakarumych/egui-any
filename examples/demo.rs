use egui_probe::Probe;
use egui_any::{Desc, Value, ValueProbe};

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "egui-any demo app",
        native_options,
        Box::new(|cc| Box::new(EguiValueDemoApp::new(cc))),
    )
    .unwrap();
}

struct EguiValueDemoApp {
    desc: Option<Desc>,
    value: Value,
}

impl EguiValueDemoApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        EguiValueDemoApp {
            desc: None,
            value: Value::Int(42),
        }
    }
}

impl eframe::App for EguiValueDemoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            egui::widgets::global_dark_light_mode_switch(ui);
        });

        egui::SidePanel::left("desc").show(ctx, |ui| {
            Probe::new("Desc", &mut self.desc).show(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui|{
                let mut value_probe = ValueProbe::new(self.desc.as_ref(), &mut self.value, "demo-value");
                Probe::new("Value", &mut value_probe).show(ui);
            });
        });
    }
}
