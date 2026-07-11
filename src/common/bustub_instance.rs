use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use super::result_writer::ResultWriter;
use crate::binder::{
    BoundStatement, CreateStatement, ExplainOptions, ExplainStatement, IndexStatement,
    TransactionStatement, TransactionStatementType, VariableSetStatement, VariableShowStatement,
};
use crate::catalog::{Catalog, IndexType, Schema};
use crate::common::{BUFFER_POOL_SIZE, LRUK_REPLACER_K};
use crate::concurrency::transaction::IsolationLevel;
use crate::sql_type::TypeId;
use crate::storage::disk::disk_scheduler::DiskManager;
use crate::storage::disk::hard_disk_manager::HardDiskManager;
use crate::{
    binder::Binder,
    buffer::buffer_pool_manager::BufferPoolManager,
    catalog::CatalogRef,
    common::errors::BustubError,
    concurrency::{LockManager, Transaction, TransactionManager},
    execution::{
        execution_engine::ExecutionEngine,
        executor_context::ExecutorContext,
        executors::mock_scan_executor::{get_mock_table_schema_of, MOCK_TABLE_LIST},
    },
    optimizer::Optimizer,
    planner::Planner,
};

#[allow(dead_code)]
pub struct BustubInstance {
    pub(crate) bpm: Arc<BufferPoolManager>,
    pub(crate) lock_manager: Arc<LockManager>,
    pub(crate) txn_manager: TransactionManager,
    pub(crate) catalog: CatalogRef,
    curr_txn: Option<Arc<Transaction>>,
    managed_txn_mode: bool,
}

impl BustubInstance {
    /// Create a new `BustubInstance` with the given database file path.
    ///
    /// This initializes an in-memory disk manager, a buffer pool manager,
    /// a lock manager, a transaction manager, and a catalog.
    pub fn new(db_file: &str) -> Result<Self, BustubError> {
        // Create the in-memory disk manager (shared between the struct and BPM).
        let disk_manager = Arc::new(HardDiskManager::new(db_file).map_err(|e| format!("{}", e))?)
            as Arc<dyn DiskManager>;

        // Initialize the buffer pool manager.
        let bpm = Arc::new(BufferPoolManager::new(
            BUFFER_POOL_SIZE,
            disk_manager,
            LRUK_REPLACER_K,
        ));

        // Initialize the catalog with the buffer pool manager.
        let catalog = CatalogRef::new(Catalog::new(bpm.clone()));

        Ok(BustubInstance {
            bpm: bpm,
            lock_manager: Arc::new(LockManager {}),
            txn_manager: TransactionManager::new(),
            catalog,
            curr_txn: None,
            managed_txn_mode: false,
        })
    }

    pub fn enable_managed_txn_mode(&mut self) {
        self.managed_txn_mode = true;
    }

    pub fn execute_sql<W: ResultWriter>(
        &mut self,
        sql: &str,
        writer: &mut W,
    ) -> Result<bool, BustubError> {
        if let Some('\\') = sql.chars().next() {
            return match sql {
                "\\dt" => {
                    self.cmd_display_tables(writer);
                    Ok(true)
                }
                "\\di" => {
                    self.cmd_display_indices(writer);
                    Ok(true)
                }
                "\\help" => {
                    self.cmd_display_help(writer);
                    Ok(true)
                }
                sql => {
                    if sql.starts_with("\\dbgmvcc") {
                        let params = sql.split(' ').collect::<Vec<_>>();
                        self.cmd_dbg_mvcc(params, writer);
                        Ok(true)
                    } else if sql.starts_with("\\txn") {
                        let params = sql.split(' ').collect::<Vec<_>>();
                        self.cmd_txn(params, writer);
                        Ok(true)
                    } else {
                        Err(BustubError::Message(format!(
                            "unsupported internal command: {}",
                            sql
                        )))
                    }
                }
            };
        }

        let (txn, is_local_txn) = match self.curr_txn.take() {
            Some(t) => (t, false),
            None => (self.txn_manager.new_txn()?, true),
        };
        match self.execute_sql_txn(sql, writer, Some(&txn)) {
            Ok(x) => {
                if is_local_txn {
                    self.txn_manager.commit_txn(&txn)?;
                } else {
                    self.curr_txn = Some(txn);
                }
                Ok(x)
            }
            Err(e) => {
                self.txn_manager.abort_txn(&txn)?;
                Err(e)
            }
        }
    }

