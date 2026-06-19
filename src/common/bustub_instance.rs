use std::sync::Arc;

use crate::{
    binder::Binder, buffer::buffer_pool_manager::BufferPoolManager, catalog::Catalog, common::errors::BustubError, concurrency::{LockManager, Transaction, TransactionManager}, execution::{execution_engine::ExecutionEngine, executor_context::ExecutorContext, mock_scan_executor::{MOCK_TABLE_LIST, get_mock_table_schema_of}}, optimizer::Optimizer, planner::Planner
};
use super::result_writer::ResultWriter;

#[allow(dead_code)]
pub struct BustubInstance {
    pub(crate) bpm: Option<Arc<BufferPoolManager>>,
    pub(crate) lock_manager: LockManager,
    pub(crate) txn_manager: TransactionManager,
    pub(crate) catalog: Catalog,
    curr_txn: Option<Arc<Transaction>>,
    managed_txn_mode: bool,
}

impl BustubInstance {
    /// Create a new `BustubInstance` with the given database file path.
    ///
    /// This initializes an in-memory disk manager, a buffer pool manager,
    /// a lock manager, a transaction manager, and a catalog.
    pub fn new(db_file: &str) -> Result<Self, BustubError> {
        use crate::buffer::buffer_pool_manager::BufferPoolManager;
        use crate::catalog::Catalog;
        use crate::common::{BUFFER_POOL_SIZE, LRUK_REPLACER_K};
        use crate::concurrency::LockManager;
        use crate::concurrency::TransactionManager;
        use crate::storage::disk::hard_disk_manager::HardDiskManager;
        use crate::storage::disk::disk_scheduler::DiskManager;

        // Create the in-memory disk manager (shared between the struct and BPM).
        let disk_manager = Arc::new({
            HardDiskManager::new(db_file)
                .map_err(|e| format!("{}", e))?
        }) as Arc<dyn DiskManager>;

        // Initialize the buffer pool manager.
        let bpm = Arc::new(BufferPoolManager::new(
            BUFFER_POOL_SIZE,
            disk_manager,
            LRUK_REPLACER_K,
        ));

        // Initialize the catalog with the buffer pool manager.
        let catalog = Catalog::new(bpm.clone());

        Ok(BustubInstance {
            bpm: Some(bpm),
            lock_manager: LockManager {},
            txn_manager: TransactionManager::new(),
            catalog,
            curr_txn: None,
            managed_txn_mode: false,
        })
    }

    pub fn enable_managed_txn_mode(&mut self) {
        self.managed_txn_mode = true;
    }

    pub fn execute_sql<W: ResultWriter>(&mut self, sql: &str, writer: &mut W) -> Result<bool, BustubError> {
        if let Some('\\') = sql.chars().next() {
            return match sql {
                "\\dt" => {
                    self.cmd_display_tables(writer);
                    Ok(true)
                },
                "\\di" => {
                    self.cmd_display_indices(writer);
                    Ok(true)
                },
                "\\help" => {
                    self.cmd_display_help(writer);
                    Ok(true)
                },
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
                        Err(BustubError::Message(format!("unsupported internal command: {}", sql)))
                    }
                }
            };
        }

        let (txn, is_local_txn) = match self.curr_txn.take() {
            Some(t) => (t, false),
            None => (self.txn_manager.new_txn()?, true)
        };
        match self.execute_sql_txn(sql, writer, Some(&txn)) {
            Ok(x) => {
                if is_local_txn {
                    self.txn_manager.commit_txn(&txn)?;
                } else {
                    self.curr_txn = Some(txn);
                }
                Ok(x)
            },
            Err(e) => {
                self.txn_manager.abort_txn(&txn)?;
                Err(e)
            }
        }
    }

    pub fn execute_sql_txn<W: ResultWriter>(&mut self, sql: &str, writer: &mut W, _txn: Option<&Transaction>) -> Result<bool, BustubError> {
        let exec_engine = ExecutionEngine {};
        let mut binder = Binder::new(&self.catalog);
        let dialect = sqlparser::dialect::GenericDialect {};
        let statements = sqlparser::parser::Parser::parse_sql(&dialect, sql)
            .map_err(|e| format!("parse error: {:?}", e))?;
        
        for sql_stmt in statements.iter() {
            let stmt = binder.bind_statement(sql_stmt).map_err(|e| format!("{:?}", e))?;
            
            match stmt {
                _ => {}
            }

            let planner = Planner::new(&self.catalog);
            let plan = planner.plan_query(&stmt).expect("");

            let optimizer = Optimizer::new(&self.catalog, false);
            let plan = optimizer.optimize(&plan);

            let exec_ctx = ExecutorContext {};
            let result_set = exec_engine.execute(&plan, &exec_ctx)?;
            let output_schema = plan.output_schema_ref();
            
            let col_names = output_schema.columns.iter().map(|c| c.get_name().to_owned()).collect::<Vec<_>>();
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
            writer.add_row(&[&table.oid.to_string(), &table.name, &table.schema.to_string()]);
        }
    }

    fn cmd_display_indices<W: ResultWriter>(&self, _writer: &mut W) {
        todo!("cmd_display_indices")
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
            if let None = self.catalog.create_table(table_name, &schema, false) {
                return Err(BustubError::Message(format!("table {} exists", table_name)));
            }
        }
        Ok(())
    }

    pub fn current_managed_txn(&self) -> Option<&Transaction> {
        self.curr_txn.as_deref()
    }
}
