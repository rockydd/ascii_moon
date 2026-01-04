use chrono::{DateTime, Duration, Local, NaiveDate, TimeZone, Utc};
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::Backend,
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph},
};
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Instant;
use unicode_width::UnicodeWidthStr;

mod poems;

use poems::{Poem, PoemLibrary};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Theme {
    Auto,
    Dark,
    Light,
}

impl std::str::FromStr for Theme {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "auto" => Ok(Theme::Auto),
            "dark" => Ok(Theme::Dark),
            "light" => Ok(Theme::Light),
            _ => Err("theme must be one of: auto, dark, light".to_string()),
        }
    }
}

fn detect_light_terminal_background() -> Option<bool> {
    // Heuristic: some terminals expose ANSI color indices via COLORFGBG="fg;bg" (or "fg:bg").
    // We treat bg 7/15 as "light background".
    let s = std::env::var("COLORFGBG").ok()?;
    let parts: Vec<&str> = s.split(|c| c == ';' || c == ':').collect();
    let bg = parts.last()?.trim().parse::<u16>().ok()?;
    Some(bg == 7 || bg == 15)
}

fn resolve_theme(theme: Theme) -> Theme {
    match theme {
        Theme::Auto => {
            if detect_light_terminal_background() == Some(true) {
                Theme::Light
            } else {
                Theme::Dark
            }
        }
        other => other,
    }
}

fn supports_truecolor() -> bool {
    // Most terminals that support 24-bit color set COLORTERM=truecolor (or 24bit).
    // iTerm does; macOS Terminal often does not.
    if let Ok(v) = std::env::var("COLORTERM") {
        let v = v.to_ascii_lowercase();
        if v.contains("truecolor") || v.contains("24bit") {
            return true;
        }
    }
    false
}

/// A TUI to show the moon phase.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Date in YYYY-MM-DD format (defaults to today)
    #[arg(short, long)]
    date: Option<String>,

    /// Render the moon to a specific number of lines (non-interactive)
    #[arg(long)]
    lines: Option<u16>,

    /// Auto-refresh period in minutes in interactive mode (0 disables auto-refresh)
    #[arg(long, default_value_t = 5)]
    refresh_minutes: u64,

    /// Hide the unlit (dark) part of the moon (renders shadow pixels as spaces)
    #[arg(long, default_value_t = false)]
    hide_dark: bool,

    /// Directory containing poem files (defaults to ./poems). Subfolders: en, zh, fr, ja, es
    #[arg(long)]
    poems_dir: Option<PathBuf>,

    /// Poem panel theme: auto (default), dark, or light
    #[arg(long, default_value = "auto")]
    theme: Theme,
}

// Synodic month (new moon to new moon) in days (average; used only to express "age" in days)
const SYNODIC_MONTH: f64 = 29.53058867;