    pub fn execute_sql_txn<W: ResultWriter>(
        &mut self,
        sql: &str,
        writer: &mut W,
        txn: Option<&Transaction>,
    ) -> Result<bool, BustubError> {
        let exec_engine = ExecutionEngine {};
        let mut binder = Binder::new(&self.catalog);
        let dialect = sqlparser::dialect::GenericDialect {};
        let statements = sqlparser::parser::Parser::parse_sql(&dialect, sql)
            .map_err(|e| format!("parse error: {:?}", e))?;

        for sql_stmt in statements.iter() {
            let stmt = binder
                .bind_statement(sql_stmt)
                .map_err(|e| format!("{:?}", e))?;

            let mut is_delete = false;
            match &stmt {
                BoundStatement::Create(create) => { self.handle_create_statement(txn, create, writer)?; continue; },
                BoundStatement::Index(index) => { self.handle_index_statement(txn, index, writer)?; continue; },
                BoundStatement::VariableSet(vs) => { self.handle_variable_set_statement(txn, vs, writer)?; continue; },
                BoundStatement::VariableShow(vs) => { self.handle_variable_show_statement(txn, vs, writer)?; continue; },
                BoundStatement::Explain(explain) => { self.handle_explain_statement(txn, explain, writer)?; continue; },
                BoundStatement::Transaction(txn_stmt) => { self.handle_txn_statement(txn, txn_stmt, writer)?; continue; },
                BoundStatement::Update(_) | BoundStatement::Delete(_) => { is_delete = true; },
                _ => {}
            }

            let mut planner = Planner::new(self.catalog.clone());
            let plan = planner.plan_query(&stmt).expect("");

            let optimizer = Optimizer::new(&self.catalog, false);
            let plan = optimizer.optimize(plan);

            let exec_ctx = ExecutorContext {
                txn: self.curr_txn.clone(),
                bpm: self.bpm.clone(),
                catalog: self.catalog.clone(),
                lock_mgr: self.lock_manager.clone(),
                is_delete,
            };
            let result_set = exec_engine.execute(&plan, &exec_ctx)?;
            let output_schema = plan.output_schema_ref();

            let col_names = output_schema
                .columns
                .iter()
                .map(|c| c.get_name().to_owned())
                .collect::<Vec<_>>();
            writer.add_header_row(col_names.as_slice());
            for tuple in result_set.iter() {
                let tuple_values = (0..output_schema.columns.len())
                    .map(|idx| tuple.get_value(&output_schema, idx).to_string())
                    .collect::<Vec<_>>();
                writer.add_row(tuple_values.as_slice());
            }
        }

        Ok(false)
    }

    fn cmd_display_tables<W: ResultWriter>(&self, writer: &mut W) {
        let tables = self.catalog.get_tables_info();
        writer.add_header_row(&["oid", "name", "cols"]);
        for table in tables {
            writer.add_row(&[
                &table.oid.to_string(),
                &table.name,
                &table.schema.to_string(),
            ]);
        }
    }

    fn cmd_display_indices<W: ResultWriter>(&self, writer: &mut W) {
        let table_names = self.catalog.get_table_names();
        writer.add_header_row(&["table_name", "index_oid", "index_name", "index_cols"]);
        for table in table_names {
            let indices = self.catalog.get_table_indexes(&table);
            for index in indices {
                writer.add_row(&[
                    &index.table_name,
                    &index.index_oid.to_string(),
                    &index.name,
                    &index.key_schema.to_string(),
                ]);
            }
        }
    }

    fn cmd_display_help<W: ResultWriter>(&self, writer: &mut W) {
        let help = r"Welcome to the BusTub shell!

\dt: show all tables
\di: show all indices
\dbgmvcc <table>: show version chain of a table
\help: show this message again
\txn: show current txn information
\txn <txn_id>: switch to txn
\txn gc: run garbage collection
\txn -1: exit txn mode

BusTub shell currently only supports a small set of Postgres queries. We'll set
up a doc describing the current status later. It will silently ignore some parts
of the query, so it's normal that you'll get a wrong result when executing
unsupported SQL queries. This shell will be able to run `create table` only
after you have completed the buffer pool manager. It will be able to execute SQL
queries after you have implemented necessary query executors. Use `explain` to
see the execution plan of your query.";
        writer.add_row(&[help.to_owned()]);
    }

