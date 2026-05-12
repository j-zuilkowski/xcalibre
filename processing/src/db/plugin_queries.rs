use crate::error::ProcessingError;
use sqlx::SqlitePool;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct InstalledPlugin {
    pub id:           String,
    pub name:         String,
    pub version:      String,
    pub api_version:  i64,
    pub plugin_type:  String,
    pub dylib_path:   String,
    pub enabled:      bool,
    pub installed_at: String,
}

pub struct NewPlugin {
    pub name:        String,
    pub version:     String,
    pub api_version: i64,
    pub plugin_type: String,
    pub dylib_path:  String,
}

pub async fn install_plugin(pool: &SqlitePool, p: &NewPlugin) -> Result<String, ProcessingError> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO installed_plugins
         (id, name, version, api_version, plugin_type, dylib_path, installed_at, updated_at)
         VALUES (?,?,?,?,?,?,?,?)",
    )
    .bind(&id).bind(&p.name).bind(&p.version).bind(p.api_version)
    .bind(&p.plugin_type).bind(&p.dylib_path).bind(&now).bind(&now)
    .execute(pool).await.map_err(ProcessingError::DbError)?;
    Ok(id)
}

pub async fn list_plugins(pool: &SqlitePool) -> Result<Vec<InstalledPlugin>, ProcessingError> {
    sqlx::query_as::<_, InstalledPlugin>(
        "SELECT id, name, version, api_version, plugin_type, dylib_path,
                CAST(enabled AS BOOLEAN) as enabled, installed_at
         FROM installed_plugins ORDER BY name ASC",
    )
    .fetch_all(pool).await.map_err(ProcessingError::DbError)
}

pub async fn set_plugin_enabled(pool: &SqlitePool, id: &str, enabled: bool) -> Result<(), ProcessingError> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("UPDATE installed_plugins SET enabled = ?, updated_at = ? WHERE id = ?")
        .bind(enabled as i64).bind(&now).bind(id)
        .execute(pool).await.map_err(ProcessingError::DbError)?;
    Ok(())
}

pub async fn uninstall_plugin(pool: &SqlitePool, id: &str) -> Result<String, ProcessingError> {
    let row: Option<(String,)> = sqlx::query_as("SELECT dylib_path FROM installed_plugins WHERE id = ?")
        .bind(id).fetch_optional(pool).await.map_err(ProcessingError::DbError)?;
    sqlx::query("DELETE FROM installed_plugins WHERE id = ?")
        .bind(id).execute(pool).await.map_err(ProcessingError::DbError)?;
    Ok(row.map(|(p,)| p).unwrap_or_default())
}
