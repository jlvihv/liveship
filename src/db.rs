use anyhow::Result;
use once_cell::sync::OnceCell;
use sqlx::sqlite::SqlitePool;
use sqlx::Row;

use crate::model::{
    AppConfig, LiveRoomInfo, RecordStatus, RecordingHistory, RecordingHistoryWithInfo,
    RecordingPlan, RecordingPlanWithInfo,
};

static POOL: OnceCell<SqlitePool> = OnceCell::new();

/// 获取数据库连接池
pub fn pool() -> SqlitePool {
    POOL.get().unwrap().clone()
}

/// 初始化数据库连接池，并建表
pub async fn init() -> Result<()> {
    let pool = SqlitePool::connect("sqlite://data.db?mode=rwc").await?;
    POOL.set(pool).unwrap();
    // 建表
    config::create_table().await?;
    plan::create_table().await?;
    history::create_table().await?;
    live::create_table().await?;
    Ok(())
}

/// 配置相关
pub mod config {
    use super::*;

    // 如果不存在，则建立配置表
    pub async fn create_table() -> Result<()> {
        // 先检查是否存在 config 表，如果不存在则创建
        // 然后写入默认配置，配置是 AppConfig 序列化成 json 字符串
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS config (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                config TEXT NOT NULL
            );
            "#,
        )
        .execute(&pool())
        .await?;
        // 不存在才插入，有则不插入，先查询是否有数据
        let sql = r#"SELECT * FROM config"#;
        let row = sqlx::query(sql).fetch_optional(&pool()).await?;
        if row.is_none() {
            save_default().await?;
        }

        Ok(())
    }

    /// 保存默认配置
    pub async fn save_default() -> Result<()> {
        let video_path = dirs::video_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "./".to_string());
        // 使用系统视频路径加 "liveship" 作为默认保存路径
        let save_path = std::path::Path::new(&video_path)
            .join("liveship")
            .to_string_lossy()
            .to_string();
        let default_config = AppConfig {
            ffmpeg_path: "ffmpeg".to_string(),
            save_path,
            live_info_check_interval: 60,
        };
        let default_config_json = serde_json::to_string(&default_config)?;
        let sql = r#"INSERT INTO config (config) VALUES (?)"#;
        sqlx::query(sql)
            .bind(default_config_json)
            .execute(&pool())
            .await?;
        Ok(())
    }

    // 获取配置
    pub async fn get() -> Result<AppConfig> {
        let sql = r#"SELECT * FROM config"#;
        let row = sqlx::query(sql).fetch_one(&pool()).await?;
        // 从数据库中取出的 config 字符串，反序列化为 AppConfig 结构体
        let config = serde_json::from_str(&row.get::<String, _>("config"))?;
        Ok(config)
    }

    /// 设置配置
    pub async fn set(config: AppConfig) -> Result<()> {
        let config_json = serde_json::to_string(&config)?;
        let sql = r#"UPDATE config SET config = ?"#;
        sqlx::query(sql).bind(config_json).execute(&pool()).await?;
        Ok(())
    }
}

/// 录制计划相关
pub mod plan {
    use super::*;

