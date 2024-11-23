use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;
use rayon::iter::IntoParallelRefIterator;

#[derive(Debug, Serialize, Deserialize)]
struct SaveData {
    #[serde(rename = "gameRecord")]
    game_record: HashMap<String, Vec<Option<ScoreRecord>>>,
    #[serde(rename = "saveInfo")]
    save_info: SaveInfo,
}

#[derive(Debug, Serialize, Deserialize)]
struct SaveInfo {
    #[serde(rename = "summary")]
    summary: Summary,
}

#[derive(Debug, Serialize, Deserialize)]
struct Summary {
    #[serde(rename = "rankingScore")]
    ranking_score: f64,
    #[serde(rename = "gameVersion")]
    game_version: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct ScoreRecord {
    score: i32,
    acc: f64,
    fc: bool,
}

#[derive(Debug, Serialize)]
struct ProcessedRecord {
    song_name: String,
    difficulty: String,
    score: i32,
    acc: f64,
    fc: bool,
    ranking_score: f64,
    game_version: String,
}

fn process_save_file(save_file_path: &Path) -> Result<Vec<ProcessedRecord>> {
    let content = fs::read_to_string(save_file_path)
        .with_context(|| format!("Failed to read file: {}", save_file_path.display()))?;
    let save_data: SaveData = serde_json::from_str(&content)
        .with_context(|| "Failed to parse JSON")?;
    let mut scores_and_rks = Vec::new();
    let ranking_score = save_data.save_info.summary.ranking_score;
    let game_version = save_data.save_info.summary.game_version.to_string();
    let difficulties = ["EZ", "HD", "IN", "AT"];

    for (song_id, song_scores) in save_data.game_record {
        let song_name = song_id.rsplit_once('.').map_or(song_id.clone(), |(base, suffix)| {
            if suffix.chars().all(|c| c.is_digit(10)) {
                base.to_string()
            } else {
                song_id.clone()
            }
        });

        for (i, score_record) in song_scores.iter().enumerate().take(4) {
            if let Some(record) = score_record {
                scores_and_rks.push(ProcessedRecord {
                    song_name: song_name.clone(),
                    difficulty: difficulties[i].to_string(),
                    score: record.score,
                    acc: record.acc,
                    fc: record.fc,
                    ranking_score,
                    game_version: game_version.clone(),
                });
            }
        }
    }

    Ok(scores_and_rks)
}

fn get_all_song_names(save_data_dir: &Path) -> Result<Vec<String>> {
    let mut song_names: HashSet<String> = HashSet::new();
    for entry in WalkDir::new(save_data_dir)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_dir() {
            let save_file_path = entry.path().join("save.json");
            if let Ok(content) = fs::read_to_string(&save_file_path) {
                if let Ok(save_data) = serde_json::from_str::<SaveData>(&content) {
                    for (song_id, _) in save_data.game_record {
                        let song_name = song_id.rsplit_once('.').map_or(song_id.clone(), |(base, suffix)| {
                            if suffix.chars().all(|c| c.is_digit(10)) {
                                base.to_string()
                            } else {
                                song_id.clone()
                            }
                        });
                        song_names.insert(song_name);
                    }
                }
            }
        }
    }
    let mut names: Vec<_> = song_names.into_iter().collect();
    names.sort();
    Ok(names)
}

fn write_to_csv(records: &[ProcessedRecord], output_path: &Path) -> Result<()> {
    let mut writer = csv::Writer::from_path(output_path)?;
    records.iter().for_each(|record| {
        writer.serialize(record).unwrap();
    });
    writer.flush()?;
    Ok(())
}

fn write_to_excel(records: &[ProcessedRecord], output_path: &Path) -> Result<()> {
    let workbook = xlsxwriter::Workbook::new(output_path.to_str().unwrap())?;
    let mut sheet = workbook.add_worksheet(None)?;

    let headers = ["song_name", "difficulty", "score", "acc", "fc", "ranking_score", "game_version"];
    for (i, header) in headers.iter().enumerate() {
        sheet.write_string(0, i as u16, header, None)?;
    }

    records.iter().enumerate().for_each(|(row, record)| {
        let row = row + 1;
        sheet.write_string(row as u32, 0, &record.song_name, None).unwrap();
        sheet.write_string(row as u32, 1, &record.difficulty, None).unwrap();
        sheet.write_number(row as u32, 2, record.score as f64, None).unwrap();
        sheet.write_number(row as u32, 3, record.acc, None).unwrap();
        sheet.write_boolean(row as u32, 4, record.fc, None).unwrap();
        sheet.write_number(row as u32, 5, record.ranking_score, None).unwrap();
        sheet.write_string(row as u32, 6, &record.game_version, None).unwrap();
    });

    workbook.close()?;
    Ok(())
}

fn main() -> Result<()> {
    let save_data_dir = PathBuf::from("saveData");
    let output_dir = PathBuf::from("rks_data_output");

    fs::create_dir_all(&output_dir)?;

    let song_names = get_all_song_names(&save_data_dir)?;

    for song_name in &song_names {
        let mut all_song_data = Vec::new();
        for entry in WalkDir::new(&save_data_dir)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_dir() {
                let save_file_path = entry.path().join("save.json");
                if let Ok(scores_and_rks) = process_save_file(&save_file_path) {
                    let song_data: Vec<_> = scores_and_rks
                        .into_iter()
                        .filter(|entry| entry.song_name == *song_name)
                        .collect();
                    all_song_data.extend(song_data);
                }
            }
        }

        if !all_song_data.is_empty() {
            let csv_path = output_dir.join(format!("{}.csv", song_name));
            let xlsx_path = output_dir.join(format!("{}.xlsx", song_name));

            write_to_csv(&all_song_data, &csv_path)?;
            write_to_excel(&all_song_data, &xlsx_path)?;
        }
    }

    Ok(())
}
