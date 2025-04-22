use std::cmp::Ordering;
use std::fmt::Debug;
use std::{collections::HashMap,fmt, path::PathBuf};
use std::cell::Cell;
use eframe::egui;
use egui::{Color32, ColorImage, DroppedFile, Vec2, Widget};
use image::{GenericImageView, ImageReader, Pixel, Rgb};
use itertools::Itertools;

mod iris_color;
mod iris_image_creation;

fn main() {
    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native("My egui App", native_options, Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc)))));
}
struct ImageWindow {
    id:usize,
    path:PathBuf,
    name:String,
    open:bool,
    color_list:HashMap<u32,iris_color::AvarageRgb>,
    color_percent:HashMap<u32,f32>,
    color_pixel_count:HashMap<u32,u32>,
    color_gradation:f32,
    color_dist_type:iris_color::ColorSpace,
    color_display_threshhold:f32,
    compare_state:CompareState,
    img: Option<iris_image_creation::HSLRect>,
    img_rect:Option<egui::TextureHandle>,
    img_bar:Option<egui::TextureHandle>,
    img_dispaly_generated:bool,
    reload_hsl_rect:bool,
    reload_hsl_bar:bool,
    clean_up_value:f32,
    mark_every_color:bool,
}

#[derive(Debug,PartialEq)]
enum CompareState {
    Percentages,
    Saturation,
}

thread_local!(static WINDOW_ID: Cell<usize> = Cell::new(0));


