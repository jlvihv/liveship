use crate::config::config_dir;
use crate::model::AppConfig;
use crate::model::LiveInfo;
use crate::model::QueryHistory;
use crate::model::RecordingHistory;
use crate::model::RecordingPlan;
use anyhow::Result;
use chrono::Utc;
use once_cell::sync::OnceCell;
use redb::{Database, ReadableTable, TableDefinition, WriteTransaction};

static INSTANCE: OnceCell<Database> = OnceCell::new();
const TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("liveship");

pub fn init() -> Result<()> {
    let path = config_dir()?.join("liveship.db");
    let db = Database::create(path)?;
    let write_txn = db.begin_write()?;
    {
        // 通过这样，在没有这个表的时候自动创建表
        let _table = write_txn.open_table(TABLE)?;
    }
    write_txn.commit()?;
    INSTANCE.set(db).unwrap();
    Ok(())
}

fn db() -> &'static Database {
    INSTANCE.get().unwrap()
}

pub mod config {
    use super::*;

    pub fn get() -> Result<AppConfig> {
        let read_txn = db().begin_read()?;
        let table = read_txn.open_table(TABLE)?;
        match table.get("config") {
            Ok(Some(config)) => Ok(serde_json::from_slice(&config.value())?),
            _ => Ok(AppConfig::default()),
        }
    }

    pub fn set(config: &AppConfig) -> Result<()> {
        let write_txn = db().begin_write()?;
        {
            let mut table = write_txn.open_table(TABLE)?;
            let config = serde_json::to_vec(config)?;
            table.insert("config", &*config)?;
        }
        write_txn.commit()?;
        Ok(())
    }
}

pub mod plan {
    use super::*;

