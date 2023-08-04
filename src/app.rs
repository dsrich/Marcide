use std::sync::mpsc;
use std::path::PathBuf;
use egui::{RichText, Color32, TextBuffer, Vec2};
use rfd::FileDialog;
use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR,MB_YESNOCANCEL, MB_ICONEXCLAMATION, MB_OK};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_S, VK_CONTROL, VK_F, VK_O, VK_R, VK_T, VK_N, VK_RMENU};
use windows_sys::w;
use std::fs::OpenOptions;
use self::code_editor::CodeEditor;
use std::io;
use std::io::{Write, Read};
use std::fs::File;
use dirs::home_dir;
//mod gks;
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
 // if we add new fields, give them default values when deserializing old state
mod code_editor;
mod richpresence;
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TemplateApp {
 
    unsafe_mode: bool,

    #[serde(skip)]
    to_find: String,

    #[serde(skip)]
    output: String,

    #[serde(skip)]
    output_window_is_open: bool,

    #[serde(skip)]
    spotify_window_is_open: bool,

    #[serde(skip)]
    window_title: String,

    #[serde(skip)]
    settings_window_is_open: bool,
    
    auto_save_to_ram: bool,

    auto_save: bool,

    #[serde(skip)]
    text: String,

    language: String,

    code_editor: CodeEditor,

    #[serde(skip)]
    last_save_path: Option<PathBuf>,

    auto_save_interval: u64,
    #[serde(skip)]
    rpc_sender: Option<mpsc::Sender<String>>,
    #[serde(skip)]
    autosave_sender: Option<mpsc::Sender<String>>,

    #[serde(skip)]
    session_started: chrono::DateTime<chrono::Local>,

    #[serde(skip)]
    code_editor_text_lenght: usize,

    #[serde(skip)]
    discord_presence_is_running: bool,

    #[serde(skip)]
    lines: Vec<String>,

    #[serde(skip)]
    finder_is_open: bool,

    #[serde(skip)]
    scroll_offset: Vec2,

    #[serde(skip)]
    is_found: Option<bool>,

    #[serde(skip)]
    occurences: usize,

    #[serde(skip)]
    opened_file: String
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            unsafe_mode: false,
            to_find: String::new(),

            output: String::new(),
            output_window_is_open: false,
            spotify_window_is_open: false,
            window_title: "Marcide".into(),
            session_started: chrono::Local::now(),
            settings_window_is_open: false,
            auto_save: true,
            auto_save_to_ram: false,
            text: String::new(),
            language: "py".into(),
            code_editor: CodeEditor::default(),
            last_save_path: None,
            auto_save_interval: 15,
            autosave_sender: None,
            rpc_sender: None,
            code_editor_text_lenght: 0,
            discord_presence_is_running: false,
            lines: Vec::new(),
            finder_is_open: false,
            scroll_offset: (0.0, 0.0).into(),
            is_found: None,
            occurences: 0,
            opened_file: String::new(),
        }
    }
}
impl TemplateApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {

