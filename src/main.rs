use chrono::{DateTime, Duration, Local, NaiveDate, TimeZone, Utc};
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
use std::io;
use unicode_width::UnicodeWidthStr;

/// A TUI to show the moon phase.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Date in YYYY-MM-DD format (defaults to today)
    #[arg(short, long)]
    date: Option<String>,
}

// Synodic month (new moon to new moon) in days
const SYNODIC_MONTH: f64 = 29.53058867;

// Known new moon: January 6, 2000 at 18:14 UTC
const KNOWN_NEW_MOON_SEC: i64 = 947182440; 

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

fn calculate_moon_phase(date: DateTime<Utc>) -> MoonStatus {
    let diff = date.timestamp() - KNOWN_NEW_MOON_SEC;
    let days_since_known_new_moon = diff as f64 / 86400.0;
    
    // Normalize to 0..29.53
    let mut age = days_since_known_new_moon % SYNODIC_MONTH;
    if age < 0.0 {
        age += SYNODIC_MONTH;
    }

    let phase_fraction = age / SYNODIC_MONTH;

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

    let angle = phase_fraction * 2.0 * std::f64::consts::PI;
    let illumination = 0.5 * (1.0 - angle.cos());

    MoonStatus {
        phase,
        phase_fraction,
        age_days: age,
        illumination: illumination * 100.0,
    }
}

struct MoonWidget {
    status: MoonStatus,
    show_labels: bool,
    language: Language,
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
                if ny < 0.0 || ny >= 1.0 || nx < 0.0 || nx >= 1.0 {
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
                     buf.get_mut(x, y).set_char(ch).set_fg(Color::Yellow);
                } else {
                    // Shadow (Earthshine)
                    buf.get_mut(x, y).set_char(ch).set_fg(Color::DarkGray);
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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut date: DateTime<Utc>) -> io::Result<()> {
    let mut show_labels = false;
    let mut show_info = true;
    let mut language = Language::English;
    loop {
        terminal.draw(|f| {
            let constraints = if show_info {
                vec![
                    Constraint::Percentage(80),
                    Constraint::Percentage(20),
                ]
            } else {
                vec![
                    Constraint::Percentage(100),
                    Constraint::Min(0),
                ]
            };

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(constraints)
                .split(f.size());

            let moon = calculate_moon_phase(date);
            
            // Render Custom Moon Widget
            f.render_widget(MoonWidget { 
                status: MoonStatus { phase: moon.phase, phase_fraction: moon.phase_fraction, age_days: moon.age_days, illumination: moon.illumination },
                show_labels,
                language,
            }, chunks[0]);

            // Info Area
            if show_info {
                let local_date: DateTime<Local> = DateTime::from(date);
                let info_text = vec![
                    Line::from(vec![
                        Span::raw("Date: "),
                        Span::styled(local_date.format("%Y-%m-%d").to_string(), Style::default().add_modifier(Modifier::BOLD)),
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
                    Line::from(Span::styled("Use <Left>/<Right> date. <l> labels. <L> language. <i> toggle info. <q> quit.", Style::default().fg(Color::DarkGray))),
                ];
                
                let info_block = Paragraph::new(info_text)
                    .block(Block::default().title(" Details ").borders(Borders::ALL))
                    .alignment(Alignment::Center);
                f.render_widget(info_block, chunks[1]);
            }
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Char('l') => {
                            show_labels = !show_labels;
                        }
                        KeyCode::Char('L') => {
                            language = language.next();
                        }
                        KeyCode::Char('i') => {
                            show_info = !show_info;
                        }
                        KeyCode::Left => {
                            date = date - Duration::days(1);
                        }
                        KeyCode::Right => {
                            date = date + Duration::days(1);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}


fn main() -> io::Result<()> {
    let args = Args::parse();
    
    // Parse date or use now
    let date = match args.date {
        Some(d) => {
            let naive = NaiveDate::parse_from_str(&d, "%Y-%m-%d")
                .expect("Invalid date format. Use YYYY-MM-DD")
                .and_hms_opt(12, 0, 0).unwrap(); // Midday
            Utc.from_utc_datetime(&naive)
        },
        None => Utc::now(),
    };

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run app
    let res = run_app(&mut terminal, date);

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
