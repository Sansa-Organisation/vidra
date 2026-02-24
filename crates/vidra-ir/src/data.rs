use std::collections::HashMap;
use std::path::Path;

use vidra_core::VidraError;

/// A single row of bound data (column name → value).
pub type DataRow = HashMap<String, String>;

/// A dataset loaded from a CSV or JSON file.
#[derive(Debug, Clone)]
pub struct DataSet {
    /// Column names in order.
    pub columns: Vec<String>,
    /// Rows of data.
    pub rows: Vec<DataRow>,
}

impl DataSet {
    /// Load a dataset from a file path. Supports .csv and .json extensions.
    pub fn load(path: &Path) -> Result<Self, VidraError> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "csv" => Self::load_csv(path),
            "json" => Self::load_json(path),
            _ => Err(VidraError::Parse {
                message: format!("unsupported data file format: .{}", ext),
                file: path.display().to_string(),
                line: 0,
                column: 0,
            }),
        }
    }

    /// Load from a CSV file. First row is treated as column headers.
    fn load_csv(path: &Path) -> Result<Self, VidraError> {
        let content = std::fs::read_to_string(path)?;
        let mut lines = content.lines();

        let header = lines
            .next()
            .ok_or_else(|| VidraError::Parse {
                message: "CSV file is empty".into(),
                file: path.display().to_string(),
                line: 0,
                column: 0,
            })?;

        let columns: Vec<String> = header
            .split(',')
            .map(|s| s.trim().trim_matches('"').to_string())
            .collect();

        let mut rows = Vec::new();
        for line in lines {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let values = Self::parse_csv_line(line);
            let mut row = DataRow::new();
            for (i, col) in columns.iter().enumerate() {
                let val = values.get(i).cloned().unwrap_or_default();
                row.insert(col.clone(), val);
            }
            rows.push(row);
        }

        Ok(DataSet { columns, rows })
    }

    /// Simple CSV line parser that handles quoted fields.
    fn parse_csv_line(line: &str) -> Vec<String> {
        let mut result = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;

        for ch in line.chars() {
            if ch == '"' {
                in_quotes = !in_quotes;
            } else if ch == ',' && !in_quotes {
                result.push(current.trim().to_string());
                current = String::new();
            } else {
                current.push(ch);
            }
        }
        result.push(current.trim().to_string());
        result
    }

    /// Load from a JSON file. Expects an array of objects.
    fn load_json(path: &Path) -> Result<Self, VidraError> {
        let content = std::fs::read_to_string(path)?;
        let parsed: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| VidraError::Parse {
                message: format!("invalid JSON: {}", e),
                file: path.display().to_string(),
                line: 0,
                column: 0,
            })?;

        let arr = parsed.as_array().ok_or_else(|| VidraError::Parse {
            message: "JSON data file must be an array of objects".into(),
            file: path.display().to_string(),
            line: 0,
            column: 0,
        })?;

        if arr.is_empty() {
            return Ok(DataSet {
                columns: Vec::new(),
                rows: Vec::new(),
            });
        }

        // Extract columns from the first object
        let first = arr[0].as_object().ok_or_else(|| VidraError::Parse {
            message: "JSON array items must be objects".into(),
            file: path.display().to_string(),
            line: 0,
            column: 0,
        })?;
        let columns: Vec<String> = first.keys().cloned().collect();

        let mut rows = Vec::new();
        for item in arr {
            if let Some(obj) = item.as_object() {
                let mut row = DataRow::new();
                for col in &columns {
                    let val = obj
                        .get(col)
                        .map(|v| match v {
                            serde_json::Value::String(s) => s.clone(),
                            other => other.to_string(),
                        })
                        .unwrap_or_default();
                    row.insert(col.clone(), val);
                }
                rows.push(row);
            }
        }

        Ok(DataSet { columns, rows })
    }
}