impl ImageWindow {
    fn new (new_file:DroppedFile)-> Self{
        let path = new_file.path.clone().unwrap();
        let mut name = path.file_name().unwrap().to_owned().to_string_lossy().to_string();
        if path.file_stem().unwrap().to_str().unwrap().to_string().len() >= 10 {
            name = path.file_stem().unwrap().to_str().unwrap().to_string()[0..10].to_string() + "." + &path.extension().unwrap().to_string_lossy()
        }
        WINDOW_ID.with(|thread_id|{
            let id = thread_id.get();
            thread_id.set(id+1);
            let open = true;
            let color_percent = HashMap::new();
            let color_list = HashMap::new();
            let color_pixel_count = HashMap::new();
            let color_gradation = 50.0;
            let color_dist_type = iris_color::ColorSpace::OkLab;
            let color_display_threshhold = 0.01;
            let compare_state = CompareState::Percentages;
            let clean_up_value = 0.01;
            ImageWindow{
                path,
                name,
                open,
                color_percent,
                color_list,
                color_pixel_count,
                color_gradation,
                id,
                color_dist_type,
                color_display_threshhold,
                compare_state,
                img: None,
                img_bar: None,
                img_rect: None,
                img_dispaly_generated: false,
                reload_hsl_rect:false,
                reload_hsl_bar:false,
                clean_up_value,
                mark_every_color:false,
            }

        })
    }
    fn generate_color_display(&mut self) {
        if self.color_list.len() <= 0 {
            return;
        }
        let mut color_sorted:Vec<_> = self.color_list.iter_mut().collect();
        color_sorted.sort_by(|a,b| {
            if self.color_percent[a.0] < self.color_percent[b.0] {
                return Ordering::Greater;
            }else{
                return Ordering::Less;
            }
        });
        let mut dom_color:Option<f32> = None; 
        for c in &color_sorted {
            if iris_color::HSL::from_rgb(&c.1.to_rgb()).l >= 0.2 {
                dom_color = Some(iris_color::HSL::from_rgb(&c.1.to_rgb()).h);
                break;
            }        
        } 
        if dom_color.is_none() {
            dom_color = Some(iris_color::HSL::from_rgb(&color_sorted[1].1.to_rgb()).h);
        }
        if self.img.is_none() {
            self.img = Some(iris_image_creation::HSLRect::new([256,128],dom_color.unwrap()));
        }
        if let Some(img) = &mut self.img {
            img.obj.clear();
            for (id,c) in color_sorted{
                if self.color_percent[id] >= self.color_display_threshhold {
                    img.add_marker(c,5,2);
                }
            }
            img.generate_h_bar();
            self.reload_hsl_bar = true;
            img.generate_sl_rect();
            self.reload_hsl_rect = true;
        }
        self.img_dispaly_generated = true;
    }
    fn show (&mut self,ctx:&egui::Context){
        if self.open{
            if (self.img_bar.is_none()|| self.reload_hsl_bar) && self.img_dispaly_generated  {
                if let Some(img) = &self.img {
                    self.img_bar = Some(ctx.load_texture("img_bar",ColorImage::from_rgb([img.size[0].try_into().unwrap(),(img.size[1]/4).try_into().unwrap()],&img.img_bar),Default::default()));
                    self.reload_hsl_bar = false;
                }
            }
            if (self.img_rect.is_none()|| self.reload_hsl_rect) && self.img_dispaly_generated{
                if let Some(img) = &self.img {
                    self.img_rect = Some(ctx.load_texture("img_rect",ColorImage::from_rgb([img.size[0].try_into().unwrap(),img.size[1].try_into().unwrap()],&img.img_rect),Default::default()));
                    self.reload_hsl_rect = false;
                }
            }
            let mut window_open = self.open;
            egui::Window::new(self.name.clone()).id(egui::Id::new(self.id)).open(&mut window_open).show(ctx, |ui| {

                let string_path = "file://".to_owned() + self.path.to_str().unwrap();
                ui.add(
                    egui::Image::new(string_path).shrink_to_fit()
                ); 

                egui::ComboBox::from_label("Select Color Space for distance")
                    .selected_text(format!("{:?}", self.color_dist_type))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.color_dist_type, iris_color::ColorSpace::Rgb, "RGB");
                        ui.selectable_value(&mut self.color_dist_type, iris_color::ColorSpace::CieLab, "CieLab");
                        ui.selectable_value(&mut self.color_dist_type, iris_color::ColorSpace::OkLab, "OkLab");
                    }
                );
                let color_deg_max:f32;
                match self.color_dist_type {
                    iris_color::ColorSpace::CieLab => color_deg_max = 300.0,
                    iris_color::ColorSpace::OkLab => color_deg_max = 2.0,
                    iris_color::ColorSpace::Rgb => color_deg_max = 500.0,
                    _=> color_deg_max = 0.0,
                }
                ui.add(egui::Slider::new(&mut self.color_gradation,0.0 ..= color_deg_max).text("Color Gradation"));
                ui.add(egui::Slider::new(&mut self.clean_up_value,0.0 ..= 0.1).text("Color Gradation"))
                    .on_hover_text("Minimum Color distance in OKLab, at which colors get merged after scan. \n (to clean up Duplicate Colors)");
                if ui.add(egui::Button::new("Scan")).clicked(){
                    self.scan_image(ui);
                }
                ui.separator();
                egui::ComboBox::from_label("Sorted by")
                    .selected_text(format!("{:?}", self.compare_state))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.compare_state,CompareState::Percentages , "Percentages");
                        ui.selectable_value(&mut self.compare_state,CompareState::Saturation , "Saturation");
                    }
                );
                if ui.checkbox(&mut self.mark_every_color,"Select every color").clicked(){
                    for (_id,c) in &mut self.color_list{
                       c.marked = self.mark_every_color; 
                    }
                };
                match self.compare_state {
                    CompareState::Percentages => {  // ----------PERCENTAGE GUI
                        let mut color_sorted:Vec<_> = self.color_list.iter_mut().collect();
                        color_sorted.sort_by(|a,b| {
                            if self.color_percent[a.0] < self.color_percent[b.0] {
                                return Ordering::Greater;
                            }else{
                                return Ordering::Less;
                            }
                        });
                        ui.add(egui::Slider::new(&mut self.color_display_threshhold,0.0 ..= 1.0).text("Color Display Threshold"));
                        egui::CollapsingHeader::new("Colors").show(ui,|ui|{
                            egui::ScrollArea::vertical().max_height(100.0).auto_shrink([false,true]).show(ui, |ui| {
                                let aw = ui.available_width();
                                egui::Grid::new("Colors").spacing(Vec2::new(0.0,3.0)).show(ui,|ui|{
                                    let mut column_count = 0;
                                    for (id,c) in color_sorted.iter_mut(){
                                        if self.color_percent[id] >= self.color_display_threshhold{
                                            iris_color::color_display(ui, c);
                                            column_count += 1;
                                            if column_count > (aw/(ui.available_width()+3.0)) as i32 {
                                                ui.end_row();
                                                column_count = 0;
                                            }
                                        }
                                    }
                                });
                            });
                        });
                        egui::CollapsingHeader::new("Color Percentages").show(ui,|ui|{
                            egui::ScrollArea::vertical().max_height(100.0).show(ui, |ui| {
                                ui.with_layout(egui::Layout::top_down(egui::Align::TOP).with_cross_justify(true),|ui|{
                                    for (id,c) in color_sorted.iter_mut(){
                                        if self.color_percent[id] >= self.color_display_threshhold || self.color_display_threshhold <= 0.0{
                                            iris_color::color_display_percent(ui, c,self.color_percent[id].clone());
                                        }
                                    }
                                });
                            });
                        });
                        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT),|ui|{
                            if let Some(rect) = &self.img_rect {
                                ui.add(
                                    egui::Image::from_texture(rect)
                                );
                            }else {
                                ui.label("No Color Display Texture found");
                            };
                            if let Some(bar) = &self.img_bar {
                                ui.add(
                                    egui::Image::from_texture(bar)
                                );
                            }else {
                                ui.label("No Color Display Texture found");
                            };
                            if ui.button("Generate").clicked() {
                                self.generate_color_display();
                            }
                        });
                    },
                    CompareState::Saturation => {
                        ui.add(egui::Slider::new(&mut self.color_display_threshhold,0.0 ..= 1.0).text("Color Display Threshold"));
                        let mut color_sorted:Vec<_> = self.color_list.iter_mut().collect();
                        color_sorted.sort_by(|a,b| {
                            if iris_color::HSL::from_rgb(&a.1.to_rgb()).s < (iris_color::HSL::from_rgb(&b.1.to_rgb()).s) {
                                return Ordering::Greater;
                            }else{
                                return Ordering::Less;
                            }
                        });
                        egui::CollapsingHeader::new("Colors").show(ui,|ui|{
                            egui::ScrollArea::vertical().max_height(100.0).auto_shrink([false,true]).show(ui, |ui| {
                                let aw = ui.available_width();
                                egui::Grid::new("Colors").spacing(Vec2::new(0.0,3.0)).show(ui,|ui|{
                                    let mut column_count = 0;
                                    for (id,c) in color_sorted.iter_mut(){
                                        if self.color_percent[id] >= self.color_display_threshhold{
                                            iris_color::color_display(ui,c);
                                            column_count += 1;
                                            if column_count > (aw/(ui.available_width()+3.0)) as i32 {
                                                ui.end_row();
                                                column_count = 0;
                                            }
                                        }
                                    }
                                });
                            });
                        });
                        egui::CollapsingHeader::new("Color Percentages").show(ui,|ui|{
                            egui::ScrollArea::vertical().max_height(100.0).show(ui, |ui| {
                                ui.with_layout(egui::Layout::top_down(egui::Align::TOP).with_cross_justify(true),|ui|{
                                    for (id,c) in &mut color_sorted{
                                        if self.color_percent[id] >= self.color_display_threshhold || self.color_display_threshhold == 0.0{
                                            iris_color::color_display_percent(ui,c,self.color_percent[id]);
                                        }
                                    }
                                });
                            });
                        });
                        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT),|ui|{
                            if let Some(rect) = &self.img_rect {
                                ui.add(
                                    egui::Image::from_texture(rect)
                                );
                            }else {
                                ui.label("No Color Display Texture found");
                            };
                            if let Some(bar) = &self.img_bar {
                                ui.add(
                                    egui::Image::from_texture(bar)
                                );
                            }else {
                                ui.label("No Color Display Texture found");
                            };
                            if ui.button("Generate").clicked() {
                                self.generate_color_display();
                            }
                        });
                    }
                }
                for (_,color) in self.color_list.iter_mut(){
                    if color.color_info_window_open {
                        color.color_info_window_show(ui.ctx());
                    }
                }
            }); 
            self.open = window_open;
        }
    }    
    fn scan_image(&mut self,ui:&mut egui::Ui){
        let image = ImageReader::open(self.path.clone()).unwrap().decode().unwrap(); 
        let size = image.width() as f64 * image.height() as f64;
        self.color_percent = HashMap::new();
        self.color_list = HashMap::new();
        let mut max_dist = f32::MIN;
        let mut min_dist = f32::MAX;
        let mut transparent_pixels:f64 = 0.0;
        for (_x,_y,rgba) in image.pixels(){
            if !(rgba.channels()[3]<= 0){
                let rgb = rgba.to_rgb();
                let mut rgb_already_registered = false;
                let mut closest_color_dist:f32 = f32::MAX;
                let mut closest_color_key:Option<u32> = None;
                if self.color_gradation >= 0.0 {
                    for (key,value) in self.color_list.iter_mut(){
                        let dist:f32;
                        match self.color_dist_type{
                            iris_color::ColorSpace::Rgb => dist = iris_color::rgb_distance(value.to_rgb(), rgb),
                            iris_color::ColorSpace::CieLab => dist = {
                                let lab_a = iris_color::CieLab::from_rgb(value.to_rgb());
                                let lab_b = iris_color::CieLab::from_rgb(rgb);
                                lab_a.distance_to_lab(&lab_b)
                            },
                            iris_color::ColorSpace::XYZ => dist = 0.0,
                            iris_color::ColorSpace::OkLab => dist = {
                                let lab_a = iris_color::OkLab::from_rgb(&value.to_rgb());
                                let lab_b = iris_color::OkLab::from_rgb(&rgb);
                                lab_a.distance_to_lab(&lab_b)
                            },
                        }
                        max_dist = max_dist.max(dist);
                        min_dist = min_dist.min(dist);
                        if dist <= self.color_gradation{
                            if closest_color_dist > dist {
                                closest_color_dist = dist;
                                closest_color_key = Some(*key);
                            }
                            rgb_already_registered = true;
                        }
                    }
                }
                if !rgb_already_registered {
                    self.color_percent.insert(self.color_list.len() as u32,(1.0/size)as f32);
                    self.color_pixel_count.insert(self.color_list.len() as u32, 1);
                    self.color_list.insert(self.color_list.len() as u32,iris_color::AvarageRgb::from_rgb(rgb));
                }else if let Some(cck) = closest_color_key{
                    if let Some(value) = self.color_list.get_mut(&cck){
                        if self.color_gradation > 0.0 {
                            value.avarage_with_rgb(&rgb,self.color_gradation);
                        }
                        if let Some(percent) = self.color_percent.get_mut(&cck){
                            *percent += (1.0/size) as f32;
                        }
                        if let Some(count) = self.color_pixel_count.get_mut(&cck){
                            *count += 1;
                        }
                    }
                }
            }else{
                transparent_pixels += 1.0;
            }
        }
        for (_,p) in self.color_percent.iter_mut(){
            *p = ((*p as f64 *size)/(size-transparent_pixels)) as f32;
        }

        for (_id,c) in self.color_list.iter_mut(){
            c.texture = Some(ui.ctx().load_texture("color_text",ColorImage::new([32,32],Color32::from_rgb(c.r, c.g, c.b)),Default::default()));
            for sub_c in c.colors.iter_mut() {
                sub_c.texture =Some(ui.ctx().load_texture("color_text",ColorImage::new([32,32],Color32::from_rgb(sub_c.r, sub_c.g, sub_c.b)),Default::default())); 
            }
        }
        self.clean_up();
    }
    fn clean_up(&mut self) {
        let mut id_remove = vec![];
        let id_list = self.color_list.clone();
        for ids in id_list.keys().into_iter().combinations(2){
            if !(id_remove.contains(&ids[0]) || id_remove.contains(&ids[1])){
                if iris_color::OkLab::from_rgb(&self.color_list[ids[0]].to_rgb()).distance_to_lab(&iris_color::OkLab::from_rgb(&self.color_list[ids[1]].to_rgb())) <= self.clean_up_value{
                    let other_value = self.color_list[ids[1]].clone();
                    let other_percent = self.color_percent[ids[1]].clone();
                    let other_pixel = self.color_pixel_count[ids[1]].clone();
                    if let Some(value) = self.color_list.get_mut(ids[0]){
                        value._avarage(&other_value);
                    }
                    if let Some(value) = self.color_percent.get_mut(ids[0]){
                        *value += other_percent;
                    }
                    if let Some(value) = self.color_pixel_count.get_mut(ids[0]){
                        *value = value.checked_add(other_pixel).unwrap_or(u32::MAX);
                    }
                    id_remove.push(ids[1]);
                } 
            }
        }
        for id in id_remove {
            self.color_list.remove(id);
            self.color_percent.remove(id);
            self.color_pixel_count.remove(id);
        }
    }
}

