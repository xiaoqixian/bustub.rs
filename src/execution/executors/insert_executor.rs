// Date:   Fri Jun 26 15:36:39 2026
// Mail:   lunar_ubuntu@qq.com
// Author: https://github.com/xiaoqixian
//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// insert_executor.rs
//
// Identification: src/include/execution/executors/insert_executor.h
//
// Copyright (c) 2015-2025, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use crate::{
    catalog::Schema,
    common::rid::RID,
    execution::{
        executor_context::ExecutorContext,
        executors::Executor,
        plans::InsertPlanNode,
    },
    storage::table::tuple::{Tuple, TupleMeta},
};

/**
 * InsertExecutor executes an insert on a table.
 * Inserted values are always pulled from a child executor.
 */
pub struct InsertExecutor<'a> {
    exec_ctx: &'a ExecutorContext,
    plan: &'a InsertPlanNode,
    #[allow(dead_code)]
    child_executor: Option<Box<dyn Executor + 'a>>,
}

impl<'a> InsertExecutor<'a> {
    pub fn new(
        exec_ctx: &'a ExecutorContext,
        plan: &'a InsertPlanNode,
        child_executor: Box<dyn Executor + 'a>,
    ) -> Self {
        Self {
            exec_ctx,
            plan,
            child_executor: Some(child_executor),
        }
    }
}

impl<'a> Executor for InsertExecutor<'a> {
    fn next(&mut self, batch_size: usize) -> Option<(Vec<Tuple>, Vec<RID>)> {
        let child_executor = self.child_executor.as_mut()?;

        let txn = self.exec_ctx.txn.as_ref().expect("expect a txn in executor").as_ref();
        let txn_id = txn.get_transaction_id();

        let table_info = self.exec_ctx.catalog.get_table_by_oid(self.plan.table_oid)
            .expect("table not found should not happen");
        let table = &table_info.table;

        let mut row_count = 0usize;

        let meta = TupleMeta { ts: txn_id, is_deleted: false };

        let table_indices = self.exec_ctx.catalog.get_table_indexes(&table_info.name);
        
        while let Some((tuples, _)) = child_executor.next(batch_size) {
            let child_out_schema = child_executor.output_schema_ref();
            row_count += tuples.len();

            for tp in &tuples {
                let rid = table.insert_tuple(&meta, tp, Some(self.exec_ctx.lock_mgr.as_ref()), Some(txn), table_info.oid).unwrap();
                
                txn.append_write_set(table_info.oid, rid);

                // update indices
                for index_info in &table_indices {
                    let index = index_info.index.as_ref();
                    let key_tuple = Tuple::from_key(tp, child_out_schema, index.get_key_schema(), index.get_key_attrs());
                    
                    index.insert_entry(&key_tuple, rid, Some(txn));
                }
            }
        }

        self.child_executor = None;
        Some((vec![Tuple::from_num(row_count as i32)], vec![]))
    }

    fn output_schema_ref(&self) -> &Schema {
        &self.plan.output_schema
    }

    fn executor_context(&self) -> &ExecutorContext {
        self.exec_ctx
    }
}