    /// 打一个标记，记录轮询检查时间
    pub fn mark_polling_time() -> Result<()> {
        let write_txn = db().begin_write()?;
        {
            let mut table = write_txn.open_table(TABLE)?;
            let now = Utc::now().timestamp_millis();
            table.insert("polling_time", now.to_be_bytes().as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// 获取上次轮询检查时间
    pub fn get_last_polling_time() -> Result<i64> {
        let read_txn = db().begin_read()?;
        let table = read_txn.open_table(TABLE)?;
        let result = table.get("polling_time")?;
        match result {
            Some(time) => {
                let time = i64::from_be_bytes(time.value().try_into()?);
                Ok(time)
            }
            None => Ok(0),
        }
    }

    /// 添加一个录制计划
    pub fn add(plan: &RecordingPlan) -> Result<()> {
        let key = format!("plan:{}", plan.url);
        let plan = serde_json::to_vec(plan)?;
        let write_txn = db().begin_write()?;
        {
            let mut table = write_txn.open_table(TABLE)?;
            table.insert(key.as_str(), &*plan)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// 获取所有录制计划
    pub fn get_all() -> Result<Vec<RecordingPlan>> {
        let read_txn = db().begin_read()?;
        let table = read_txn.open_table(TABLE)?;
        let iter = table.range("plan:".."planz")?;
        let mut plans = Vec::new();
        for kv in iter {
            let (_, plan) = kv?;
            let mut plan: RecordingPlan = serde_json::from_slice(&plan.value())?;
            // 获取直播间信息
            if plan.live_info.is_none() {
                plan.live_info = super::live::get(&plan.url)?;
            }
            plans.push(plan);
        }
        sort(&mut plans);
        Ok(plans)
    }

    /// 获取一个录制计划
    pub fn get(url: String) -> Result<Option<RecordingPlan>> {
        let read_txn = db().begin_read()?;
        let table = read_txn.open_table(TABLE)?;
        let result = table.get(format!("plan:{}", url).as_str())?;
        match result {
            Some(plan) => {
                let plan: RecordingPlan = serde_json::from_slice(&plan.value())?;
                Ok(Some(plan))
            }
            None => Ok(None),
        }
    }

    /// 获取所有启用的录制计划
    pub fn get_enabled() -> Result<Vec<RecordingPlan>> {
        let read_txn = db().begin_read()?;
        let table = read_txn.open_table(TABLE)?;
        let iter = table.range("plan:".."planz")?;
        let mut plans = Vec::new();
        for kv in iter {
            let (_, plan) = kv?;
            let mut plan: RecordingPlan = serde_json::from_slice(&plan.value())?;
            if plan.enabled {
                // 获取直播间信息
                if plan.live_info.is_none() {
                    plan.live_info = super::live::get(&plan.url)?;
                }
                plans.push(plan);
            }
        }
        sort(&mut plans);
        Ok(plans)
    }

    /// 删除一个录制计划，即使不存在也不会报错
    pub fn delete(url: &str) -> Result<()> {
        let key = format!("plan:{}", url);
        let write_txn = db().begin_write()?;
        {
            let mut table = write_txn.open_table(TABLE)?;
            table.remove(key.as_str())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// 更新录制计划的状态
    pub fn update_status(url: &str, enabled: bool) -> Result<()> {
        let key = format!("plan:{}", url);
        let write_txn = db().begin_write()?;
        {
            let mut table = write_txn.open_table(TABLE)?;
            let plan_vec;
            if let Some(result) = table.get(key.as_str())? {
                let mut plan: RecordingPlan = serde_json::from_slice(&result.value())?;
                plan.enabled = enabled;
                let plan = serde_json::to_vec(&plan)?;
                plan_vec = plan;
            } else {
                return Err(anyhow::anyhow!("plan not found"));
            }
            table.insert(key.as_str(), &*plan_vec)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// 对录制计划进行排序，按照 created_at 降序
    fn sort(plans: &mut Vec<RecordingPlan>) {
        plans.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    }
}

/// 这个模块用来标记正在录制中的 url
pub mod recording {
    use super::*;

    /// 添加一个正在录制中的 url，会检查是否此 url 已存在，如果已存在，则返回错误
    /// 确保一个 url 只有一个正在录制中的记录
    /// 值是开始录制的时间
    pub(super) fn add(write_txn: &WriteTransaction, url: &str, start_time: i64) -> Result<()> {
        let key = format!("recording:{}", url);
        let mut table = write_txn.open_table(TABLE)?;
        if table.get(key.as_str())?.is_some() {
            return Err(anyhow::anyhow!("url is recording"));
        }
        table.insert(key.as_str(), start_time.to_be_bytes().as_slice())?;
        Ok(())
    }

    /// 删除一个正在录制中的 url，返回开始录制的时间
    pub(super) fn delete(write_txn: &WriteTransaction, url: &str) -> Result<i64> {
        let key = format!("recording:{}", url);
        let mut table = write_txn.open_table(TABLE)?;
        let result = table.remove(key.as_str())?;
        match result {
            Some(start_time) => {
                let start_time = i64::from_be_bytes(start_time.value().try_into()?);
                Ok(start_time)
            }
            None => Err(anyhow::anyhow!("url is not recording")),
        }
    }

    pub fn is_exist(url: &str) -> Result<bool> {
        let key = format!("recording:{}", url);
        let read_txn = db().begin_read()?;
        let table = read_txn.open_table(TABLE)?;
        let value = table.get(key.as_str())?;
        Ok(value.is_some())
    }
}

pub mod history {
    use super::*;

    /// 增加一条录制历史
    pub fn add(history: &RecordingHistory) -> Result<()> {
        let write_txn = db().begin_write()?;
        // 先增加正在录制中的标记，如果这个 url 已经存在，会直接报错
        recording::add(&write_txn, &history.url, history.start_time)?;
        {
            let mut table = write_txn.open_table(TABLE)?;
            // 然后增加历史记录
            let key = format!("history:{}:{}", history.url, history.start_time);
            let history = serde_json::to_vec(history)?;
            table.insert(key.as_str(), &*history)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// 获取所有录制历史
    pub fn get_all() -> Result<Vec<RecordingHistory>> {
        let mut histories = Vec::new();
        let read_txn = db().begin_read()?;
        let table = read_txn.open_table(TABLE)?;
        let iter = table.range("history:".."historyz")?;
        for kv in iter {
            let (_, history) = kv?;
            let mut history: RecordingHistory = serde_json::from_slice(&history.value())?;
            // 获取直播间信息
            if history.live_info.is_none() {
                history.live_info = super::live::get(&history.url).map_err(|e| {
                    eprintln!("get live info error: {:?}", e);
                    e
                })?;
            }
            histories.push(history);
        }
        sort(&mut histories);
        Ok(histories)
    }

    /// 获取一条录制历史
    pub fn get(url: &str, start_time: i64) -> Result<RecordingHistory> {
        let key = format!("history:{}:{}", url, start_time);
        let read_txn = db().begin_read()?;
        let table = read_txn.open_table(TABLE)?;
        let result = table.get(key.as_str())?;
        match result {
            Some(history) => {
                let history: RecordingHistory = serde_json::from_slice(&history.value())?;
                Ok(history)
            }
            None => Err(anyhow::anyhow!("history not found")),
        }
    }

    /// 删除一条录制历史
    pub fn delete(url: &str, start_time: i64, delete_file: bool) -> Result<()> {
        if delete_file {
            let history = get(url, start_time)?;
            std::fs::remove_file(history.path)?;
        }
        let key = format!("history:{}:{}", url, start_time);
        let write_txn = db().begin_write()?;
        {
            let mut table = write_txn.open_table(TABLE)?;
            table.remove(key.as_str())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// 把一条录制历史的状态设置为终止，将更新 end_time 和 status，同时将正在录制中的标记删除
    pub fn end(url: &str) -> Result<()> {
        let write_txn = db().begin_write()?;
        let start_time = recording::delete(&write_txn, url)?;
        let key = format!("history:{}:{}", url, start_time);
        {
            let mut table = write_txn.open_table(TABLE)?;
            let history_vec;
            if let Some(history) = table.get(key.as_str())? {
                let mut history: RecordingHistory = serde_json::from_slice(&history.value())?;
                history.end_time = Utc::now().timestamp_millis();
                history.status = crate::model::RecordStatus::NotRecording;
                let history = serde_json::to_vec(&history)?;
                history_vec = history;
            } else {
                return Err(anyhow::anyhow!("history not found"));
            };
            table.insert(key.as_str(), &*history_vec)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// 排序，按照 start_time 降序
    fn sort(histories: &mut Vec<RecordingHistory>) {
        histories.sort_by(|a, b| b.start_time.cmp(&a.start_time));
    }
}

pub mod live {

    use super::*;

    /// 添加一条直播信息
    pub fn add(live: &LiveInfo) -> Result<()> {
        let write_txn = db().begin_write()?;
        {
            let mut table = write_txn.open_table(TABLE)?;
            let key = format!("live:{}", live.url);
            let live = serde_json::to_vec(live)?;
            table.insert(key.as_str(), &*live)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// 删除一条直播信息
    pub fn delete(url: &str) -> Result<()> {
        let write_txn = db().begin_write()?;
        {
            let mut table = write_txn.open_table(TABLE)?;
            table.remove(format!("live:{}", url).as_str())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// 获取所有直播信息
    pub fn get_all() -> Result<Vec<LiveInfo>> {
        let mut lives = Vec::new();
        let read_txn = db().begin_read()?;
        let table = read_txn.open_table(TABLE)?;
        let iter = table.range("live:".."livez")?;
        for kv in iter {
            let (_, live) = kv?;
            let live: LiveInfo = serde_json::from_slice(&live.value())?;
            lives.push(live);
        }
        Ok(lives)
    }

    /// 获取一条直播信息
    pub fn get(url: &str) -> Result<Option<LiveInfo>> {
        let read_txn = db().begin_read()?;
        let table = read_txn.open_table(TABLE)?;
        let value = table.get(format!("live:{}", url).as_str())?;
        match value {
            Some(value) => {
                let live: LiveInfo = serde_json::from_slice(&value.value())?;
                Ok(Some(live))
            }
            None => Ok(None),
        }
    }
}

pub mod query_history {
    use std::collections::VecDeque;

    use super::*;

    pub fn add(history: &QueryHistory) -> Result<()> {
        let write_txn = db().begin_write()?;
        {
            let mut table = write_txn.open_table(TABLE)?;
            let now = chrono::Utc::now().timestamp_millis();
            let key = format!("query_history:{}:{}", history.url, now);
            let history = serde_json::to_vec(history)?;
            table.insert(key.as_str(), &*history)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn get_all() -> Result<Vec<QueryHistory>> {
        let mut histories = VecDeque::new();
        let read_txn = db().begin_read()?;
        let table = read_txn.open_table(TABLE)?;
        let iter = table.range("query_history:".."query_history")?;
        for kv in iter {
            let (_, history) = kv?;
            let history: QueryHistory = serde_json::from_slice(&history.value())?;
            histories.push_front(history);
        }
        Ok(histories.into())
    }
}

mod tests {
    #[test]
    fn test_kv() {
        super::init().unwrap();
    }
}