    fn cmd_dbg_mvcc<W: ResultWriter>(&self, _params: Vec<&str>, _writer: &mut W) {
        todo!("cmd_dbg_mvcc")
    }

    fn cmd_txn<W: ResultWriter>(&self, _params: Vec<&str>, _writer: &mut W) {
        todo!("cmd_dbg_txn")
    }
}

impl BustubInstance {
    pub fn create_mock_table(&self) -> Result<(), BustubError> {
        for &table_name in MOCK_TABLE_LIST {
            let schema = get_mock_table_schema_of(table_name)?;
            if let None = self.catalog.create_table(table_name, schema, false) {
                return Err(BustubError::Message(format!("table {} exists", table_name)));
            }
        }
        Ok(())
    }

    pub fn current_managed_txn(&self) -> Option<&Transaction> {
        self.curr_txn.as_deref()
    }
}

/// DDL
impl BustubInstance {
    /// Global session variable storage.
    /// Uses a static OnceLock with a Mutex to provide interior mutability
    /// without requiring modifications to the struct definition.
    fn get_session_variable(key: &str) -> String {
        static VARS: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
        let vars = VARS.get_or_init(|| Mutex::new(HashMap::new()));
        let guard = vars.lock().unwrap();
        guard.get(key).cloned().unwrap_or_default()
    }

    /// Set a session variable value.
    fn set_session_variable(key: &str, value: &str) {
        static VARS: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
        let vars = VARS.get_or_init(|| Mutex::new(HashMap::new()));
        let mut guard = vars.lock().unwrap();
        guard.insert(key.to_string(), value.to_string());
    }

    /// Check if the optimizer should force starter rules.
    fn is_force_starter_rule() -> bool {
        let value = Self::get_session_variable("force_optimizer_starter_rule");
        value == "1" || value == "true" || value == "yes"
    }

    fn handle_create_statement<W: ResultWriter>(
        &self,
        txn: Option<&Transaction>,
        create: &CreateStatement,
        writer: &mut W,
    ) -> Result<(), BustubError> {
        let schema = Schema::new(create.columns.clone());
        let table_info = self
            .catalog
            .create_table(&create.table, schema, true)
            .ok_or_else(|| BustubError::Message(format!("table {} exists.", create.table)))?;
        let schema = &table_info.schema;

        let index = match create.primary_key.is_empty() {
            true => None,
            false => {
                let mut index_col_ids = Vec::new();
                for col in create.primary_key.iter() {
                    let idx = schema.get_col_idx(&col);
                    if schema.get_column(idx as usize).column_type != TypeId::Integer {
                        return Err(BustubError::Message(
                            "only support creating index on integer column.".to_string(),
                        ));
                    }
                    index_col_ids.push(idx as usize);
                }

                if index_col_ids.is_empty() {
                    return Err(BustubError::Message(
                        "primary key should not be empty.".to_string(),
                    ));
                }

                let key_schema = Schema::copy_schema(schema, &index_col_ids);

                let key_size = index_col_ids.len() * 4;

                let key_size = [4usize, 8, 16, 32, 64]
                    .into_iter()
                    .find(|x| key_size <= *x)
                    .ok_or_else(|| {
                        BustubError::Message("unsupported: primary key size exceeds 64 bytes".to_string())
                    })?;

                let index = self
                    .catalog
                    .create_index(
                        txn,
                        format!("{}_pk", create.table).as_str(),
                        &create.table,
                        schema,
                        key_schema,
                        &index_col_ids,
                        key_size,
                        true,
                        IndexType::BPlusTreeIndex,
                    )
                    .ok_or_else(|| BustubError::Message(format!("index {}_pk exists", create.table)))?;

                Some(index)
            }
        };


        match &index {
            Some(index) => writer.add_row(&[format!("Table created with oid = {}, primary key index created with id = {}.", table_info.oid, index.index_oid)]),
            None => writer.add_row(&[format!("Table created with oid = {}.", table_info.oid)])
        };

        Ok(())
    }