        // Load previous app state (if any).
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }
        
        Default::default()
    }
}
fn finder(text : String, to_find : String) -> io::Result<Vec<usize>> {
    //let reader = BufReader::new(file);
    let mut line_numbers: Vec<usize> = Vec::new();

    for (line_number, line) in text.lines().enumerate() {
        let line_content = line;
        if line_content.contains(&to_find) {
            line_numbers.push(line_number + 1); // Add 1 to convert zero-based index to line number
        }
    }

    Ok(line_numbers)
}
fn mkdir(){
    let mut command = String::new();
    if let Some(home_dir) = home_dir() {
        command = format!("mkdir {}\\%marcide.temp%", home_dir.display())
    }
    let cmdcomm = std::process::Command::new("cmd")
        .arg("/C")
        .arg(command)
        .status();
    match cmdcomm {
        Ok(_) => {println!("Failed to excecute command!")}
        Err(_) => {}
    }
}
fn rmdir() {
    let mut command = String::new();
    if let Some(home_dir) = home_dir() {
        command = format!("rmdir /s /q {}\\%marcide.temp%", home_dir.display())
    }
    let cmdcomm = std::process::Command::new("cmd")
        .arg("/C")   
        .arg(command)    
        .status();
    match cmdcomm {
        Ok(_) => {println!("Failed to excecute command!")}
        Err(_) => {}
    }
}
fn runfile(path : Option<PathBuf>, language : String) -> std::process::Output {
    //first check if the env variables are set
    let env = std::process::Command::new(language.clone()).output();
    match env {
        Ok(_) => {/*Env variable found*/},
        Err(_) => {
            //notify user
            println!("env varaible not found");
            unsafe {
                MessageBoxW(0,  w!("Troubleshoot : did you add the compiler to the PATH system variable?\nDid you check the spelling of which programming language you want to syntax?"), w!("Fatal error"), MB_ICONERROR | MB_OK)
            };
            }
    }
    let command_to_be_excecuted = format!("{} {}",/*lang if first asked so we can decide which script compiler needs to be run ie: py test.py or lua test.lua */ language, path.unwrap().display());
    let cmdcomm = std::process::Command::new("cmd")
        .arg("/C")   
        .arg(command_to_be_excecuted)    
        .output();
    match cmdcomm {
        Ok(mut ok) => {
            if ok.stdout.len() == 0 {
                ok.stdout = ok.stderr.clone();
            };
            ok
        }
        Err(_) => {unsafe { MessageBoxW(0,  w!("Troubleshoot : Did you add python / lua to system variables?\n(as py | as lua)"), w!("Fatal error"), MB_ICONERROR | MB_OK) }; cmdcomm.unwrap()}
    }
}
fn count_lines(text: &str) -> usize {
    text.split('\n').count()
}
fn openfile(path : Option<PathBuf>) -> String {
    let mut contents = String::new();
        if let Some(file_path) = path {
            let mut file = File::open(file_path)
                .expect("Failed to open file");

            file.read_to_string(&mut contents)
                .expect("Failed to read file");
        }

        return contents;
}
fn savetofile(path : Option<PathBuf>, text : String){
        if let Some(file_path) = path {
            let mut file = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(file_path.clone()).expect("wrong folder dumbass");
            // Write some data to the file
            match write!(file ,"{}", text){
                Ok(_) => {},
                Err(e) => {
                    println!("Error opening the file : {}", e);
                }
            }
    }
}
impl eframe::App for TemplateApp {
    fn on_close_event(&mut self) -> bool {
        //remove temp dir
        rmdir();
        if !self.auto_save || self.last_save_path.is_none(){
            //implement save warnings using winapi
            unsafe{
                match MessageBoxW(0,  w!("Do you want to save before qutting the application?"), w!("Save before quitting"), MB_ICONERROR | MB_YESNOCANCEL){
                    //yes
                    6 => {
                        //save text then quit
                        let files = FileDialog::new()
                            .set_title("Save as")
                            .set_directory("/")
                            .save_file();
                        if files.clone().is_some(){
                            self.last_save_path = files.clone();
                            savetofile(files.clone(), self.text.clone());
                            return true;
                        }
                        else {
                            self.on_close_event();
                        }
                        
                    },
                    //no
                    7 => {
                        //quit
                        return true;
                    },
                    //cancel
                    2 => {
                        return false;
                    },
                    _ => {}
                };
            }
        }
        true
    }
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if !self.auto_save_to_ram {
            self.code_editor.code.clear();
        }
        eframe::set_value(storage, eframe::APP_KEY, self);
        
    }
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut go_to_offset : bool = false;
        
        let _projname: String = self.opened_file.clone();
        let starttime: String = self.session_started.format("%m-%d %H:%M:%S").to_string();
            let tx = self.rpc_sender.get_or_insert_with(||{
                let (tx,rx) = mpsc::channel::<String>();
                std::thread::spawn(move || loop {
                    //reciver, text always gets updated
                    match rx.try_recv(){
                        Ok(text) => {
                            match richpresence::rpc(text,starttime.clone()){
                                Err(err) => println!("Richpresence failed : {}", err),
                                Ok(_) => {}
                            };
                        },
                        Err(_) => {
                            //code editor didnt recive new input, shit on it
                        }
                    };
                    
                });
                tx
            });
            match tx.send(self.opened_file.clone()){
                Ok(_) => {},
                Err(err) => {println!("Failed to send msg : {}",  err)}
            };
            
            self.discord_presence_is_running = true;       
        
        
        
        //title logic for * when unsaved
        
        //hotkeys
        if let Some(wintitle) = self.last_save_path.clone(){
            if let Some(file_name) = wintitle.file_name() {
                if let Some(file_name_str) = file_name.to_str() {
                    self.window_title = format!("Marcide - {}", file_name_str.to_string());
                    self.opened_file = file_name_str.clone().to_string();
                }
            }
            if self.code_editor_text_lenght != self.code_editor.code.len() {
                self.window_title = self.window_title.clone() + &"*".as_str();
            }
        }
        
        _frame.set_window_title(self.window_title.as_str());
        //get alt input => if true ctrl => false
        let altimput = unsafe {
            GetAsyncKeyState(VK_RMENU as i32)
        };
        let alt_is_pressed = (altimput as u16 & 0x8000) != 0;
        let nimput = unsafe {
            GetAsyncKeyState(VK_N as i32)
        };
        let nis_pressed = (nimput as u16 & 0x8000) != 0;
        let oimput = unsafe {
            GetAsyncKeyState(VK_O as i32)
        };
        let ois_pressed = (oimput as u16 & 0x8000) != 0;
        let rimput = unsafe {
            GetAsyncKeyState(VK_R as i32)
        };
        let ris_pressed = (rimput as u16 & 0x8000) != 0;
        let timput = unsafe {
            GetAsyncKeyState(VK_T as i32)
        };
        let tis_pressed = (timput as u16 & 0x8000) != 0;
        let has_focus = ctx.input(|i| i.focused);
        let ctrlimput = unsafe {
            GetAsyncKeyState(VK_CONTROL as i32)
        };
        let mut ctrlis_pressed = (ctrlimput as u16 & 0x8000) != 0;
        //listen if ENTER key is pressed so we can send the message, except when r or l shift is pressed
        let sinp = unsafe {
             GetAsyncKeyState(VK_S as i32)
        };
        let sis_pressed = (sinp as u16 & 0x8000) != 0;
        //if f is pressed
        let fimput = unsafe {
            GetAsyncKeyState(VK_F as i32)
        };
        let fis_pressed = (fimput as u16 & 0x8000) != 0;
        //save hotkey
        if alt_is_pressed {
            ctrlis_pressed = false;
        }
        if sis_pressed && ctrlis_pressed && has_focus {
            if self.last_save_path.is_some() {
                savetofile(self.last_save_path.clone(), self.text.clone());
                self.code_editor_text_lenght = self.code_editor.code.len();
            }
            else {
                let files = FileDialog::new()
                            .set_title("Save as")
                            .set_directory("/")
                            .save_file();
                    if files.clone().is_some(){
                        self.last_save_path = files.clone();
                        savetofile(files.clone(), self.text.clone());
                        self.code_editor_text_lenght = self.code_editor.code.len();
                    }
            }
        }
        //finder hotkey
        if ctrlis_pressed && fis_pressed && has_focus {
            if !self.finder_is_open {
                self.finder_is_open = true;
            }
        }
        if ctrlis_pressed && ris_pressed && has_focus {
            if !self.output_window_is_open {
                if self.unsafe_mode || self.language == "py" || self.language == "lua" {
                    //save to temp folder
                    if self.last_save_path.is_none() {
                        mkdir();
                        //C:\Users\%user_name%\marcide.temp
                        if let Some(mut home_dir) = home_dir() {
   
                            let to_push = format!("%marcide.temp%\\temp.{}", self.language);
                            home_dir.push(to_push);
                    
                            // Set the files variable
                            let files: Option<PathBuf> = Some(home_dir);
                            //save file
                            savetofile(files.clone(), self.text.clone());
                            //run file
                            self.output_window_is_open = true;
                            self.output = String::from_utf8_lossy(&runfile(files.clone(), self.language.clone()).stdout).to_string();
                                      
                            }
                        }
                        else {
                            let files = self.last_save_path.clone();
                            self.output_window_is_open = true;
                            self.output = String::from_utf8_lossy(&runfile(files.clone(), self.language.clone()).stdout).to_string();
                            
                        }
                }
                else {
                    unsafe{
                        MessageBoxW(0,  w!("This ide can only run .lua, and .py files safely\nIf you want to run something with a different extension, turn on unsafe mode"), w!("Fatal error"), MB_ICONEXCLAMATION | MB_OK);
                    }
                }
            }
        }
        if ctrlis_pressed && fis_pressed && has_focus {
            if !self.finder_is_open {
            self.finder_is_open = !self.finder_is_open;
            }
        }
        if ctrlis_pressed && ois_pressed && has_focus {
            let files = FileDialog::new()
                        .set_title("Open")
                        .set_directory("/")
                        .pick_file();
                    if files.clone().is_some(){
                        self.last_save_path = files.clone();
                        self.code_editor.code = openfile(files);
                        self.code_editor_text_lenght = self.code_editor.code.len();
                    }
        }
        if ctrlis_pressed && tis_pressed && has_focus {
            if !self.settings_window_is_open{
            self.settings_window_is_open = !self.settings_window_is_open;
            }
        }
        if ctrlis_pressed && nis_pressed && has_focus {
            let files = FileDialog::new()
                            .set_title("Save as")
                            .set_directory("/")
                            .save_file();
                    if files.clone().is_some(){
                        self.last_save_path = files.clone();
                        savetofile(files.clone(), self.text.clone());
                        self.code_editor_text_lenght = self.code_editor.code.len();
                    }
        }
        self.text = self.code_editor.code.clone();
        self.code_editor.language = self.language.clone();
        if self.output_window_is_open {
            egui::Window::new("Output")
            .open(&mut self.output_window_is_open)
            .show(ctx, |ui| {
                ui.label(RichText::from("Output").size(25.0).color(Color32::from_rgb(92, 92, 92)));
                ui.label(RichText::from(self.output.clone()).size(20.0).color(Color32::from_rgb(255, 255, 255)));
            });
        }
        if self.finder_is_open{
            egui::Window::new("Finder")
            .open(&mut self.finder_is_open)
            .show(ctx, |ui| {
                let occurence: usize;
                
                ui.label("Finder");
                ui.text_edit_singleline(&mut self.to_find);
                if ui.button("Search").clicked(){
                    let occur : io::Result<Vec<usize>> = finder(self.code_editor.code.clone(), self.to_find.clone());
                    if occur.as_ref().unwrap().len() == 0 {
                        self.is_found = None;
                    }
                    else {
                        go_to_offset = true;
                        occurence = occur.as_ref().unwrap()[0];
                        self.scroll_offset[1] = occurence as f32;
                        self.occurences = occur.unwrap().len();
                        self.is_found = Some(true);
                    }
                    //update scroll offset and done!
                } 
                if self.is_found.is_none() {
                    ui.colored_label(Color32::RED, "0 Occurences in the text");
                }
                else {
                    ui.colored_label(Color32::GREEN, format!("Appears in {} line(s)", self.occurences));
                }
            });
        }
        /* 
        if self.spotify_window_is_open{
            egui::Window::new("Spotify")
            .open(&mut self.spotify_window_is_open)
            .show(ctx, |ui|{

            });
        }
        */
        //autosave implementation
        if self.last_save_path.is_some() {
            //define recv sender
            
            let tx = self.autosave_sender.get_or_insert_with(||{
                let (tx,rx) = mpsc::channel::<String>();
                std::thread::spawn(move || loop {
                    //reciver, text always gets updated
                    match rx.try_recv(){
                        Ok(text) => {
                            let lines : Vec<&str> = text.lines().collect();
                            savetofile(Some(PathBuf::from(lines[1])), lines[0].to_string());
                        },
                        Err(_) => {
                            //code editor didnt recive new input, shit on it
                        }
                    };
                    
                });
                tx
            });
            if self.auto_save {
                if let Some(path) = self.last_save_path.clone() {
                    let data_to_send : String = format!("{}\n{}",self.code_editor.code.clone(), path.to_str().unwrap_or_default().to_string());
                    if self.code_editor_text_lenght < self.code_editor.code.len() {
                        tx.send(data_to_send).expect("Unable to send msg");
                        self.code_editor_text_lenght = self.code_editor.code.len();
                    }
                    
                }
            }   
        }
        if self.settings_window_is_open{
            egui::Window::new("Settings")
                .open(&mut self.settings_window_is_open)
                .show(ctx, |ui| {
                    ui.label(egui::RichText::from("File handling").size(20.0));
                    ui.checkbox(&mut self.auto_save, "Autosave to file");
                    if self.auto_save {
                        ui.label("Autosave will only turn on after you have saved your file somewhere.");
                    }
                    //this will enable/disable the savestate feature to codeeditor.code
                    ui.checkbox(&mut self.auto_save_to_ram, "Autosave");
                    if self.auto_save_to_ram {
                        ui.label("This will save your text temporarily in the application. ");
                    }
                    ui.separator();
                    ui.label(egui::RichText::from("Programming language").size(20.0));
                    ui.text_edit_singleline(&mut self.language);
                    if self.language == "quran" || self.language == "Quran" || self.language == "Korán" || self.language == "korán" {
                        ui.hyperlink_to("Quran", "https://mek.oszk.hu/06500/06534/06534.pdf");
                    }
                    if self.language == "marci1175" || self.language == "marci" || self.language == "Marci" || self.language == "Marcell" || self.language == "Varga Marcell" {
                        ui.separator();
                        ui.label("Credits");
                        ui.separator();
                        ui.label("Made by : Varga Marcell also known as marci1175 at 5111 days old");
                        ui.label("Had so much fun developing this lol.");
                        ui.hyperlink_to("Github", "https://github.com/marci1175");
                    }
                    ui.checkbox(&mut self.unsafe_mode, "Unsafe mode");
                    if self.unsafe_mode {
                        ui.label("With unsafe mode on you can try running any programming language");
                    }
                });
        }
        egui::TopBottomPanel::top("Settings").show(ctx, |ui|{
            /*Brownie recipie
                For 1 small baking dish 1 cup (2 sticks) butter 4 medium sized eggs 2 cups 
                brown sugar 3/4 cup cocoa powder (you can substitute 3.5 oz really dark chocolate) 1 cup flour 1/2 teaspoon vanilla extract 
                3/4 cup chopped almonds or other nuts  
    
                1. Melt the butter, let it cool a little  
                2. Beat eggs, sugar into butter  
                3. Mix in the rest of the ingredients  
                4. Put into baking dish  
                5. Bake ~30 minutes at 375 F 
            */
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui|{
                //define buttons
                let run = ui.button("Run");
                let find = ui.button("Find");
                let save = ui.button("Save");
                let save_as = ui.button("Save as");
                let open = ui.button("Open");
                let settings = ui.button("Settings");
                let support = ui.button("Support");
                if run.clicked() {
                    if self.unsafe_mode || self.language == "py" || self.language == "lua" {
                        //save to temp folder
                        if self.last_save_path.is_none() {
                            mkdir();
                            //C:\Users\%user_name%\marcide.temp
                            if let Some(mut home_dir) = home_dir() {

                                let to_push = format!("%marcide.temp%\\temp.{}", self.language);
                                home_dir.push(to_push);
                        
                                // Set the files variable
                                let files: Option<PathBuf> = Some(home_dir);
                                //save file
                                savetofile(files.clone(), self.text.clone());
                                //run file
                                self.output_window_is_open = true;
                                self.output = String::from_utf8_lossy(&runfile(files.clone(), self.language.clone()).stdout).to_string();

                                }
                            }
                            else {
                                let files = self.last_save_path.clone();
                                self.output_window_is_open = true;
                                self.output = String::from_utf8_lossy(&runfile(files.clone(), self.language.clone()).stdout).to_string();

                                }
                    }
                    else {
                        unsafe{
                            MessageBoxW(0,  w!("This ide can only run .lua, and .py files out of box"), w!("Fatal error"), MB_ICONEXCLAMATION | MB_OK);
                        }
                    }

                }
                if open.clicked() {
                    let files = FileDialog::new()
                        .set_title("Open")
                        .set_directory("/")
                        .pick_file();
                    if files.clone().is_some(){
                        self.last_save_path = files.clone();
                        self.code_editor.code = openfile(files);
                        self.code_editor_text_lenght = self.code_editor.code.len();
                    }
                }
                if save_as.clicked(){
                    let files = FileDialog::new()
                            .set_title("Save as")
                            .set_directory("/")
                            .save_file();
                    if files.clone().is_some(){
                        self.last_save_path = files.clone();
                        savetofile(files.clone(), self.text.clone());
                        self.code_editor_text_lenght = self.code_editor.code.len();
                    }
                };
                if save.clicked(){
                    if self.last_save_path.clone().is_none(){
                        let files = FileDialog::new()
                            .set_title("Save as")
                            .set_directory("/")
                            .save_file();
                        self.last_save_path = files.clone();
                        savetofile(self.last_save_path.clone(), self.text.clone());
                        self.code_editor_text_lenght = self.code_editor.code.len();
                    }
                    else if self.code_editor_text_lenght < self.code_editor.code.len() {
                        savetofile(self.last_save_path.clone(), self.text.clone());
                        self.code_editor_text_lenght = self.code_editor.code.len();
                    }
                    else {
                        //do nothing
                    }
                    
                }
                if find.clicked(){
                    self.finder_is_open = !self.finder_is_open;
                }
                if settings.clicked(){
                    self.settings_window_is_open = !self.settings_window_is_open;
                }
                if support.clicked(){
                    match webbrowser::open("https://discord.gg/7s3VRr4H6j"){
                        Ok(_) => {},
                        Err(_) => {}
                    };
                }
                /*
                if ui.button("Spotify").clicked(){
                    self.spotify_window_is_open = true;
                }
                */
                run.on_hover_text("You can run py and lua files\nCTRL + R");
                open.on_hover_text("CTRL + O");
                save_as.on_hover_text("CTRL + N");
                save.on_hover_text("CTRL + S");
                find.on_hover_text("CTRL + F");
                settings.on_hover_text("CTRL + T");
                support.on_hover_text("If you encounter errors make sure to contact support!");
            });
            
        });

        egui::TopBottomPanel::bottom("Stats").show(ctx, |ui|{
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui|{
                let lenght = self.text.len();
                let lines = count_lines(&self.text);
                let final_lenght = lenght - (lines - 1);
                ui.label(lines.to_string() + " : Lines");

                //separate self.label into a vector by whitespaces, then count them
                let trimmed_string = self.text.trim();
                let words: Vec<&str> = trimmed_string.split_whitespace().collect();
                ui.label(words.len().to_string() + " : Words");
                ui.label(final_lenght.to_string() + " : Characters");
                ui.separator();
                let current_datetime = chrono::Local::now();
                let datetime_str = current_datetime.format("%H:%M:%S ").to_string();
                let sessiondate_str = self.session_started.format("%H:%M:%S ").to_string();
                ui.label(format!("Session started : {}", sessiondate_str));
                ui.label(format!("Current time : {}", datetime_str));
                ctx.request_repaint();
            });
        });
        egui::CentralPanel::default().show(ctx, |ui|{
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui|{
                    self.scroll_offset = code_editor::CodeEditor::show(&mut self.code_editor, "id".into(), ui, self.scroll_offset, go_to_offset);
                });
        });
        
        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally choose either panels OR windows.");
            });
        }
    }
}