    // 如果不存在，则建立录制计划表
    pub async fn create_table() -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS recording_plan (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL UNIQUE,
                strategy TEXT NOT NULL,
                stream_kind TEXT NOT NULL,
                stream_resolution TEXT NOT NULL,
                enabled BOOLEAN NOT NULL DEFAULT FALSE,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                updated_at INTEGER NOT NULL DEFAULT 0
            )
            "#,
        )
        .execute(&pool())
        .await?;
        Ok(())
    }

    // 新增录制计划
    pub async fn add(plan: &RecordingPlan) -> Result<()> {
        // 将录制策略转换为 json 字符串以方便保存
        let strategy = serde_json::to_string(&plan.strategy)?;
        // 如果有则更新，如果没有则新建
        let sql = r#"
            INSERT OR REPLACE INTO recording_plan (url, strategy, stream_kind, stream_resolution, enabled)
            VALUES (?, ?, ?, ?, TRUE)
        "#;
        sqlx::query(sql)
            .bind(&plan.url)
            .bind(&strategy)
            .bind(&plan.stream_kind)
            .bind(&plan.stream_resolution)
            .execute(&pool())
            .await?;
        Ok(())
    }

    /// 获取所有录制计划
    pub async fn get_all() -> Result<Vec<RecordingPlan>> {
        let sql = r#"
            SELECT * FROM recording_plan
        "#;
        let rows = sqlx::query(sql).fetch_all(&pool()).await?;
        let mut plans = Vec::new();
        for row in rows {
            let plan = RecordingPlan {
                id: row.get("id"),
                url: row.get("url"),
                strategy: serde_json::from_str(&row.get::<String, _>("strategy"))?,
                stream_kind: row.get("stream_kind"),
                stream_resolution: row.get("stream_resolution"),
                enabled: row.get("enabled"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            plans.push(plan);
        }
        Ok(plans)
    }

    /// 获取录制计划的详细版本，包括直播间信息
    pub async fn get_all_with_info() -> Result<Vec<RecordingPlanWithInfo>> {
        let sql = r#"
            SELECT rp.*, lri.anchor_name, lri.anchor_avatar, lri.title, lri.platform_kind, lri.room_cover FROM recording_plan rp
            LEFT JOIN live_room_info lri ON rp.url = lri.url
        "#;
        let rows = sqlx::query(sql).fetch_all(&pool()).await?;
        let mut plans = Vec::new();
        for row in rows {
            let plan = RecordingPlanWithInfo {
                id: row.get("id"),
                url: row.get("url"),
                strategy: serde_json::from_str(&row.get::<String, _>("strategy"))?,
                stream_kind: row.get("stream_kind"),
                stream_resolution: row.get("stream_resolution"),
                enabled: row.get("enabled"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                live_room_info: LiveRoomInfo {
                    url: row.get("url"),
                    anchor_name: row.get("anchor_name"),
                    anchor_avatar: row.get("anchor_avatar"),
                    title: row.get("title"),
                    platform_kind: serde_json::from_str(&row.get::<String, _>("platform_kind"))?,
                    room_cover: row.get("room_cover"),
                },
            };
            plans.push(plan);
        }
        Ok(plans)
    }

    /// 获取所有开启的录制计划
    pub async fn get_enabled() -> Result<Vec<RecordingPlan>> {
        let sql = r#"
            SELECT * FROM recording_plan WHERE enabled = TRUE
        "#;
        let rows = sqlx::query(sql).fetch_all(&pool()).await?;
        let mut plans = Vec::new();
        for row in rows {
            let plan = RecordingPlan {
                id: row.get("id"),
                url: row.get("url"),
                strategy: serde_json::from_str(&row.get::<String, _>("strategy"))?,
                stream_kind: row.get("stream_kind"),
                stream_resolution: row.get("stream_resolution"),
                enabled: row.get("enabled"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            plans.push(plan);
        }
        Ok(plans)
    }

    // 删除录制计划
    pub async fn delete(url: &str) -> Result<()> {
        let sql = r#"
            DELETE FROM recording_plan WHERE url = ?
        "#;
        sqlx::query(sql).bind(url).execute(&pool()).await?;
        Ok(())
    }

    // 更改录制计划状态
    pub async fn update_status(url: &str, enabled: bool) -> Result<()> {
        let sql = r#"
            UPDATE recording_plan SET enabled = ? WHERE url = ?
        "#;
        sqlx::query(sql)
            .bind(enabled)
            .bind(url)
            .execute(&pool())
            .await?;
        Ok(())
    }
}

pub mod history {
    use super::*;

    // 如果不存在，则建立录制历史表
    pub async fn create_table() -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS recording_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL,
                status TEXT NOT NULL,
                start_time INTEGER NOT NULL,
                end_time INTEGER NOT NULL,
                path TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool())
        .await?;
        Ok(())
    }

    // 新增录制历史
    pub async fn add(history: &RecordingHistory) -> Result<()> {
        // 先查询同 url 是否有录制中的历史，如果有则不插入，报错
        // 确保同一 url 同一时刻只能有一个录制
        let sql = r#"
            SELECT * FROM recording_history WHERE url = ? AND status = ?"#;
        let row = sqlx::query(sql)
            .bind(&history.url)
            .bind(serde_json::to_string(&RecordStatus::Recording)?)
            .fetch_optional(&pool())
            .await?;
        if row.is_some() {
            return Err(anyhow::anyhow!(format!(
                "已有相同 url 正在录制中：{}",
                history.url
            )));
        }
        // 将 RecordStatus 枚举序列化为 json 字符串保存
        let status = serde_json::to_string(&history.status)?;
        let sql = r#"
            INSERT INTO recording_history (url, status, start_time, end_time, path)
            VALUES (?, ?, ?, ?, ?)
        "#;
        sqlx::query(sql)
            .bind(&history.url)
            .bind(&status)
            .bind(&history.start_time)
            .bind(&history.end_time)
            .bind(&history.path)
            .execute(&pool())
            .await?;
        Ok(())
    }

    /// 获取所有录制历史
    pub async fn get_all() -> Result<Vec<RecordingHistory>> {
        let sql = r#"
            SELECT * FROM recording_history ORDER BY id DESC
        "#;
        let rows = sqlx::query(sql).fetch_all(&pool()).await?;
        let mut histories = Vec::new();
        for row in rows {
            let history = RecordingHistory {
                id: row.get("id"),
                url: row.get("url"),
                status: serde_json::from_str(&row.get::<String, _>("status"))?,
                start_time: row.get("start_time"),
                end_time: row.get("end_time"),
                path: row.get("path"),
                file_size: 0,
                deleted: false,
            };
            histories.push(history);
        }
        Ok(histories)
    }

    /// 获取录制历史的详细版本，包括直播间信息
    pub async fn get_all_with_info() -> Result<Vec<RecordingHistoryWithInfo>> {
        let sql = r#"
            SELECT rh.*, lri.anchor_name, lri.anchor_avatar, lri.title, lri.platform_kind, lri.room_cover FROM recording_history rh
            LEFT JOIN live_room_info lri ON rh.url = lri.url
            ORDER BY rh.id DESC
        "#;
        let rows = sqlx::query(sql).fetch_all(&pool()).await?;
        let mut histories = Vec::new();
        for row in rows {
            let history = RecordingHistoryWithInfo {
                id: row.get("id"),
                url: row.get("url"),
                status: serde_json::from_str(&row.get::<String, _>("status"))?,
                start_time: row.get("start_time"),
                end_time: row.get("end_time"),
                path: row.get("path"),
                file_size: 0,
                deleted: false,
                live_room_info: LiveRoomInfo {
                    url: row.get("url"),
                    anchor_name: row.get("anchor_name"),
                    anchor_avatar: row.get("anchor_avatar"),
                    title: row.get("title"),
                    platform_kind: serde_json::from_str(&row.get::<String, _>("platform_kind"))?,
                    room_cover: row.get("room_cover"),
                },
            };
            histories.push(history);
        }
        Ok(histories)
    }

    /// 删除录制历史，通过 id
    pub async fn delete(id: i64) -> Result<()> {
        let sql = r#"
            DELETE FROM recording_history WHERE id = ?
        "#;
        sqlx::query(sql).bind(id).execute(&pool()).await?;
        Ok(())
    }

    // 更新录制历史状态，应该只有一种可能，就是从录制中变为录制结束
    pub async fn end(url: &str) -> Result<()> {
        let sql = r#"
            UPDATE recording_history SET status = ?, end_time = ? WHERE url = ? AND status = ?
        "#;
        sqlx::query(sql)
            .bind(serde_json::to_string(&RecordStatus::NotRecording)?)
            .bind(chrono::Utc::now().timestamp_millis())
            .bind(url)
            .bind(serde_json::to_string(&RecordStatus::Recording)?)
            .execute(&pool())
            .await?;
        Ok(())
    }
}

pub mod live {
    use super::*;

    // 如果不存在，则建立直播间信息表
    pub async fn create_table() -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS live_room_info (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL UNIQUE,
                anchor_name TEXT NOT NULL DEFAULT '',
                anchor_avatar TEXT NOT NULL DEFAULT '',
                title TEXT NOT NULL DEFAULT '',
                platform_kind TEXT NOT NULL,
                room_cover TEXT NOT NULL DEFAULT ''
            )
            "#,
        )
        .execute(&pool())
        .await?;
        Ok(())
    }

    /// 新增直播间信息
    pub async fn add(info: &LiveRoomInfo) -> Result<()> {
        let platform_kind = serde_json::to_string(&info.platform_kind)?;
        let sql = r#"
            INSERT OR REPLACE INTO live_room_info (url, anchor_name, anchor_avatar, title, platform_kind, room_cover)
            VALUES (?, ?, ?, ?, ?, ?)
        "#;
        sqlx::query(sql)
            .bind(&info.url)
            .bind(&info.anchor_name)
            .bind(&info.anchor_avatar)
            .bind(&info.title)
            .bind(&platform_kind)
            .bind(&info.room_cover)
            .execute(&pool())
            .await?;
        Ok(())
    }

    /// 获取指定 url 的直播间信息
    pub async fn get(url: &str) -> Result<Option<LiveRoomInfo>> {
        let sql = r#"
            SELECT * FROM live_room_info WHERE url = ?
        "#;
        let row = sqlx::query(sql).bind(url).fetch_optional(&pool()).await?;
        if let Some(row) = row {
            let info = LiveRoomInfo {
                url: row.get("url"),
                anchor_name: row.get("anchor_name"),
                anchor_avatar: row.get("anchor_avatar"),
                title: row.get("title"),
                platform_kind: serde_json::from_str(&row.get::<String, _>("platform_kind"))?,
                room_cover: row.get("room_cover"),
            };
            Ok(Some(info))
        } else {
            Ok(None)
        }
    }

    /// 删除指定 url 的直播间信息
    pub async fn delete(url: &str) -> Result<()> {
        let sql = r#"
            DELETE FROM live_room_info WHERE url = ?
        "#;
        sqlx::query(sql).bind(url).execute(&pool()).await?;
        Ok(())
    }
}
