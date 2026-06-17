use std::sync::{Arc, Mutex};

use crate::{
    binder::{Binder, BoundStatement}, buffer::buffer_pool_manager::BufferPoolManager, catalog::Catalog, concurrency::{LockManager, Transaction, TransactionManager}, execution::{execution_engine::ExecutionEngine, executor_context::ExecutorContext}, optimizer::Optimizer, planner::Planner, storage::disk::disk_scheduler::DiskManager
};
use super::result_writer::ResultWriter;

pub struct BustubInstance {
    pub(crate) disk_maanger: Box<dyn DiskManager>,
    pub(crate) bpm: Arc<BufferPoolManager>,
    pub(crate) lock_manager: LockManager,
    pub(crate) txn_manager: TransactionManager,
    pub(crate) catalog: Catalog,
    curr_txn: Option<Transaction>,
}

impl BustubInstance {
    pub fn execute_sql<W: ResultWriter>(&mut self, sql: &str, writer: &mut W) -> Result<bool, String> {
        let (mut txn, is_local_txn) = match self.curr_txn.take() {
            Some(t) => (t, false),
            None => (self.txn_manager.new_txn(), true)
        };
        match self.execute_sql_txn(sql, writer, Some(&mut txn)) {
            Ok(x) => {
                if is_local_txn {
                    self.txn_manager.commit_txn(&txn);
                } else {
                    self.curr_txn = Some(txn);
                }
                Ok(x)
            },
            Err(e) => {
                self.txn_manager.abort_txn(&txn);
                Err(e)
            }
        }
    }

    pub fn execute_sql_txn<W: ResultWriter>(&mut self, sql: &str, writer: &mut W, _txn: Option<&mut Transaction>) -> Result<bool, String> {
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
                        Err(format!("unsupported internal command: {}", sql))
                    }
                }
            };
        }

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

    fn cmd_display_tables<W: ResultWriter>(&self, _writer: &mut W) {
        todo!("cmd_display_tables")
    }

    fn cmd_display_indices<W: ResultWriter>(&self, _writer: &mut W) {
        todo!("cmd_display_indices")
    }

    fn cmd_display_help<W: ResultWriter>(&self, _writer: &mut W) {
        todo!("cmd_display_help")
    }

    fn cmd_dbg_mvcc<W: ResultWriter>(&self, _params: Vec<&str>, _writer: &mut W) {
        todo!("cmd_dbg_mvcc")
    }

    fn cmd_txn<W: ResultWriter>(&self, _params: Vec<&str>, _writer: &mut W) {
        todo!("cmd_dbg_txn")
    }
}