    /// Handle CREATE INDEX statement.
    fn handle_index_statement<W: ResultWriter>(
        &self,
        txn: Option<&Transaction>,
        stmt: &IndexStatement,
        writer: &mut W,
    ) -> Result<(), BustubError> {
        // Collect column IDs from the index column references.
        let mut col_ids = Vec::new();
        for col in &stmt.cols {
            let col_name = col
                .col_names
                .last()
                .ok_or_else(|| BustubError::Message("empty column name".to_string()))?;
            let idx = stmt.table.schema.get_col_idx(col_name);
            // Only integer columns are supported for indexing.
            if stmt.table.schema.get_column(idx as usize).column_type != TypeId::Integer {
                return Err(BustubError::Message(
                    "only support creating index on integer column".to_string(),
                ));
            }
            col_ids.push(idx as usize);
        }

        if col_ids.is_empty() {
            return Err(BustubError::Message(
                "index columns should not be empty".to_string(),
            ));
        }

        // Construct the key schema from the selected index columns.
        let key_schema = Schema::copy_schema(&stmt.table.schema, &col_ids);

        // Compute the index key size, rounding up to the nearest supported size.
        let raw_key_size = col_ids.len() * 4;
        let key_size = [4usize, 8, 16, 32, 64]
            .into_iter()
            .find(|x| raw_key_size <= *x)
            .ok_or_else(|| {
                BustubError::Message(
                    "unsupported: index key size exceeds 64 bytes".to_string(),
                )
            })?;

        // Create the index via the catalog.
        let index = self
            .catalog
            .create_index(
                txn,
                &stmt.index_name,
                &stmt.table.table,
                &stmt.table.schema,
                key_schema,
                &col_ids,
                key_size,
                false,
                stmt.index_type,
            )
            .ok_or_else(|| {
                BustubError::Message(format!(
                    "failed to create index {}",
                    stmt.index_name
                ))
            })?;

        writer.add_row(&[format!(
            "Index created with oid = {} with type = {}",
            index.index_oid, index.index_type
        )]);

        Ok(())
    }

    /// Handle EXPLAIN statement.
    fn handle_explain_statement<W: ResultWriter>(
        &self,
        _txn: Option<&Transaction>,
        stmt: &ExplainStatement,
        writer: &mut W,
    ) -> Result<(), BustubError> {
        let mut output = String::new();

        // Print binder result.
        if (stmt.options & ExplainOptions::Binder as u8) != 0 {
            output.push_str("=== BINDER ===\n");
            output.push_str(&stmt.statement.to_string());
            output.push('\n');
        }

        // Plan the inner statement.
        let mut planner = Planner::new(self.catalog.clone());
        let plan = planner
            .plan_query(&stmt.statement)
            .map_err(|e| BustubError::Message(format!("plan error: {:?}", e)))?;

        let show_schema = (stmt.options & ExplainOptions::Schema as u8) != 0;

        // Print planner result.
        if (stmt.options & ExplainOptions::Planner as u8) != 0 {
            output.push_str("=== PLANNER ===\n");
            output.push_str(&plan.to_string());
            if show_schema {
                output.push_str(&format!("\nschema: {}\n", plan.output_schema()));
            }
            output.push('\n');
        }

        // Optimize the plan.
        let optimizer = Optimizer::new(&self.catalog, Self::is_force_starter_rule());
        let optimized_plan = optimizer.optimize(plan.clone());

        // Print optimizer result.
        if (stmt.options & ExplainOptions::Optimizer as u8) != 0 {
            output.push_str("=== OPTIMIZER ===\n");
            output.push_str(&optimized_plan.to_string());
            if show_schema {
                output.push_str(&format!(
                    "\nschema: {}\n",
                    optimized_plan.output_schema()
                ));
            }
            output.push('\n');
        }

        writer.add_row(&[output]);

        Ok(())
    }

