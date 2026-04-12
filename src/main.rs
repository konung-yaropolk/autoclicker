use enigo::{
    Button, Coordinate, Direction, Enigo, Keyboard, Mouse, Settings,
};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process;
use std::thread;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Action {
    #[serde(rename = "type")]
    action_type: String,

    #[serde(default)]
    x: i32,
    #[serde(default)]
    y: i32,

    #[serde(default)]
    text: String,

    #[serde(default = "default_delay")]
    delay: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Workflow {
    repetitions: u32,
    actions: Vec<Action>,
}

fn default_delay() -> f64 {
    1.0
}

fn main() {
    println!("🚀 Rust Autoclicker Suite");
    println!("================================\n");

    loop {
        println!("1. Run automation (load workflow.json or custom file)");
        println!("2. Record new workflow (mouse clicks + text)");
        println!("3. Show live mouse position helper");
        println!("4. Exit\n");

        print!("Choose (1-4): ");
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();
        let choice = choice.trim();

        match choice {
            "1" => run_automation(),
            "2" => record_workflow(),
            "3" => show_mouse_position(),
            "4" => {
                println!("👋 Goodbye!");
                break;
            }
            _ => println!("❌ Invalid option, try again.\n"),
        }
    }
}

// ====================== 1. RUN AUTOMATION ======================
fn run_automation() {
    let workflow = load_workflow();
    let repetitions = workflow.repetitions;
    let actions = workflow.actions;

    println!("✅ Loaded {} actions × {} repetitions", actions.len(), repetitions);

    let mut enigo = Enigo::new(&Settings::default()).expect("Failed to initialize Enigo");

    println!("\nPress ENTER to START (Ctrl+C to stop)...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    for rep in 1..=repetitions {
        println!("\n🔄 Repetition {}/{}", rep, repetitions);

        for action in &actions {
            if let Err(e) = execute_action(&mut enigo, action, rep) {
                eprintln!("❌ Error: {}", e);
                process::exit(1);
            }
        }
    }

    println!("\n✅ Automation completed successfully!");
    pause_to_menu();
}

fn execute_action(enigo: &mut Enigo, action: &Action, rep: u32) -> Result<(), String> {
    match action.action_type.as_str() {
        "click" => {
            enigo
                .move_mouse(action.x, action.y, Coordinate::Abs)
                .map_err(|e| e.to_string())?;
            enigo
                .button(Button::Left, Direction::Click)
                .map_err(|e| e.to_string())?;
            println!("✅ Clicked at ({}, {})", action.x, action.y);
        }
        "type" => {
            let final_text = action.text.replace("{rep}", &rep.to_string());
            enigo.text(&final_text).map_err(|e| e.to_string())?;
            println!("✅ Typed: {}", final_text);
        }
        _ => println!("⚠️ Unknown action type: {}", action.action_type),
    }

    thread::sleep(Duration::from_secs_f64(action.delay));
    Ok(())
}

// // ====================== 2. RECORDER ======================
// fn record_workflow() {
//     println!("\n🎥 Recorder started");
//     println!("Instructions:");
//     println!("   • Move mouse to desired position");
//     println!("   • Press ENTER          → Record CLICK");
//     println!("   • Type 't' + ENTER     → Record TEXT field");
//     println!("   • Type 'q' + ENTER     → Finish and print JSON\n");

//     let mut actions: Vec<Action> = Vec::new();

//     loop {
//         print!("\nCommand (ENTER = click, t = text, q = quit): ");
//         io::stdout().flush().unwrap();

//         let mut cmd = String::new();
//         io::stdin().read_line(&mut cmd).unwrap();
//         let cmd = cmd.trim().to_lowercase();

//         match cmd.as_str() {
//             "q" => break,
//             "t" => record_text_action(&mut actions),
//             "" => record_click_action(&mut actions),
//             _ => println!("❌ Unknown command."),
//         }
//     }

//     let workflow = Workflow {
//         repetitions: 5,
//         actions,
//     };

//     let json = serde_json::to_string_pretty(&workflow).unwrap();

//     println!("\n{}", "=".repeat(70));
//     println!("✅ RECORDING FINISHED! Copy the JSON below:\n");
//     println!("{}", json);
//     println!("\n{}", "=".repeat(70));

//     pause_to_menu();
// }

// ====================== 2. RECORDER ======================
fn record_workflow() {
    println!("\n🎥 Recorder started");
    println!("Instructions:");
    println!("   • Move mouse to desired position");
    println!("   • Press ENTER          → Record CLICK");
    println!("   • Type 't' + ENTER     → Record TEXT field");
    println!("   • Type 'q' + ENTER     → Finish and print JSON\n");

    let mut actions: Vec<Action> = Vec::new();

    loop {
        print!("\nCommand (ENTER = click, t = text, q = quit): ");
        io::stdout().flush().unwrap();

        let mut cmd = String::new();
        io::stdin().read_line(&mut cmd).unwrap();
        let cmd = cmd.trim().to_lowercase();

        match cmd.as_str() {
            "q" => break,
            "t" => record_text_action(&mut actions),
            "" => record_click_action(&mut actions),
            _ => println!("❌ Unknown command."),
        }
    }

    // === NEW: Ask for repetitions ===
    println!("\n{}", "=".repeat(60));
    print!("How many repetitions do you want? (default: 1): ");
    io::stdout().flush().unwrap();

    let mut rep_str = String::new();
    io::stdin().read_line(&mut rep_str).unwrap();
    let repetitions: u32 = rep_str.trim().parse().unwrap_or(1);

    let workflow = Workflow {
        repetitions,
        actions,
    };

    let json = serde_json::to_string_pretty(&workflow).unwrap();

    println!("\n✅ RECORDING FINISHED!");
    println!("Repetitions set to: {}", repetitions);
    println!("\nCopy the JSON below and save as workflow.json:\n");
    println!("{}", json);
    println!("\n{}", "=".repeat(60));

    pause_to_menu();
}


fn record_click_action(actions: &mut Vec<Action>) {
    let enigo = Enigo::new(&Settings::default()).expect("Enigo init failed");
    let (x, y) = enigo.location().unwrap_or((0, 0));

    print!("   Click at ({}, {}) → Delay (seconds, default 1.0): ", x, y);
    io::stdout().flush().unwrap();

    let mut delay_str = String::new();
    io::stdin().read_line(&mut delay_str).unwrap();
    let delay: f64 = delay_str.trim().parse().unwrap_or(1.0);

    actions.push(Action {
        action_type: "click".to_string(),
        x,
        y,
        text: String::new(),
        delay,
    });
    println!("   ✅ Recorded CLICK at ({}, {}) | delay = {}s", x, y, delay);
}

fn record_text_action(actions: &mut Vec<Action>) {
    let enigo = Enigo::new(&Settings::default()).expect("Enigo init failed");
    let (x, y) = enigo.location().unwrap_or((0, 0));

    println!("   Text field at ({}, {})", x, y);
    print!("   Base text (use {{rep}} for repetition): ");
    io::stdout().flush().unwrap();

    let mut text = String::new();
    io::stdin().read_line(&mut text).unwrap();
    let text = text.trim().to_string();

    print!("   Delay after typing (seconds, default 2.0): ");
    io::stdout().flush().unwrap();

    let mut delay_str = String::new();
    io::stdin().read_line(&mut delay_str).unwrap();
    let delay: f64 = delay_str.trim().parse().unwrap_or(2.0);

    actions.push(Action {
        action_type: "type".to_string(),
        x: 0,
        y: 0,
        text: text.clone(),
        delay,
    });
    println!("   ✅ Recorded TYPE \"{}\" | delay = {}s", text, delay);
}

// ====================== 3. LIVE MOUSE POSITION ======================
fn show_mouse_position() {
    let enigo = Enigo::new(&Settings::default()).expect("Enigo init failed");

    println!("\n🖱️  Live mouse position (Ctrl+C to stop)\n");

    loop {
        if let Ok((x, y)) = enigo.location() {
            print!("\rX: {:4} | Y: {:4}   ", x, y);
            io::stdout().flush().unwrap();
        }
        thread::sleep(Duration::from_millis(200));
    }
}

// ====================== HELPERS ======================
fn pause_to_menu() {
    println!("\nPress ENTER to return to main menu...");
    let mut input = String::new();
    let _ = std::io::stdin().read_line(&mut input);
}

// ====================== WORKFLOW LOADING ======================
fn load_workflow() -> Workflow {
    if let Some(arg) = env::args().nth(1) {
        if let Ok(w) = load_file(&PathBuf::from(arg)) {
            return w;
        }
    }

    if let Ok(mut path) = env::current_exe() {
        path.pop();
        path.push("workflow.json");
        if let Ok(w) = load_file(&path) {
            return w;
        }
    }

    println!("\n❌ workflow.json not found.");
    loop {
        println!("Enter full path to workflow JSON (or press Enter to cancel):");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let trimmed = input.trim();

        if trimmed.is_empty() {
            println!("Cancelled.");
            process::exit(0);
        }

        let path = PathBuf::from(trimmed);
        if let Ok(w) = load_file(&path) {
            return w;
        }
    }
}

fn load_file(path: &PathBuf) -> Result<Workflow, String> {
    if !path.exists() {
        return Err(format!("File not found: {}", path.display()));
    }

    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let workflow: Workflow = serde_json::from_str(&content)
        .map_err(|e| format!("JSON error: {}", e))?;

    println!("✅ Loaded: {}", path.file_name().unwrap().to_string_lossy());
    Ok(workflow)
}
