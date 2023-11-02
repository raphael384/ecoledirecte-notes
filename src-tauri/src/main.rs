// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string, to_string_pretty, Error, Value};
use std::fs;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Body {
    identifiant: String,
    motdepasse: String,
    is_relogin: bool,
    uuid: String,
}

#[derive(Serialize, Deserialize)]
struct Infos {
    token: String,
    id: String,
}

#[derive(Serialize, Deserialize)]
struct OldNotes {
    mean: f64,
    coef_mean: f64,
    actual_mean: f64,
    actual_coef_mean: f64,
}

#[derive(Serialize, Deserialize)]
struct NoteSet {
    average: f64,
    evolution: f64,
    class_average: f64,
    coef_average: f64,
    coef_evolution: f64,
    coef_class_average: f64,
}

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/109.0.0.0 Safari/537.36";
const BASE_URL: &str = "https://api.ecoledirecte.com/";

#[tauri::command]
fn get() -> NoteSet {
    let (token, id) = check_new_connection();

    let notes = get_notes(token, id).unwrap();

    let courses = notes["data"]["periodes"][0]["ensembleMatieres"]["disciplines"]
        .as_array()
        .unwrap();

    let wrong_id = [2, 2862, 2863, 3, 4, 2890, 2972];

    let mut great_courses: Vec<Value> = vec![];

    for course in courses.iter() {
        if !wrong_id.contains(&course["id"].as_i64().unwrap()) {
            great_courses.push(course.clone());
        }
    }

    let mut sum: f64 = 0.;
    let mut len: f64 = 0.;

    let mut mean_calculator = |forwho| {
        for course in great_courses.iter() {
            if course["moyenne"].as_str().unwrap() != "" {
                sum += course[forwho]
                    .as_str()
                    .unwrap()
                    .replace(",", ".")
                    .parse::<f64>()
                    .unwrap();
                len += 1.;
            } else {
                sum += 0.;
                len += 0.;
            }
        }

        sum / len
    };

    let evolution_calculator = |old, new| {
        ((new - old) / old) * 100.
    };

    let old_notes_json = fs::read_to_string("./old_notes.json");

    let data: String;

    if old_notes_json.is_err() {

        let default_notes = OldNotes {
            mean: 10.,
            coef_mean: 10.,
            actual_mean: 10.,
            actual_coef_mean: 10.
        };
        
        fs::write("./old_notes.json", to_string_pretty(&default_notes).unwrap())
        .expect("Something goes wrong during file wrinting");

        data = serde_json::to_string(&default_notes).unwrap();
    } else {
        data = old_notes_json.unwrap();
    }

    let old_notes = from_str::<OldNotes>(&data).unwrap();

    let coef_mean = notes["data"]["periodes"][0]["ensembleMatieres"]["moyenneGenerale"]
        .as_str()
        .unwrap()
        .replace(",", ".")
        .parse::<f64>()
        .unwrap();

    let coef_class_mean = notes["data"]["periodes"][0]["ensembleMatieres"]["moyenneClasse"]
        .as_str()
        .unwrap()
        .replace(",", ".")
        .parse::<f64>()
        .unwrap();

    let mean = mean_calculator("moyenne");

    let note_set = NoteSet {
        average: mean,
        coef_average: coef_mean,
        coef_class_average: coef_class_mean,
        class_average: mean_calculator("moyenneClasse"),
        evolution: evolution_calculator(old_notes.mean, mean),
        coef_evolution: evolution_calculator(old_notes.coef_mean, coef_mean),
    };

    if coef_mean != old_notes.actual_coef_mean {
        let old_notes_json = OldNotes {
            mean: old_notes.actual_mean,
            coef_mean: old_notes.actual_coef_mean,
            actual_mean: mean,
            actual_coef_mean: coef_mean
        };

        fs::write("./old_notes.json", to_string_pretty(&old_notes_json).unwrap())
        .expect("Something goes wrong during file wrinting");
    }

    return  note_set;
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn get_notes(token: String, id: String) -> Result<Value, Error> {
    let client = Client::new();

    let body = "data={}";

    let res = client
        .post(format!("{BASE_URL}v3/eleves/{id}/notes.awp?verbe=get"))
        .body(body)
        .header("User-Agent", USER_AGENT)
        .header("X-token", token)
        .send();

    let json: Value = serde_json::from_str(&res.ok().unwrap().text().ok().unwrap())?;

    if json["message"].as_str().unwrap_or("") == "Token invalide !" {
        let infos = connect().unwrap();

        let result = get_notes(infos.0, infos.1);

        if result.is_ok() {
            return result;
        } else if result.is_err() {
            panic!("Someting goes wrong : {:#?}", result.err());
        }
    }

    Ok(json)
}

fn check_new_connection() -> (String, String) {
    // let myargs: Vec<String> = args().collect();

    // let default = String::from("");
    // let result = myargs.get(1).unwrap_or(&default);

    // let aliases = [
    //     "--new-connection",
    //     "-n",
    //     "--new",
    //     "--renew",
    //     "--renew-connection",
    // ];

    let is_existing: bool;

    match fs::metadata("./infos.json") {
        Ok(_) => is_existing = true,
        Err(_) => is_existing = false,
    }

    let read_json = || {
        let data = fs::read_to_string("./infos.json").expect("Unable to read file");

        from_str::<Infos>(&data)
    };

    let write_new_json = || {
        let (token, id) = connect().unwrap();

        let infos = Infos {
            token: token.clone(),
            id: id.clone(),
        };

        fs::write("./infos.json", to_string_pretty(&infos).unwrap())
            .expect("Something goes wrong during file wrinting");

        (token, id)
    };

    if !is_existing {
        write_new_json()
    } else {
        let infos = read_json();

        if infos.is_ok() {
            let infos = infos.unwrap();
            (infos.token, infos.id)
        } else {
            println!(
                "Something goes wrong during deserialization: {:#?}, creating new json...",
                infos.err().unwrap()
            );
            write_new_json()
        }
    }
}

fn connect() -> Result<(String, String), Error> {
    let client = Client::new();

    // you have to replace with yours

    let infos = Body {
        identifiant: "id".to_string(),
        motdepasse: "password".to_string(),
        is_relogin: false,
        uuid: "".to_string(),
    };

    let body = format!("data={}", to_string(&infos).unwrap());

    let res = client
        .post(format!("{BASE_URL}v3/login.awp"))
        .body(body)
        .header("User-Agent", USER_AGENT)
        .send();

    let json: Value = serde_json::from_str(&res.ok().unwrap().text().ok().unwrap())?;

    let token = json["token"].as_str().unwrap();
    let id = json["data"]["accounts"][0]["id"].as_u64().unwrap();

    let infos = Infos {
        token: token.to_string(),
        id: id.to_string(),
    };

    fs::write("./infos.json", to_string_pretty(&infos).unwrap())
        .expect("Something goes wrong during file wrinting");

    Ok((token.to_string(), id.to_string()))
}