    /// Handle SHOW variable statement.
    fn handle_variable_show_statement<W: ResultWriter>(
        &self,
        _txn: Option<&Transaction>,
        stmt: &VariableShowStatement,
        writer: &mut W,
    ) -> Result<(), BustubError> {
        let content = Self::get_session_variable(&stmt.variable);
        writer.add_row(&[format!("{}={}", stmt.variable, content)]);
        Ok(())
    }

    /// Handle SET variable statement.
    fn handle_variable_set_statement<W: ResultWriter>(
        &self,
        _txn: Option<&Transaction>,
        stmt: &VariableSetStatement,
        _writer: &mut W,
    ) -> Result<(), BustubError> {
        Self::set_session_variable(&stmt.variable, &stmt.value);
        Ok(())
    }

    /// Handle transaction control statement (BEGIN / COMMIT / ROLLBACK).
    fn handle_txn_statement<W: ResultWriter>(
        &self,
        txn: Option<&Transaction>,
        stmt: &TransactionStatement,
        writer: &mut W,
    ) -> Result<(), BustubError> {
        match stmt.txn_type {
            TransactionStatementType::Begin => {
                if !self.managed_txn_mode {
                    writer.add_row(&[{
                        "begin statement is only supported in managed txn mode, \
                         please use bustub-shell"
                            .to_string()
                    }]);
                    return Ok(());
                }

                let txn_activated = self.curr_txn.is_some();
                let iso_level_str = Self::get_session_variable("global_isolation_level");
                let new_txn = if iso_level_str == "serializable" {
                    self.txn_manager
                        .new_txn_with_iso_level(IsolationLevel::Serializable)?
                } else {
                    // Default to snapshot isolation.
                    self.txn_manager
                        .new_txn_with_iso_level(IsolationLevel::SnapshotIsolation)?
                };

                let prefix = if txn_activated {
                    "pause current txn and begin new txn "
                } else {
                    "begin txn "
                };
                let msg = format!(
                    "{}txn_id={} txn_real_id={} read_ts={} commit_ts={} status={:?} iso_lvl={:?}",
                    prefix,
                    new_txn.get_transaction_id_human_readable(),
                    new_txn.get_transaction_id(),
                    new_txn.get_read_ts(),
                    new_txn.get_commit_ts(),
                    new_txn.get_state(),
                    new_txn.get_isolation_level(),
                );
                writer.add_row(&[msg]);
                Ok(())
            }
            TransactionStatementType::Commit => {
                let txn = match txn {
                    Some(t) => t,
                    None => {
                        writer.add_row(&["commit / rollback can only be used with txn".to_string()]);
                        return Ok(());
                    }
                };
                match self.txn_manager.commit_txn(txn) {
                    Ok(()) => {
                        writer.add_row(&[format!(
                            "txn committed, txn_id={}, status={:?}, read_ts={}, commit_ts={}",
                            txn.get_transaction_id_human_readable(),
                            txn.get_state(),
                            txn.get_read_ts(),
                            txn.get_commit_ts(),
                        )]);
                    }
                    Err(_) => {
                        writer.add_row(&[format!(
                            "txn failed to commit, txn_id={}, status={:?}, read_ts={}, commit_ts={}",
                            txn.get_transaction_id_human_readable(),
                            txn.get_state(),
                            txn.get_read_ts(),
                            txn.get_commit_ts(),
                        )]);
                    }
                }
                Ok(())
            }
            TransactionStatementType::Rollback => {
                let txn = match txn {
                    Some(t) => t,
                    None => {
                        writer.add_row(&["commit / rollback can only be used with txn".to_string()]);
                        return Ok(());
                    }
                };
                self.txn_manager.abort_txn(txn)?;
                writer.add_row(&[format!(
                    "txn aborted, txn_id={}, status={:?}, read_ts={}",
                    txn.get_transaction_id_human_readable(),
                    txn.get_state(),
                    txn.get_read_ts(),
                )]);
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{errors::BustubError, result_writer::NoopWriter};

    #[test]
    fn bustub_debug() -> Result<(), BustubError> {
        let sql = r"select * from __mock_t4_1m;";
        let mut writer = NoopWriter {};
        let mut bustub = BustubInstance::new("bustub.db")?;
        bustub.create_mock_table()?;
        bustub.execute_sql(sql, &mut writer)?;
        Ok(())
    }
}
