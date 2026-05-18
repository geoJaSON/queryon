use std::fs::File;
use std::io::{BufWriter, Write};

use serde_json::{Map, Value};
use tauri::State;

use crate::error::{AppError, AppResult};
use crate::models::{ConnId, ExportData};
use crate::state::AppState;

/// Stream a result set to disk. `path` is chosen via the dialog plugin on the
/// frontend; `format` is "csv" or "json".
#[tauri::command]
pub async fn export_result(
    data: ExportData,
    format: String,
    path: String,
) -> AppResult<()> {
    let file = File::create(&path).map_err(|e| AppError::msg(e.to_string()))?;
    let mut w = BufWriter::new(file);

    match format.as_str() {
        "csv" => {
            let mut wtr = csv::Writer::from_writer(&mut w);
            wtr.write_record(&data.columns)
                .map_err(|e| AppError::msg(e.to_string()))?;
            for row in &data.rows {
                wtr.write_record(row.iter().map(|c| c.as_deref().unwrap_or("")))
                    .map_err(|e| AppError::msg(e.to_string()))?;
            }
            wtr.flush().map_err(|e| AppError::msg(e.to_string()))?;
        }
        "json" => {
            let arr: Vec<Value> = data
                .rows
                .iter()
                .map(|row| {
                    let mut obj = Map::new();
                    for (i, col) in data.columns.iter().enumerate() {
                        let v = row
                            .get(i)
                            .and_then(|c| c.clone())
                            .map(Value::String)
                            .unwrap_or(Value::Null);
                        obj.insert(col.clone(), v);
                    }
                    Value::Object(obj)
                })
                .collect();
            serde_json::to_writer_pretty(&mut w, &arr)
                .map_err(|e| AppError::msg(e.to_string()))?;
            w.flush().map_err(|e| AppError::msg(e.to_string()))?;
        }
        other => return Err(AppError::msg(format!("unknown export format: {other}"))),
    }
    Ok(())
}

/// On-demand: decode a geometry value to EWKT. Accepts the raw EWKB hex
/// (with or without a leading `\x`).
#[tauri::command]
pub async fn decode_geometry(
    state: State<'_, AppState>,
    id: ConnId,
    hex: String,
) -> AppResult<String> {
    let entry = state.pool(id)?;
    let h = hex.trim().trim_start_matches("\\x");
    let client = entry.pool.get().await?;
    let row = client
        .query_one(
            "SELECT ST_AsEWKT(decode($1, 'hex')::geometry)",
            &[&h],
        )
        .await?;
    Ok(row.get(0))
}