#[derive(Default)]
struct MyEguiApp {
    image_windows:Vec<ImageWindow>,
    image_creation_windows:Vec<iris_image_creation::ImageCreator>,
}

impl MyEguiApp {
    fn has_image_window(&self,path:PathBuf) -> bool{
        for im in &self.image_windows{
            if path == im.path{
                return true;
            }
        }
        return false;
    }
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        egui_extras::install_image_loaders(&cc.egui_ctx);
        Self::default()
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut image_window_to_remove:Vec<usize> = vec![];
            let mut image_creation_windows_to_remove:Vec<usize> = vec![];
            for (index,w) in self.image_windows.iter_mut().enumerate(){
                if w.open {
                    w.show(ui.ctx());
                }else{
                   image_window_to_remove.push(index); 
                }
            }
            for (index,w) in self.image_creation_windows.iter_mut().enumerate(){
                if w.open {
                    w.show(ui.ctx());
                }else{
                   image_creation_windows_to_remove.push(index); 
                }
            }
            for index in image_window_to_remove{
                self.image_windows.remove(index);
            }
            if ui.add(egui::Button::new("iw")).clicked(){
                self.image_creation_windows.push(iris_image_creation::ImageCreator::new());
            }
        }); 
    }
    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        if raw_input.dropped_files.len() >= 1 {
            for file in raw_input.dropped_files.iter(){
                if !self.has_image_window(file.path.clone().expect("No Path")){
                    self.image_windows.push(ImageWindow::new(file.clone())); 
                }
            }
        }
    }
}
    