/// Interpolate template placeholders `{{key}}` in a string with values from a data row.
pub fn interpolate(template: &str, row: &DataRow) -> String {
    let mut result = template.to_string();
    for (key, value) in row {
        let pattern = format!("{{{{{}}}}}", key);
        result = result.replace(&pattern, value);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpolate_basic() {
        let mut row = DataRow::new();
        row.insert("name".to_string(), "Alice".to_string());
        row.insert("role".to_string(), "Engineer".to_string());

        assert_eq!(
            interpolate("Hello {{name}}, you are a {{role}}", &row),
            "Hello Alice, you are a Engineer"
        );
    }

    #[test]
    fn test_interpolate_no_placeholders() {
        let row = DataRow::new();
        assert_eq!(interpolate("No placeholders here", &row), "No placeholders here");
    }

    #[test]
    fn test_interpolate_missing_key() {
        let mut row = DataRow::new();
        row.insert("name".to_string(), "Bob".to_string());
        // {{missing}} stays as-is
        assert_eq!(
            interpolate("Hello {{name}}, you are {{missing}}", &row),
            "Hello Bob, you are {{missing}}"
        );
    }

    #[test]
    fn test_load_csv() {
        let dir = std::env::temp_dir();
        let csv_path = dir.join("vidra_test_data.csv");
        std::fs::write(
            &csv_path,
            "name,email,role\nAlice,alice@example.com,Engineer\nBob,bob@example.com,Designer\n",
        )
        .unwrap();

        let ds = DataSet::load(&csv_path).unwrap();
        assert_eq!(ds.columns, vec!["name", "email", "role"]);
        assert_eq!(ds.rows.len(), 2);
        assert_eq!(ds.rows[0].get("name").unwrap(), "Alice");
        assert_eq!(ds.rows[1].get("role").unwrap(), "Designer");

        let _ = std::fs::remove_file(&csv_path);
    }

    #[test]
    fn test_load_csv_quoted() {
        let dir = std::env::temp_dir();
        let csv_path = dir.join("vidra_test_quoted.csv");
        std::fs::write(
            &csv_path,
            "name,bio\n\"Alice\",\"Loves coding, coffee\"\n\"Bob\",\"Designer, artist\"\n",
        )
        .unwrap();

        let ds = DataSet::load(&csv_path).unwrap();
        assert_eq!(ds.rows[0].get("bio").unwrap(), "Loves coding, coffee");

        let _ = std::fs::remove_file(&csv_path);
    }

    #[test]
    fn test_load_json() {
        let dir = std::env::temp_dir();
        let json_path = dir.join("vidra_test_data.json");
        std::fs::write(
            &json_path,
            r#"[
                {"name": "Alice", "email": "alice@test.com", "count": 42},
                {"name": "Bob", "email": "bob@test.com", "count": 7}
            ]"#,
        )
        .unwrap();

        let ds = DataSet::load(&json_path).unwrap();
        assert_eq!(ds.rows.len(), 2);
        assert_eq!(ds.rows[0].get("name").unwrap(), "Alice");
        assert_eq!(ds.rows[1].get("count").unwrap(), "7");

        let _ = std::fs::remove_file(&json_path);
    }

    #[test]
    fn test_data_driven_template_flow() {
        // End-to-end: load data, interpolate template
        let dir = std::env::temp_dir();
        let csv_path = dir.join("vidra_test_template.csv");
        std::fs::write(
            &csv_path,
            "first_name,last_name,title\nAlice,Smith,CEO\nBob,Jones,CTO\n",
        )
        .unwrap();

        let ds = DataSet::load(&csv_path).unwrap();
        let template = "Welcome {{first_name}} {{last_name}}, {{title}} of Vidra";

        let results: Vec<String> = ds
            .rows
            .iter()
            .map(|row| interpolate(template, row))
            .collect();

        assert_eq!(results[0], "Welcome Alice Smith, CEO of Vidra");
        assert_eq!(results[1], "Welcome Bob Jones, CTO of Vidra");

        let _ = std::fs::remove_file(&csv_path);
    }
}