const MOON_ART_RAW: &str = r#"                                                                                    #@&&%#%&(#&###&%###&&&&#/(@&(###.  %/#,                                                                             
                                                                            #&%%#&@%(&%##(*%&%##(###&&%&%#(#%&%%%&%###%(%#(#((@&&&(/.                                                                   
                                                                   .%&&##%###/%%#%%#&,%%&%%%%#%%%%%%&&&&%%%%##%&(#(%&(###%/##&##%(*(&%@#%*%/                                                            
                                                             /#/%&%#%(@%##%(((#&&&%%%%&%%%%&%&&&&&&&%%%%%%%%%%%#####%#%&#%#%%%%%%%%&&&&%%.%%%%%*(                                                       
                                                       ,(.@&%((#(@%#&%###(####((%&%%%%%%%&&&&&&&#&&&&&%%%##%###%####(%#%##%#%%%%%%&&%&&(%&&&&%&&%&&&#,                                                  
                                                   /(*/**,.%#((((*###%###((###%##%(%%%#%%%%%%%%%%%%#%%%%%%##%########%(####%%%%%%%%%&&&&%#%%%&%%&%%%%%%&#&                                              
                                               /*/((%%(#####((%((((((((#((#(##(###########%#%%#&%###%##(#%%%%#####(#%#(((##&#%##%%&&&%&%%%%%%%%#%/#%(#(/%%%###                                          
                                           ,*/,(/%/#/((#((((/(((((*//(////((#((#//(/((((#########(#(##(#(##(#(#%%((((#(#####&%###%%%%%&%&&&%%%%%#%%###(((##(*,,,/((##/####                                   
                                        .,.,///((/(((/(/*((/&*////**/*//********////((((((((#(##(##((#(#(#%%((((#(#####&%###%%%%%&%&&&%%%%%#%%###(((##(*,,,/((##/####                                   
                                     .,,,**////*********,,,*,**//(//***********//*****/*,**////((/((///((((((((((##(####%#((###%%%%%&&&&%%%&####%&(((((##((%####%((%(#&*                                
                                  ..,,,*,*,*,.,******//******,,*///////*****/******/********/////*/(/((///////(/(((//(/((((((((((#%%%%%&%&%%&%((#%#%(#(###(((#((#(##((#%%*@                             
                               ,..,.,,,*,*....,,,*//(*/////((/(((((//(/**/*/***/((((((///**///////////((///////**(////*********(#/###%#%%%%#%&%///(%####(##(//(((((#((#(/(#(*                           
                             ......,,,,,*,,.,,,****#&(((((#((/////(#//*/((####((//((//(((((((///////////((///////*//*/*/*/*******//((##%#%%#%%(#%%#%%%#(((#%##(##(%(#((((##(%*#*#                        
                           ........,,,,**,*/*///(((((((%#/////(/(%/////**//##(#*,,,*#/(/(%%%#*//((/////////*/////*////***/******(((((#%##%########%(((##((###%%(#((%(((###%((#((%#                      
                        ..........,,....,*///((//(((%##((((//(/(/*****,,,,***//(*/((*/(((#(####((#////(////###(#(((///(*///#((///###%####%#(##%####(///((####%###(##/(((####(##%#,%%                    
                      .........,........,/(//((//#(,,,,,**,**//**,,,,,,,,,**/******//(#%((((((##((/(/*/////(#(/(((//(((((((/////(###(%%%####%%%#%&##((/(/*//((#(*((##(######(((((##(#@                  
                    ...............,..,***/*////(/*,,,,*,,,,,.....,,,,,,,,************//(#%(#(##(((///((((/(((((((#(((((////////#(((###(#####%###%##((((((((((/#((/((##%&%%##(((%(%/(#(%                
                   ..................**/////*/(//,,,,.,,,,,.........,,,,*,,,,**,*,********//#/((###(##((((##((###(%#####(///(##(/#((//(((((((((########(#(##(%#((#%((##(#((%####(##%###((%%               
                 ...............,,,,*//**//*//*,**,.............,,.,,,,,,,***,*#,****/****##(((((###(((((###%##%(((#(((///////#(#((%(#(((((((((#######(///%##(#((###(#(((#((((((#(%#(#(//(#             
               ..,,............,,,,*******(/,,...,......,..,..,,,,,.,,,,**,,,,,*,***/*****/%(/(#/####((#(##(####((#(/((/(/#//###(((((//(//(#%#####%#%##/##((###%#%#((((#(//((((#/((##(((#((,            
              ................,.,,,,**,***,,,.,,,,,,................,,,,/,,,,,,********#//////###/(((###%%(((###((((/(/****/(///((/(///*(//(%#((#####((((########%%##(%(((((##((((#####%##((((          
            /*..............,,..,,.,.,*,,...,...,......,........,.,.,,,,*,,,,,*******//////////////#%%%%####(##((#(((((/**/*////((((((****/(#%###((((#####(%#%##%%&%#%(#((##%/#%(##((##(%%##(##         
           (.,.........,.,..,,,,...,,*,,....,,,,,,,,.,,,......,,,,,,,,,,**,,,,,,****///*////*(/((/(###(#(/****//(((((((/***(//***(//**/***//(((#%##%%%###(((##((##%%%##((((((###%(#########(#///        
           ..........,...,,,,,,,,,,,,,,..,,,,,.......,..,...,,.,,,,,,,,,,,,,,,******/**//**/**//(##((**,*,,,,,****/*/(((**,,**///(/****//((///(%%%#%#%#%(#(###%#%%%((((/%(((##&#%#&###%#%%#(*//       
         ,/.........,.,.,./,**,,,,,,,**,,,,,,,,...........,....,,,,...,,,,,,.,,*(*,***//#///***/**((#((*******,********,,***,****//(((////(####%%%#####&%%#%#(#%####%#(((##(#(#(((#(####%(#%%(##(/      
        ..............,,*/,,,,*,,,,.,.,,,,,,,,,,,,,,....,,,,,,,,,,,,,,,,,,,,,,,,*****////*****//////,,,,,,,,,,*,*,,,,,**,,,,,,/(##%%((///#%#%%##%%#%##%%%##%%###%&###(######%####/%(((##%###/#((#     
       ..............,,*//*,,****,,,,,*(,,*,,.,,,,.,,..,,,...,*,,,,,,,,,,,,,,**///**///((///****(***,,,,...,,,,,,,,,,,***,,,,,,*/((%((((####(%%%%%%%%&###%%%%###(###((##%%#(/((/((###%###%&###/(///    
      ...,..........,/*&&***,***,,,,,,,,,,,.,..,,,(,,,,,,,.,,,,,,,,,,,,,,,,,,/****//////////*******,,,,,,,,,,,,,,,,,,***,,,,,,,,,**//((((#%##%##%##%%####%&%#(###%(//((/#(#((/#&*%/##(((####((##((//   
     %.............,*,*****,,,*,,.,,..*...,,,..,,*,,,,,,,,..,,,,,,*****,,***///*/*****//(((//*/(/*/,,,*,,,,,,,,,,,,,,,*,*,,,,,....,,///(##%%%%%%%####%(##%((##(###/(((#(##*//**/(//,%((((((##%(((/##/   
     ...............,,,,,***,,,....,,..,,..,,,,,,,,,,,,,*,..,*,,,,*/***,**/////(///****,,**//((///*,,,,,,.,,,,,,,*,,,,,,,,,,....,,*/(###%&&%%%#%%%%#(*/((((((//((#%/*,*,,*,,*,(//((/(##((((/,,(///  
    ................,....*,*,,.,,,,,.,,,,,*/*,,.,*,*,,,,*,,,*,,,,,,,,,,,,**//////(//**,***,*/////*,,,,,,.,,,,,,,,,,,**,,,,.,,,,,,,.,**////%(#%%###%####(#/(#((/(((//(*******,,,,.,,.,*//(/%(//((*,,,,*. 
    .......................,***,,.,,.,,,*,*****,*,***,,,,**,**,,,,*,*,,,,,,,,*/(/////(//*/(/((%(/**.,,.......,,,,,,,*,,,,,,,,,,,,,,.,,*////(((#(//#(%#%#(#/(#((((((((****,*,,,,,.....*.,(/(##//(//,.,,,/ 
    ,,...............*....,,,*,,,*...,..,,,*/*,*,**,,,/(,*,,,**,,,,,*,,,,,,,,,,,***/(#((%##(((///*,,..........,.,,,**,,,,,,,.,,,,,,,,.(,*/(#((((/**//%&#(/#(##(####(/***,,,,,,,,,....,..*#(##(/,/**...,* 
    /,*...................,,,,****..,,,.***,,*,,**,***,,,**,*,*,**,,**,,,,*,,,,,,,**(/((#(##(//*/**,,......,,,,,,,,,*,,,,,,,,,,,,,,.,.,**/(((/////**/((#((####%#&#%#((/*,,,,,,,,,.....,,,(((##(#/,//,*,,/*
    */,...................,,,,****.*,,..,,*******/**/****,**,********,,,*/*,,.,,/*,/((/((///(((////*,*,,,..,/....,,,,,,,,,,,*,,,,,,....,/*//////,,******,*/(((%&&&%&&%%#/*,,,,,..,....,*((/((###/(#(//(**/*
    (*,...............,,.,,,*****,,..*/***(##///////**/******/*/*//*********,,/(/((////(//(/**//*,**,/*,****,/**,,,,,,,*,,,,,,,...,,,.,...,,/*******,,,,,//(((((((#%%%#/(/*,,*.,,..,....,/##(*((((((((/*,*/*
    (*,..................,,,******,*,*,,***/(*(/*/(((///(*//*//*/((/**/////////////////****,********,,,,**,,,*,/(/(*.,,(,.,,......,,,,.....,,***,,,,,,,,*//(%((/(#&%#%#/**,,,,,,,,.,.,*(#/((#,(((((/(//***
    (*#*...................*,*//**/**.,,,*///((((//*(((#((((///*///(///*,*,,,,***/////**,,,,,,,,,,,,,*/((/****,,,/(((#/***/*..,/*..........,....,,..,,...*,/(/((/(##%%#%#(///,,.,,,,,///#((/,*(,/(#/#(/(*,*
    /(/*,..........,....,.,**///(//**,,,,**((/(#(((///###((((///////*//*****,,,,,,,****,,,,*,,,,,,,,,,/#((/*,,**/**,#**/*,...............,.,,................,,*/((//#####(###//*(/#/(////#,.(,,//(((#//..,
    *((*................,***//#(////**,,****//(//(((#%%%&%(///(/////*/******,,,,,...,,*,,,,,,,,..,,.....,**,,//*///#///,,............,,*,,................*...,*,**/(######(##(((((((//*,/,.**/,#//(/#/,...
    ,(#,.................,,***/(((/**,,,,*,,**/(((((#%%%%%%(///////////***,,,,,,.,..,,***,,,,..,,.....,**,,//*///#///,,............,,*,,................*...,,//*/*****/(((((((/(((((((/(//*//,#/*/(/%(,,.,
    #*,.................,*,,/*////,,**//**#***//(//*/////*//////**//****,,,,**,,***///////***,,,,,.,.,****,((/(/**//*,,.....................,.............*/*,*****,*,***/((%####(((*(/((,(////,**//**....
    /**..................,**,,,/,,,******/*//*///*////**/*********/*//,,**,,,****//////((#///**//******,**((((((/(/**,.........,,,,......................,,..,**//*//,,*,..*/(/((//(**/#*,/*((////*//*,.,*
    *,(*,.................***,#/,,,*/**,*,****//////*/****//,,.,,,***,.....,,***((#((///(////((/**//,/((//((#((#(//***,...,*,.,,,.........,,......,.*,,,,,.,,,******,*,........,*.,,,***,,*/*/(/(**//,,,*,
    ../*/.................*..,,,,,,,,,**,,***///*//(//*****,**,,,,,**,,..,,,,****/////*/(*//(####((#(##((###((#((////,*,.,,,,,..........,..*,,,.,,*,,,,.,,,,*****,,,,..,..........,,/***/,//(/////((*,,,* 
    ,/%**/.....,......,........,,,,.*,,**/**,****/((///*//**,**,**,,,,,,,,,*,.***/,**//(((/(#((((((##%%%%%#%((/(/*/***,,,...........,....,,*,,*,,,..,,.,,,,*,**..,,,............,,,,,**/*/(///(/////*..* 
    */(**,......,........,......,,*,.,,,,*/******///*/((///***,,****,*****,**,,*/,******//(((((####(#%%&%%%%#(((///(/*//,,.,.,,,,,......,,,,,(#((%,,/,*,,,*/,,.,,.,,.........,,,,,,,,*(//(//*////////**  
    //#//,.....*.,(............,,,,,*,,**,***********/((///*****,***,*/*////**(*,,,**/**//((###%%%###%%%%%####(#(////*,**,,,,,,,,,,,,,,*.,*/&%####(/((/*((*,,,,,.,,....,..,,,,**,,**//*//**////////*  
    ,/%/(*,.,............,,....,,,,,,,,,***,*,,,,,****///(/*//*******///(((((///*****//(((###(%(#%%###%%#######&#((((((##(/*/*,*,,*,,,,,*//(#((((%(#%(//***,,,,,,.,...,...,,*,/(###((///(///(/##/(,   
     (*/#(,...,...,...,...,,...,,*,.,,*(,,*,,*,,,/(*,////////***(//(((((//(##((((((/(//(((((#####&###################%#%###(#(#(*/*,,,,,*((#((((((((/(#/(((*,,,,.,,.....,...,,*,/(###((///(///(/##/(,   
      (/(%(*,,**..,..,...,....,.,*,,/////*,,*,,,,/***/(/*//**/,**(##%#(#((###((#((((((((((###%######(####%##%###%##%%%%%%%%%%(((****/*(/((((#((#(((((///**//,/.,,,,,..,..*****/#(%%#/(##/(/////////    
       ///(//***#,,,.,.,,*,,.....,.,****//,...,,***,*///////(***,**/((###((((###%###((#(((((#####%############%%%##%&&%%&%#((//***/*****/(((((((###%#((((//*,/*,..,.....,,****/((##(/(////(///(///*     
        (((((//**,,,***/**/*,.,..,,,,,**/,,,,.,,,,,,*,**/((/*,*,.,**///((((#########(((#########%%%%%%%#%%#%%####%%%&&%%%%#((((//////*//((((((((##%&##%%/*(//((/,,,*,,,,,**,****/////(//////(//(/(      
        .##/(/(/*,*//*///////,,,......**,,....,,,,,,,*,*,*/#*/,*,,,**(//((((####(((((#(((#%(%#(##%%%&%%%###%%######%%%%%%%%####((/(((/////(//(#((#%%##(%#%#%%/*((**,,*,,,******/*/(////(////*(/////       
          ##((//*///*//*///(/*/,,.,...,,*,*.,...,,,,,,*,*,*/#*/,*,,,**(//((((####(((((#(((#%(%#(##%%%&%%%###%%######%%%%%%%%####((/(((/////(//(#((#%%##(%#%#%%/*((**,,*,,,******/*/(////(////*(/////       
          ##((/(/////(///(*/*,.*(,.,....,,,#/**(*,*,,******/((,,,,,**(//((((((##(((((((#(##(((%%###########%%%&&%%%&%%%%%#%%%%%%&##(##((((/*****/**/(##((/(/#((((**(#(******//****/****////(**         
           ##((/(/////(///(*/*,.*(,.,....,,,#/**(*,*,,******/((,,,,,**(//((((((##(((((((#(##(((%%###########%%%&&%%%&%%%%%#%%%%%%&##(##((((/*****/**/(##((/(/#((((**(#(******//****/****////(**         
            %%(#/////(##(/*//(/,./,,,/*,.,,//*(((*,,,****,,,/**,,,,,,*******(#((#((#((((##((####(((((##%#%%%##%%##%%##%%%%&&&&%%%%####/((/********//(&#%#%&#(/(#(/*///*****,**/*/**//***////**          
             %###%(/((((/*,*.....,,,,.,***,,/((//*,,*,,,,,,,,*,*,,,***/*/#**((#%((###((((((##((%######%#####%##%%%###%%%%&&&&&%&&%%#(//************(#%%#//*#((//********##((((/**///////*//            
              .##%(#((((/,,.........**(///***//**,,*,,****,,,***,**,,,,,,**(((#########((((##(######%%%%#%%%###%#####%#%%%%&%%&&&&&%#(/(//***/******(##(#(*((/((//***,*/(#(((//////*////*(/             
                #(((##(*%/*,,.,.......*(///*///***#(/*,***,*,,,...,,*,,,/(##(((###%###(###&%&%%##(###(#########((###%%#%#%#%%%%%&%%%##((/(/******,**/(((((#(/**/(/**////(#((*(((//(/**///,              
                  #((#%(#(/,,,,,......,*/**,*,,*/***/******,,,,,,,,,,,**(#(((%#####((((##%###%%(((((#######(#########%##%%%%%%%#%%%%%%/////#/(#%##(//(##(/#(((((/((((((/(%(#((((/****/                
                   /((####///,*,,,..,.,,,,**,*,,*//*****,,,,*/,,,,,,,,**///#(&%####((((####((##(((((######%##%%#%##%#((##%%%%#%%%%%%%%%#((((###%%##(/((##(#(#%#(#%#((((#%#(##((((/****                  
                     /(((#&%#(/*/,,,,,,,/****(/,,,*******,,,,,,,,,,,,,,***/*/(###%##(#####%####((#####%#%%%%%%%%#(##########%%%%&&%%#######%####((((#%###%%#%##%#%%%###(%%#((%((////                    
                       (((((%###(/***(**//(//*(,**,,/***,,,,,,,,,,,,,,*,***/###%##((##%%%%%%######%%%%%%%%%%%%######(###%##%&&&%%%#%%#%%%&%#%#(####(##%%%%%%%%%%%%%%#%%%#(#&((///*.                     
                         #/(((/((*(/////(//(/,,*/(///*((/*/*****//**//*(/**(###%%#%%%%%#&%%%%###%%%%%&%%%#%(#((#####%###%%%%%%%##%#%%%%%########(###%#%#(##%%%&&%%%%%%###%(////*                        
                           ///(((/**//((/((#*/,,***//*/#%%#(#/(((****######%%%%%%%&%%%&%%&&%%%##%%%%%%%%###(#(#(###%&&%###(#%%#######%%##(####%###########%%%&%%%%&&#(((///***                          
                              //(/((***(((//((***((***/(#%%&&#%%#%#(%#%%%&%&&&&&%%%%&&%%%&%&%%%%%##%##(########(#%&%%%%######%%%###(##(##(######%#%####%%%%%%%%%##///////***                            
                                /,/%(((/(/##((((%#(*(###((%##%%%%%#%%#%&%%&&&&&@@&&&&%%%%%%%%%%%%%#%##%##%%##%%%&%%####%#%%#%%##(#(####((###############%#%%####(///****                               
                                   ////(((#(#((#(((/##%(%###%##%%##%%%%&&&&&&%%&&&&&%%%&%%%%%&&&#%##%%%%###%%%%##%####(###%###(####(#(##########(##(#(#%#%((/(////***,                                  
                                      /*////((###(##(##((((##%%%%%&%&%&&&&&&&&&&&&&&&%&%&%%%%%%%###%&&%#%%######%%########(###(#%%%##(####(#####%%####%((((/////***.                                    
                                         //*//(//((#((#((#%#%%#%%%%%&&&%%%%%%&&&&&&&&&&&&&&&&&&%%%%%##(####((##%######%#%%%%%##(((####%%###%%%#%####(#(((/(///*,                                        
                                            ./////(((##(#((####%%%%%%%%#&&&&&%%%%&&&&&&%%%&%%%%%%%&%%##%%%%%%%%#((####%##(#(###((##%#######%%%####((////((/**                                           
                                                ***#/(((#((##((%#%%%%%&%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%####((##((((((####((#####%%%%%##%%%%%##(#(///#((/*,,                                               
                                                    ,**(((/((((###%#%%%%%&&&&%%&&&%&&&&&&%%&%%%%#%%%%#######((#(((((#(#(((####(######(##(((////////*,                                                   
                                                         *(((#(((###%%#%%%%%&&%%%%%%%%%%%%#&&%%%%%###%%%#((###(((####((#(###(#######(((((((/////                                                        
                                                               ((/(####(#%%%%%%#%%%%%%%%%%##%%%%%#%######(########%##((((((((###//(((/////                                                              
                                                                     .(((##(##%%%#%%%%%%%%%%%#%%##%%%%#((####(((((((((((/((((((////,                                                                    
                                                                              */(%%%%%%%%%##%##########(/(((/(((((////////.                                                                             
"#;

fn moon_lit_color(truecolor: bool) -> Color {
    if truecolor {
        Color::Rgb(232, 208, 88) // warm moonlight
    } else {
        // 256-color gold. Important for terminals that don't parse 24-bit SGR (38;2;...),
        // which can otherwise degrade to green (color index 2).
        Color::Indexed(214)
    }
}

fn moon_shadow_color(truecolor: bool) -> Color {
    if truecolor {
        Color::Rgb(92, 92, 98) // soft graphite
    } else {
        Color::Indexed(242)
    }
}

#[derive(Debug, Clone, Copy)]
enum MoonPhase {
    New,
    WaxingCrescent,
    FirstQuarter,
    WaxingGibbous,
    Full,
    WaningGibbous,
    LastQuarter,
    WaningCrescent,
}

impl MoonPhase {
    fn name(&self) -> &'static str {
        match self {
            MoonPhase::New => "New Moon",
            MoonPhase::WaxingCrescent => "Waxing Crescent",
            MoonPhase::FirstQuarter => "First Quarter",
            MoonPhase::WaxingGibbous => "Waxing Gibbous",
            MoonPhase::Full => "Full Moon",
            MoonPhase::WaningGibbous => "Waning Gibbous",
            MoonPhase::LastQuarter => "Last Quarter",
            MoonPhase::WaningCrescent => "Waning Crescent",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Language {
    English = 0,
    Chinese = 1,
    French = 2,
    Japanese = 3,
    Spanish = 4,
}

impl Language {
    fn next(&self) -> Self {
        match self {
            Language::English => Language::Chinese,
            Language::Chinese => Language::French,
            Language::French => Language::Japanese,
            Language::Japanese => Language::Spanish,
            Language::Spanish => Language::English,
        }
    }
    
    fn name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Chinese => "中文",
            Language::French => "Français",
            Language::Japanese => "日本語",
            Language::Spanish => "Español",
        }
    }
}

struct Feature {
    names: [&'static str; 5],
    lat: f64,
    lon: f64,
}

const LUNAR_FEATURES: &[Feature] = &[
    Feature { names: ["Oceanus Procellarum", "风暴洋", "Océan des Tempêtes", "嵐の大洋", "Océano de las Tormentas"], lat: 18.4, lon: -57.4 },
    Feature { names: ["Mare Imbrium", "雨海", "Mer des Pluies", "雨の海", "Mar de las Lluvias"], lat: 32.8, lon: -25.6 },
    Feature { names: ["Mare Serenitatis", "澄海", "Mer de la Sérénité", "晴れの海", "Mar de la Serenidad"], lat: 20.0, lon: 13.5 },
    Feature { names: ["Mare Tranquillitatis", "静海", "Mer de la Tranquillité", "静かの海", "Mar de la Tranquilidad"], lat: 3.5, lon: 22.4 },
    Feature { names: ["Mare Crisium", "危海", "Mer des Crises", "危難の海", "Mar de las Crisis"], lat: 17.0, lon: 58.5 },
    Feature { names: ["Tycho", "第谷", "Tycho", "ティコ", "Tycho"], lat: -43.3, lon: -11.2 },
    Feature { names: ["Copernicus", "哥白尼", "Copernic", "コペルニクス", "Copérnico"], lat: 9.6, lon: -20.1 },
    Feature { names: ["Kepler", "开普勒", "Kepler", "ケプラー", "Kepler"], lat: 8.1, lon: -38.0 },
    Feature { names: ["Aristarchus", "阿里斯塔克斯", "Aristarque", "アリスタルコス", "Aristarco"], lat: 23.7, lon: -47.4 },
    Feature { names: ["Plato", "柏拉图", "Platon", "プラトン", "Platón"], lat: 51.6, lon: -9.3 },
];

struct MoonStatus {
    phase: MoonPhase,
    phase_fraction: f64, // 0.0 to 1.0 (0=New, 0.5=Full, 1.0=New)
    age_days: f64,
    illumination: f64,
}

fn normalize_degrees(mut deg: f64) -> f64 {
    deg %= 360.0;
    if deg < 0.0 {
        deg += 360.0;
    }
    deg
}

fn deg_to_rad(deg: f64) -> f64 {
    deg * std::f64::consts::PI / 180.0
}

fn julian_day_utc(dt: DateTime<Utc>) -> f64 {
    // Unix epoch (1970-01-01T00:00:00Z) is JD 2440587.5
    let unix = dt.timestamp() as f64 + (dt.timestamp_subsec_nanos() as f64) * 1e-9;
    unix / 86400.0 + 2440587.5
}

fn calculate_moon_phase(date: DateTime<Utc>) -> MoonStatus {
    // This uses a common Meeus-style approximation:
    // compute Sun and Moon ecliptic longitudes and take their elongation.
    // This is far more accurate than assuming a constant-length synodic month.
    let jd = julian_day_utc(date);
    let d = jd - 2451545.0; // days since J2000.0

    // Sun (approx): mean longitude L and mean anomaly g
    let l0 = normalize_degrees(280.460 + 0.9856474 * d);
    let g = normalize_degrees(357.528 + 0.9856003 * d);
    let lambda_sun = normalize_degrees(
        l0 + 1.915 * deg_to_rad(g).sin() + 0.020 * deg_to_rad(2.0 * g).sin(),
    );

    // Moon (approx): mean longitude l, mean anomaly Mm, mean elongation D, argument of latitude F
    let l = normalize_degrees(218.316 + 13.176396 * d);
    let mm = normalize_degrees(134.963 + 13.064993 * d);
    let d_moon = normalize_degrees(297.850 + 12.190749 * d);
    let f = normalize_degrees(93.272 + 13.229350 * d);

    // Moon longitude with a set of major periodic terms (degrees)
    let lambda_moon = normalize_degrees(
        l + 6.289 * deg_to_rad(mm).sin()
            + 1.274 * deg_to_rad(2.0 * d_moon - mm).sin()
            + 0.658 * deg_to_rad(2.0 * d_moon).sin()
            + 0.214 * deg_to_rad(2.0 * mm).sin()
            - 0.186 * deg_to_rad(g).sin()
            - 0.059 * deg_to_rad(2.0 * d_moon - 2.0 * mm).sin()
            - 0.057 * deg_to_rad(2.0 * d_moon - mm - g).sin()
            + 0.053 * deg_to_rad(2.0 * d_moon + mm).sin()
            + 0.046 * deg_to_rad(2.0 * d_moon - g).sin()
            + 0.041 * deg_to_rad(mm - g).sin()
            - 0.035 * deg_to_rad(d_moon).sin()
            - 0.031 * deg_to_rad(mm + g).sin()
            - 0.015 * deg_to_rad(2.0 * f - 2.0 * d_moon).sin()
            + 0.011 * deg_to_rad(2.0 * d_moon - 4.0 * mm).sin(),
    );

    // Elongation (0..360): 0=new, 180=full
    let elongation_deg = normalize_degrees(lambda_moon - lambda_sun);
    let phase_fraction = elongation_deg / 360.0;

    // Express "age" in days using the mean synodic month (good enough for display).
    let age = phase_fraction * SYNODIC_MONTH;

    let segment = (phase_fraction * 8.0).round() as i32 % 8;
    let phase = match segment {
        0 => MoonPhase::New,
        1 => MoonPhase::WaxingCrescent,
        2 => MoonPhase::FirstQuarter,
        3 => MoonPhase::WaxingGibbous,
        4 => MoonPhase::Full,
        5 => MoonPhase::WaningGibbous,
        6 => MoonPhase::LastQuarter,
        7 => MoonPhase::WaningCrescent,
        _ => MoonPhase::New,
    };

    let illumination = 0.5 * (1.0 - deg_to_rad(elongation_deg).cos());

    MoonStatus {
        phase,
        phase_fraction,
        age_days: age,
        illumination: illumination * 100.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn illumination_close_to_timeanddate_example_2025_12_13_utc() {
        // timeanddate.com shows ~37.1% illumination for Washington DC at Dec 12, 2025 11:46:50 pm local.
        // That corresponds to 2025-12-13 04:46:50 UTC (EST is UTC-5).
        // Source: https://www.timeanddate.com/moon/phases/
        let dt = Utc.with_ymd_and_hms(2025, 12, 13, 4, 46, 50).unwrap();
        let moon = calculate_moon_phase(dt);
        let expected = 37.1;
        let diff = (moon.illumination - expected).abs();
        assert!(
            diff <= 6.0,
            "illumination {:.2}% differs too much from expected {:.1}% (diff {:.2}%)",
            moon.illumination,
            expected,
            diff
        );
    }

    #[test]
    fn near_full_moon_is_highly_illuminated_2025_12_04_utc() {
        // timeanddate.com lists Full Moon on Dec 4, 2025 at 6:14 pm (Washington DC).
        // That's 2025-12-04 23:14:00 UTC.
        // Source: https://www.timeanddate.com/moon/phases/
        let dt = Utc.with_ymd_and_hms(2025, 12, 4, 23, 14, 0).unwrap();
        let moon = calculate_moon_phase(dt);
        assert!(
            moon.illumination >= 95.0,
            "expected near-full illumination, got {:.2}%",
            moon.illumination
        );
    }
}

struct MoonWidget {
    status: MoonStatus,
    show_labels: bool,
    language: Language,
    hide_dark: bool,
    truecolor: bool,
}

#[derive(Debug, Clone)]
struct PoemViewState {
    poem: Poem,
    glow_phase: u64,
    last_anim: Instant,
    twinkle_seed: u64,
    twinkles: Vec<Twinkle>,
    // Fade-in state for poem body lines: 0..=LINE_FADE_STEPS.
    line_fade: Vec<u8>,
    fade_idx: usize,
    fade_step: u8,
}

#[derive(Debug, Clone)]
struct Twinkle {
    // Stored as coordinates relative to the poem inner area (0..w-1, 0..h-1).
    x: u16,
    y: u16,
    ttl: u16,
    kind: u8,
}

fn lcg_next_u32(seed: &mut u64) -> u32 {
    // Simple LCG for deterministic twinkles; good enough for UI.
    *seed = seed
        .wrapping_mul(6364136223846793005u64)
        .wrapping_add(1442695040888963407u64);
    (*seed >> 32) as u32
}

fn soft_palette(glow_phase: u64) -> (Color, Color, Color) {
    // A calm, romantic palette (lavender / moonlight / blush).
    // We keep it subtle and avoid rapid cycling.
    let step = (glow_phase / 12) % 3;
    match step {
        0 => (
            Color::Rgb(245, 223, 235), // title: blush
            Color::Rgb(206, 204, 235), // body: lavender
            Color::Rgb(170, 180, 210), // dim: mist
        ),
        1 => (
            Color::Rgb(240, 232, 250),
            Color::Rgb(200, 216, 240),
            Color::Rgb(165, 175, 205),
        ),
        _ => (
            Color::Rgb(250, 240, 235),
            Color::Rgb(210, 198, 238),
            Color::Rgb(160, 170, 200),
        ),
    }
}

const LINE_FADE_STEPS: u8 = 6;

fn reset_poem_fade(state: &mut PoemViewState) {
    state.line_fade = vec![0; state.poem.lines.len()];
    state.fade_idx = 0;
    state.fade_step = 0;
}

fn soft_palette_for_theme(glow_phase: u64, theme: Theme, truecolor: bool) -> (Color, Color, Color) {
    match theme {
        Theme::Light => {
            // Higher contrast on light terminals.
            let step = (glow_phase / 16) % 3;
            if truecolor {
                match step {
                    0 => (
                        Color::Rgb(92, 40, 88),   // title: plum
                        Color::Rgb(52, 62, 120),  // body: deep indigo
                        Color::Rgb(95, 95, 105),  // dim: graphite
                    ),
                    1 => (
                        Color::Rgb(70, 45, 115),
                        Color::Rgb(40, 75, 130),
                        Color::Rgb(90, 90, 100),
                    ),
                    _ => (
                        Color::Rgb(110, 40, 70),
                        Color::Rgb(65, 55, 120),
                        Color::Rgb(95, 95, 105),
                    ),
                }
            } else {
                // 256-color approximations for light backgrounds.
                match step {
                    0 => (Color::Indexed(90), Color::Indexed(25), Color::Indexed(244)),
                    1 => (Color::Indexed(91), Color::Indexed(24), Color::Indexed(244)),
                    _ => (Color::Indexed(89), Color::Indexed(61), Color::Indexed(244)),
                }
            }
        }
        _ => {
            if truecolor {
                soft_palette(glow_phase)
            } else {
                // 256-color "romantic" palette for dark terminals.
                let step = (glow_phase / 16) % 3;
                match step {
                    0 => (Color::Indexed(212), Color::Indexed(110), Color::Indexed(245)),
                    1 => (Color::Indexed(213), Color::Indexed(111), Color::Indexed(245)),
                    _ => (Color::Indexed(211), Color::Indexed(109), Color::Indexed(245)),
                }
            }
        }
    }
}

fn render_poem_lines_soft(
    poem: &Poem,
    line_fade: &[u8],
    glow_phase: u64,
    theme: Theme,
    truecolor: bool,
) -> Vec<Line<'static>> {
    let (title_c, body_c, dim_c) = soft_palette_for_theme(glow_phase, theme, truecolor);
    let mut out: Vec<Line> = Vec::new();

    out.push(Line::from(Span::styled(
        poem.title.clone(),
        // Avoid BOLD/ITALIC: Terminal.app can remap "bright colors" unexpectedly.
        Style::default().fg(title_c),
    )));

    if poem.author.is_empty() {
        out.push(Line::from(Span::styled("", Style::default().fg(dim_c))));
    } else {
        out.push(Line::from(Span::styled(
            format!("— {}", poem.author),
            Style::default().fg(dim_c),
        )));
    }

    out.push(Line::from(""));

    for (i, line) in poem.lines.iter().enumerate() {
        let level = line_fade.get(i).copied().unwrap_or(0).min(LINE_FADE_STEPS);
        if level == 0 {
            out.push(Line::from(Span::styled("", Style::default().fg(dim_c))));
            continue;
        }

        // Fade without DIM/BOLD/ITALIC: use dim color first, then body color.
        let style = if level < (LINE_FADE_STEPS / 2).max(1) {
            Style::default().fg(dim_c)
        } else {
            Style::default().fg(body_c)
        };

        {
            out.push(Line::from(Span::styled(
                line.clone(),
                style,
            )));
        }
    }

    out
}

fn twinkle_char(kind: u8) -> char {
    match kind % 4 {
        0 => '·',
        1 => '⋅',
        2 => '.',
        _ => '˙',
    }
}

fn update_twinkles(twinkles: &mut Vec<Twinkle>, seed: &mut u64, area: Rect) {
    // Maintain a small, persistent field of twinkles that fade slowly.
    // This avoids the "pop in/pop out" effect.
    if area.width < 6 || area.height < 6 {
        twinkles.clear();
        return;
    }

    // Decay
    for t in twinkles.iter_mut() {
        t.ttl = t.ttl.saturating_sub(1);
        // Keep within bounds if the pane resizes
        t.x = t.x.min(area.width.saturating_sub(1));
        t.y = t.y.min(area.height.saturating_sub(1));
    }
    twinkles.retain(|t| t.ttl > 0);

    // Target density: scale gently with pane size so larger terminals feel more "starry"
    // without becoming noisy.
    //
    // Roughly: one twinkle per ~90 cells, clamped to a tasteful range.
    let cells = area.width as usize * area.height as usize;
    let base = (cells / 90).clamp(10, 22);
    let jitter = lcg_next_u32(seed) as usize % 4; // +0..3
    let target = base + jitter;
    while twinkles.len() < target {
        // Spawn away from borders so it feels like "in the sky", not hugging the frame.
        let x = 1 + (lcg_next_u32(seed) as u16 % area.width.saturating_sub(2));
        let y = 1 + (lcg_next_u32(seed) as u16 % area.height.saturating_sub(2));
        let max_ttl = 28 + (lcg_next_u32(seed) as u16 % 26); // 28..53 ticks (~3.4s..6.4s at 120ms)
        let kind = (lcg_next_u32(seed) % 4) as u8;
        twinkles.push(Twinkle {
            x,
            y,
            ttl: max_ttl,
            kind,
        });
    }
}

fn render_twinkles(
    buf: &mut Buffer,
    area: Rect,
    twinkles: &[Twinkle],
    glow_phase: u64,
    theme: Theme,
    truecolor: bool,
) {
    // Draw twinkles *only* on blank cells so we don't overwrite poem text.
    let (_, _, dim_c) = soft_palette_for_theme(glow_phase, theme, truecolor);
    for t in twinkles {
        let x = area.left() + t.x;
        let y = area.top() + t.y;
        if x <= area.left() || x + 1 >= area.right() || y <= area.top() || y + 1 >= area.bottom()
        {
            continue;
        }

        if buf.get(x, y).symbol() != " " {
            continue;
        }

        // Keep it subtle; avoid BOLD/DIM modifiers for Terminal.app.
        let style = Style::default().fg(dim_c);

        buf.get_mut(x, y).set_char(twinkle_char(t.kind)).set_style(style);
    }
}

impl Widget for MoonWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Pre-process source art into a grid for easy sampling
        let source_lines: Vec<Vec<char>> = MOON_ART_RAW
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| l.chars().collect())
            .collect();
        
        if source_lines.is_empty() { return; }

        // Calculate bounding box of non-whitespace characters
        let mut min_x = usize::MAX;
        let mut max_x = 0;
        let mut min_y = usize::MAX;
        let mut max_y = 0;

        for (y, line) in source_lines.iter().enumerate() {
            for (x, &ch) in line.iter().enumerate() {
                if ch != ' ' {
                    if x < min_x { min_x = x; }
                    if x > max_x { max_x = x; }
                    if y < min_y { min_y = y; }
                    if y > max_y { max_y = y; }
                }
            }
        }

        if min_x > max_x || min_y > max_y { return; }

        let crop_w = (max_x - min_x + 1) as f64;
        let crop_h = (max_y - min_y + 1) as f64;

        // Aspect ratio of the cropped source art
        let art_aspect = crop_w / crop_h;

        let avail_w = area.width as f64;
        let avail_h = area.height as f64;

        // Calculate drawing dimensions to fit 'area' while maintaining aspect ratio
        let (draw_w, draw_h) = if avail_w / avail_h < art_aspect {
            // Limited by width
            (avail_w, avail_w / art_aspect)
        } else {
            // Limited by height
            (avail_h * art_aspect, avail_h)
        };

        // Center the drawing in the area
        let start_x = area.left() as f64 + (avail_w - draw_w) / 2.0;
        let start_y = area.top() as f64 + (avail_h - draw_h) / 2.0;

        let phase = self.status.phase_fraction;

        // Iterate over the target terminal area
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                // Normalized coordinates relative to the drawn moon box (0.0 to 1.0)
                let ny = (y as f64 - start_y) / draw_h;
                let nx = (x as f64 - start_x) / draw_w;

                // Check if we are inside the moon drawing box
                if !(0.0..1.0).contains(&ny) || !(0.0..1.0).contains(&nx) {
                    continue;
                }

                // Sample from Source Art (Nearest Neighbor) mapped to CROP box
                let src_y = (min_y as f64 + ny * crop_h).floor() as usize;
                let src_x = (min_x as f64 + nx * crop_w).floor() as usize;

                if src_y >= source_lines.len() { continue; }
                let row = &source_lines[src_y];
                let ch = if src_x < row.len() { row[src_x] } else { ' ' };

                // Circular Mask & Spherical Projection Logic
                let dx = nx - 0.5;
                let dy = ny - 0.5;
                let dist_sq = dx * dx + dy * dy;

                // Radius is 0.5. Radius^2 is 0.25.
                if dist_sq > 0.25 {
                    continue;
                }

                // Map to -1..1 range for sphere math
                let u = dx * 2.0;
                let v = dy * 2.0;
                
                // z is the depth of the sphere at this pixel (towards viewer)
                // x^2 + y^2 + z^2 = 1
                let z = (1.0 - u * u - v * v).sqrt();

                // Sun vector calculation
                // Angle 0 = New Moon (Sun behind Moon, Vector 0,0,-1)
                // Angle PI = Full Moon (Sun behind Earth, Vector 0,0,1)
                let angle = phase * 2.0 * std::f64::consts::PI;
                let sun_x = angle.sin();
                let sun_z = -angle.cos();

                // Dot product of Surface Normal (u, v, z) and Sun Vector (sun_x, 0, sun_z)
                // If positive, the point is illuminated.
                let intensity = u * sun_x + z * sun_z;

                if intensity > 0.0 {
                    // IMPORTANT: set full style to avoid attribute "leakage" (DIM/BOLD/ITALIC)
                    // when the layout changes (e.g. poem panel toggled).
                    buf.get_mut(x, y)
                        .set_char(ch)
                        .set_style(Style::default().fg(moon_lit_color(self.truecolor)));
                } else if !self.hide_dark {
                    // Shadow (Earthshine)
                    buf.get_mut(x, y)
                        .set_char(ch)
                        .set_style(Style::default().fg(moon_shadow_color(self.truecolor)));
                }
            }
        }

        // Render Labels
        if self.show_labels {
            for feature in LUNAR_FEATURES {
                // Orthographic projection
                let rad_lat = feature.lat.to_radians();
                let rad_lon = feature.lon.to_radians();
                
                let u = rad_lat.cos() * rad_lon.sin();
                let v = rad_lat.sin();
                
                // Project to screen UV (0..1)
                // In math, v is Up. In screen, ny goes Down.
                // Center is 0.5, 0.5
                // Scale 0.95 to pull labels slightly inwards.
                // Offset (-0.10, -0.10) to shift labels Down-Left (fixing Top-Right bias).
                let scale = 0.95;
                let u_adj = u * scale - 0.10;
                let v_adj = v * scale - 0.10;
                
                let nx = 0.5 + u_adj / 2.0;
                let ny = 0.5 - v_adj / 2.0; 
                
                let term_x = start_x + nx * draw_w;
                let term_y = start_y + ny * draw_h;
                
                let x_idx = term_x as u16;
                let y_idx = term_y as u16;

                // Simple collision check with screen bounds
                if x_idx >= area.left() && x_idx < area.right() && y_idx >= area.top() && y_idx < area.bottom() {
                    buf.get_mut(x_idx, y_idx).set_char('x').set_fg(Color::Red);
                    let label_x = x_idx + 1;
                    let name = feature.names[self.language as usize];
                    if label_x + (name.width() as u16) < area.right() {
                        buf.set_string(label_x, y_idx, name, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
                    }
                }
            }
        }
    }
}

fn placeholder_poem() -> Poem {
    Poem {
        title: "Moon".to_string(),
        author: "".to_string(),
        lines: vec!["(no poems found)".to_string()],
    }
}

fn pick_poem(lib: &PoemLibrary, lang: Language) -> Poem {
    lib.random_poem(lang).unwrap_or_else(placeholder_poem)
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut date: DateTime<Utc>,
    mut follow_now: bool,
    refresh_minutes: u64,
    mut hide_dark: bool,
    poems_dir: Option<PathBuf>,
    theme: Theme,
) -> io::Result<()> {
    let mut show_labels = false;
    let mut show_info = true;
    let mut language = Language::English;
    let mut show_poem = false;

    let theme = resolve_theme(theme);
    let truecolor = supports_truecolor();
    let poem_library = poems::load_poems(poems_dir.as_deref());
    let mut poem_state = PoemViewState {
        poem: pick_poem(&poem_library, language),
        glow_phase: 0,
        last_anim: Instant::now(),
        twinkle_seed: rand::random::<u64>(),
        twinkles: Vec::new(),
        line_fade: Vec::new(),
        fade_idx: 0,
        fade_step: 0,
    };
    reset_poem_fade(&mut poem_state);
    let tick_rate = if refresh_minutes == 0 {
        None
    } else {
        Some(std::time::Duration::from_secs(refresh_minutes.saturating_mul(60)))
    };
    let mut last_tick = Instant::now();
    let mut needs_redraw = true;
    loop {
        // Poem animation: slow, romantic, peaceful.
        // - Gentle breathing glow (slow phase increment)
        // - Fade-in by line (DIM -> normal)
        const ANIM_RATE: std::time::Duration = std::time::Duration::from_millis(120);
        if show_poem && poem_state.last_anim.elapsed() >= ANIM_RATE {
            poem_state.last_anim = Instant::now();
            poem_state.glow_phase = poem_state.glow_phase.wrapping_add(1);
            // Advance fade for the current line
            if poem_state.fade_idx < poem_state.poem.lines.len() {
                poem_state.fade_step = poem_state.fade_step.saturating_add(1);
                let level = poem_state.fade_step.min(LINE_FADE_STEPS);
                if let Some(slot) = poem_state.line_fade.get_mut(poem_state.fade_idx) {
                    *slot = level;
                }
                if poem_state.fade_step >= LINE_FADE_STEPS {
                    poem_state.fade_idx += 1;
                    poem_state.fade_step = 0;
                }
            }
            needs_redraw = true;
        }

        if needs_redraw {
            terminal.draw(|f| {
                // Clear the whole frame first so style modifiers from the previous layout
                // can't affect the new layout (macOS Terminal is especially sensitive to this).
                f.render_widget(Clear, f.size());

                let constraints = if show_info {
                    vec![Constraint::Percentage(80), Constraint::Percentage(20)]
                } else {
                    vec![Constraint::Percentage(100), Constraint::Min(0)]
                };

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints(constraints)
                    .split(f.size());

                let moon = calculate_moon_phase(date);

                // Main content area: Moon on the left, optional poem panel on the right.
                let main_cols = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(if show_poem {
                        // Ensure both panes have a minimum; moon will "shrink" naturally.
                        vec![Constraint::Min(18), Constraint::Min(28)]
                    } else {
                        vec![Constraint::Percentage(100), Constraint::Min(0)]
                    })
                    .split(chunks[0]);

                // Render Custom Moon Widget
                f.render_widget(
                    MoonWidget {
                        status: MoonStatus {
                            phase: moon.phase,
                            phase_fraction: moon.phase_fraction,
                            age_days: moon.age_days,
                            illumination: moon.illumination,
                        },
                        show_labels,
                        language,
                        hide_dark,
                        truecolor,
                    },
                    main_cols[0],
                );

                if show_poem {
                    let (title_c, _, dim_c) =
                        soft_palette_for_theme(poem_state.glow_phase, theme, truecolor);
                    let border_style = Style::default().fg(title_c);
                    let block = Block::default()
                        .title(" Moon Poem ")
                        .borders(Borders::ALL)
                        .border_style(border_style);
                    let inner = block.inner(main_cols[1]);
                    f.render_widget(block, main_cols[1]);

                    if inner.width >= 2 && inner.height >= 2 {
                        let poem_lines = render_poem_lines_soft(
                            &poem_state.poem,
                            &poem_state.line_fade,
                            poem_state.glow_phase,
                            theme,
                            truecolor,
                        );
                        let paragraph = Paragraph::new(poem_lines)
                            .alignment(Alignment::Left)
                            .style(Style::default().fg(dim_c))
                            .wrap(ratatui::widgets::Wrap { trim: false });
                        f.render_widget(paragraph, inner);

                        // Persistent twinkles on blank space.
                        // We update based on the current pane size, then render after poem text.
                        update_twinkles(&mut poem_state.twinkles, &mut poem_state.twinkle_seed, inner);
                        let buf = f.buffer_mut();
                        render_twinkles(
                            buf,
                            inner,
                            &poem_state.twinkles,
                            poem_state.glow_phase,
                            theme,
                            truecolor,
                        );
                    }
                }

                // Info Area
                if show_info {
                    let local_date: DateTime<Local> = DateTime::from(date);
                    let mode = if follow_now { "Now (auto)" } else { "Manual" };
                    let info_text = vec![
                        Line::from(vec![
                            Span::raw("Date: "),
                            Span::styled(
                                local_date.format("%Y-%m-%d").to_string(),
                                Style::default().add_modifier(Modifier::BOLD),
                            ),
                        ]),
                        Line::from(vec![
                            Span::raw("Mode: "),
                            Span::styled(mode, Style::default().fg(Color::Green)),
                        ]),
                        Line::from(vec![
                            Span::raw("Phase: "),
                            Span::styled(moon.phase.name(), Style::default().fg(Color::Cyan)),
                        ]),
                        Line::from(format!("Age: {:.1} days", moon.age_days)),
                        Line::from(format!("Illumination: {:.1}%", moon.illumination)),
                        Line::from(vec![
                            Span::raw("Language: "),
                            Span::styled(language.name(), Style::default().fg(Color::Green)),
                        ]),
                        Line::from(""),
                        Line::from(Span::styled(
                            "Use <Left>/<Right> date (switches to Manual). <n> now (auto). <l> labels. <L> language. <d> hide dark. <p> poem. <P> next poem. <i> toggle info. <q> quit.",
                            Style::default().fg(Color::DarkGray),
                        )),
                    ];

                    let info_block = Paragraph::new(info_text)
                        .block(Block::default().title(" Details ").borders(Borders::ALL))
                        .alignment(Alignment::Center);
                    f.render_widget(info_block, chunks[1]);
                }
            })?;
            needs_redraw = false;
        }

        // Timer tick: refresh "now" periodically
        if let Some(tick_rate) = tick_rate {
            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
                if follow_now {
                    date = Utc::now();
                }
                needs_redraw = true;
            }
        }

        // Wait for input/resize up to the next tick
        let timeout = {
            // Keep the UI responsive for animation even if refresh_minutes is large.
            let base = if let Some(tick_rate) = tick_rate {
                tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| std::time::Duration::from_secs(0))
            } else {
                std::time::Duration::from_millis(250)
            };
            if show_poem {
                base.min(ANIM_RATE)
            } else {
                base
            }
        };

        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Char('l') => {
                            show_labels = !show_labels;
                            needs_redraw = true;
                        }
                        KeyCode::Char('L') => {
                            language = language.next();
                            if show_poem {
                                poem_state.poem = pick_poem(&poem_library, language);
                                poem_state.glow_phase = 0;
                                poem_state.last_anim = Instant::now();
                                poem_state.twinkle_seed = rand::random::<u64>();
                                poem_state.twinkles.clear();
                                reset_poem_fade(&mut poem_state);
                            }
                            needs_redraw = true;
                        }
                        KeyCode::Char('i') => {
                            show_info = !show_info;
                            needs_redraw = true;
                        }
                        KeyCode::Char('d') => {
                            hide_dark = !hide_dark;
                            needs_redraw = true;
                        }
                        KeyCode::Char('p') => {
                            show_poem = !show_poem;
                            if show_poem {
                                poem_state.poem = pick_poem(&poem_library, language);
                                poem_state.glow_phase = 0;
                                poem_state.last_anim = Instant::now();
                                poem_state.twinkle_seed = rand::random::<u64>();
                                poem_state.twinkles.clear();
                                reset_poem_fade(&mut poem_state);
                            }
                            needs_redraw = true;
                        }
                        KeyCode::Char('P') => {
                            if show_poem {
                                poem_state.poem = pick_poem(&poem_library, language);
                                poem_state.glow_phase = 0;
                                poem_state.last_anim = Instant::now();
                                poem_state.twinkle_seed = rand::random::<u64>();
                                poem_state.twinkles.clear();
                                reset_poem_fade(&mut poem_state);
                                needs_redraw = true;
                            }
                        }
                        KeyCode::Char('n') => {
                            follow_now = true;
                            date = Utc::now();
                            last_tick = Instant::now();
                            needs_redraw = true;
                        }
                        KeyCode::Left => {
                            follow_now = false;
                            date -= Duration::days(1);
                            needs_redraw = true;
                        }
                        KeyCode::Right => {
                            follow_now = false;
                            date += Duration::days(1);
                            needs_redraw = true;
                        }
                        _ => {}
                    }
                }
                Event::Resize(_, _) => {
                    needs_redraw = true;
                }
                _ => {}
            }
        }
    }
}


// Helper function to convert ratatui::style::Color to ANSI foreground code
fn color_to_ansi_fg(color: Color) -> String {
    match color {
        Color::Reset => "\x1b[39m".to_string(),
        Color::Black => "\x1b[30m".to_string(),
        Color::Red => "\x1b[31m".to_string(),
        Color::Green => "\x1b[32m".to_string(),
        Color::Yellow => "\x1b[33m".to_string(),
        Color::Blue => "\x1b[34m".to_string(),
        Color::Magenta => "\x1b[35m".to_string(),
        Color::Cyan => "\x1b[36m".to_string(),
        Color::Gray => "\x1b[90m".to_string(), // Bright Black
        Color::DarkGray => "\x1b[30m".to_string(), // Often same as Black
        Color::LightRed => "\x1b[91m".to_string(),
        Color::LightGreen => "\x1b[92m".to_string(),
        Color::LightYellow => "\x1b[93m".to_string(),
        Color::LightBlue => "\x1b[94m".to_string(),
        Color::LightMagenta => "\x1b[95m".to_string(),
        Color::LightCyan => "\x1b[96m".to_string(),
        Color::White => "\x1b[97m".to_string(),
        Color::Rgb(r, g, b) => format!("\x1b[38;2;{};{};{}m", r, g, b),
        Color::Indexed(_) => "\x1b[39m".to_string(), // Default to reset
    }
}

fn print_moon(lines: u16, date: DateTime<Utc>, hide_dark: bool) -> io::Result<()> {
    let moon = calculate_moon_phase(date);

    // The moon art is roughly 160 chars wide and 80 chars high in the source.
    // This gives an aspect ratio of 2.0 (width/height).
    let aspect_ratio = 2.0;
    let width = (lines as f64 * aspect_ratio) as u16;

    // Don't let the width exceed the terminal width
    // In non-TTY scenarios, `size()` can fail; fall back to a reasonable default.
    let (terminal_width, _) = crossterm::terminal::size().unwrap_or((80, 0));
    let width = width.min(terminal_width);

    let area = Rect::new(0, 0, width, lines);
    let mut buffer = Buffer::empty(area);

    let widget = MoonWidget {
        status: moon,
        show_labels: false,
        language: Language::English,
        hide_dark,
        truecolor: supports_truecolor(),
    };
    widget.render(area, &mut buffer);

    // Manually print the buffer to stdout with color
    let mut stdout = io::stdout();
    let mut last_fg = Color::Reset;

    for y in 0..area.height {
        for x in 0..area.width {
            let cell = buffer.get(x, y);
            if cell.fg != last_fg {
                write!(stdout, "{}", color_to_ansi_fg(cell.fg))?;
                last_fg = cell.fg;
            }
            write!(stdout, "{}", cell.symbol())?;
        }
        writeln!(stdout, "\x1b[0m")?; // Reset color at end of line and print newline
    }

    stdout.flush()?;
    Ok(())
}


fn main() -> io::Result<()> {
    let args = Args::parse();
    
    // Parse date or use now
    let (date, follow_now) = match args.date {
        Some(d) => {
            let naive_date = NaiveDate::parse_from_str(&d, "%Y-%m-%d").map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Invalid date format. Use YYYY-MM-DD",
                )
            })?;
            let naive = naive_date
                .and_hms_opt(12, 0, 0)
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid date"))?; // Midday
            (Utc.from_utc_datetime(&naive), false)
        },
        None => (Utc::now(), true),
    };

    if let Some(lines) = args.lines {
        // Non-interactive print mode
        return print_moon(lines, date, args.hide_dark);
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run app
    let res = run_app(
        &mut terminal,
        date,
        follow_now,
        args.refresh_minutes,
        args.hide_dark,
        args.poems_dir.clone(),
        args.theme,
    );

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}
