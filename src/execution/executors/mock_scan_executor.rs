//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// mock_scan_executor.rs
//
// Identification: src/execution/mock_scan_executor.rs
//
// Copyright (c) 2015-2022, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use crate::catalog::{Column, Schema};
use crate::common::rid::RID;
use crate::execution::executor_context::ExecutorContext;
use crate::execution::executors::Executor;
use crate::execution::plans::MockScanPlanNode;
use crate::sql_type::{TypeId, Value};
use crate::storage::table::tuple::Tuple;
use rand::seq::SliceRandom;
use rand::thread_rng;

// ---------------------------------------------------------------------------
// TA usernames and office hour schedules for different semesters
// ---------------------------------------------------------------------------

static TA_LIST_2022: &[&str] = &[
    "amstqq", "durovo", "joyceliaoo", "karthik-ramanathan-3006",
    "kush789", "lmwnshn", "mkpjnx", "skyzh",
    "thepinetree", "timlee0119", "yliang412",
];

static TA_LIST_2023: &[&str] = &[
    "abigalekim", "arvinwu168", "christopherlim98", "David-Lyons",
    "fanyuex2", "Mayank-Baranwal", "skyzh", "yarkhinephyo",
    "yliang412",
];

static TA_LIST_2023_FALL: &[&str] = &[
    "skyzh", "yliang412", "fernandolis10", "wiam8", "anurag-23",
    "Mayank-Baranwal", "abigalekim", "ChaosZhai", "aoleizhou",
    "averyqi115", "kswim8",
];

static TA_LIST_2024: &[&str] = &[
    "AlSchlo", "walkingcabbages", "averyqi115", "lanlou1554",
    "sweetsuro", "ChaosZhai", "SDTheSlayer", "xx01cyx",
    "yliang412", "thelongmarch-azx",
];

static TA_LIST_2024_FALL: &[&str] = &[
    "17zhangw", "connortsui20", "J-HowHuang", "lanlou1554",
    "prashanthduvvada", "unw9527", "xx01cyx", "yashkothari42",
];

static TA_OH_2022: &[&str] = &[
    "Tuesday", "Wednesday", "Monday", "Wednesday", "Thursday",
    "Friday", "Wednesday", "Randomly", "Tuesday", "Monday",
    "Tuesday",
];

static TA_OH_2023: &[&str] = &[
    "Friday", "Thursday", "Tuesday", "Monday", "Tuesday",
    "Tuesday", "Randomly", "Wednesday", "Thursday",
];

static TA_OH_2023_FALL: &[&str] = &[
    "Randomly", "Tuesday", "Wednesday", "Tuesday", "Thursday",
    "Tuesday", "Friday", "Yesterday", "Friday", "Friday",
    "Never",
];

static TA_OH_2024: &[&str] = &[
    "Friday", "Thursday", "Friday", "Wednesday", "Thursday",
    "Yesterday", "Monday", "Tuesday", "Tuesday", "Monday",
];

static TA_OH_2024_FALL: &[&str] = &[
    "Wednesday", "Thursday", "Tuesday", "Monday", "Friday",
    "Thursday", "Tuesday", "Friday",
];

static COURSE_ON_DATE: &[&str] = &[
    "Monday", "Tuesday", "Wednesday", "Thursday", "Friday",
    "Saturday", "Sunday",
];

/// List of all mock table names.
pub static MOCK_TABLE_LIST: &[&str] = &[
    "__mock_table_1",
    "__mock_table_2",
    "__mock_table_3",
    "__mock_table_tas_2022",
    "__mock_table_tas_2023",
    "__mock_table_tas_2023_fall",
    "__mock_table_tas_2024",
    "__mock_table_tas_2024_fall",
    "__mock_agg_input_small",
    "__mock_agg_input_big",
    "__mock_external_merge_sort_input",
    "__mock_table_schedule_2022",
    "__mock_table_schedule",
    "__mock_table_123",
    "__mock_graph",
    // For leaderboard Q1
    "__mock_t1",
    // For leaderboard Q2
    "__mock_t4_1m",
    "__mock_t5_1m",
    "__mock_t6_1m",
    // For leaderboard Q3
    "__mock_t7",
    "__mock_t8",
    "__mock_t9",
    // For P3 leaderboard Q4
    "__mock_t10",
    "__mock_t11",
];

const GRAPH_NODE_CNT: usize = 10;

// ---------------------------------------------------------------------------
// Helper utilities
// ---------------------------------------------------------------------------

/// Repeat a string `s` `n` times.
fn string_repeat(s: &str, n: usize) -> String {
    std::iter::repeat(s).take(n).collect::<String>()
}

// ---------------------------------------------------------------------------
// Schema retrieval
// ---------------------------------------------------------------------------

/// Return the schema for the given mock table name.
pub fn get_mock_table_schema_of(table: &str) -> Result<Schema, String> {
    match table {
        "__mock_table_1" => Ok(Schema::new(vec![
            Column::new("colA", TypeId::Integer),
            Column::new("colB", TypeId::Integer),
        ])),
        "__mock_table_2" => Ok(Schema::new(vec![
            Column::new_with_length("colC", TypeId::Varchar, 128),
            Column::new_with_length("colD", TypeId::Varchar, 128),
        ])),
        "__mock_table_3" => Ok(Schema::new(vec![
            Column::new("colE", TypeId::Integer),
            Column::new_with_length("colF", TypeId::Varchar, 128),
        ])),
        "__mock_table_tas_2022" | "__mock_table_tas_2023"
        | "__mock_table_tas_2023_fall" | "__mock_table_tas_2024"
        | "__mock_table_tas_2024_fall" => Ok(Schema::new(vec![
            Column::new_with_length("github_id", TypeId::Varchar, 128),
            Column::new_with_length("office_hour", TypeId::Varchar, 128),
        ])),
        "__mock_table_schedule_2022" | "__mock_table_schedule" => Ok(Schema::new(vec![
            Column::new_with_length("day_of_week", TypeId::Varchar, 128),
            Column::new("has_lecture", TypeId::Integer),
        ])),
        "__mock_agg_input_small" | "__mock_agg_input_big" => Ok(Schema::new(vec![
            Column::new("v1", TypeId::Integer),
            Column::new("v2", TypeId::Integer),
            Column::new("v3", TypeId::Integer),
            Column::new("v4", TypeId::Integer),
            Column::new("v5", TypeId::Integer),
            Column::new_with_length("v6", TypeId::Varchar, 128),
        ])),
        "__mock_external_merge_sort_input" => Ok(Schema::new(vec![
            Column::new("v1", TypeId::Integer),
            Column::new("v2", TypeId::Integer),
            Column::new("v3", TypeId::Integer),
        ])),
        "__mock_graph" => Ok(Schema::new(vec![
            Column::new("src", TypeId::Integer),
            Column::new("dst", TypeId::Integer),
            Column::new("src_label", TypeId::Integer),
            Column::new("dst_label", TypeId::Integer),
            Column::new("distance", TypeId::Integer),
        ])),
        "__mock_table_123" => Ok(Schema::new(vec![
            Column::new("number", TypeId::Integer),
        ])),
        "__mock_t4_1m" | "__mock_t5_1m" | "__mock_t6_1m" => Ok(Schema::new(vec![
            Column::new("x", TypeId::Integer),
            Column::new("y", TypeId::Integer),
        ])),
        "__mock_t1" => Ok(Schema::new(vec![
            Column::new("x", TypeId::Integer),
            Column::new("y", TypeId::Integer),
            Column::new("z", TypeId::Integer),
        ])),
        "__mock_t7" => Ok(Schema::new(vec![
            Column::new("v", TypeId::Integer),
            Column::new("v1", TypeId::Integer),
            Column::new("v2", TypeId::Integer),
        ])),
        "__mock_t8" => Ok(Schema::new(vec![
            Column::new("v4", TypeId::Integer),
        ])),
        "__mock_t9" => Ok(Schema::new(vec![
            Column::new("x", TypeId::Integer),
            Column::new("y", TypeId::Integer),
        ])),
        "__mock_t10" | "__mock_t11" => Ok(Schema::new(vec![
            Column::new("x", TypeId::Integer),
            Column::new("y", TypeId::Integer),
        ])),
        _ => Err(format!("mock table {} not found", table)),
    }
}

// ---------------------------------------------------------------------------
// Table size retrieval
// ---------------------------------------------------------------------------

/// Return the number of rows for the given mock table plan.
fn get_size_of(plan: &MockScanPlanNode) -> usize {
    let table = plan.table.as_str();
    match table {
        "__mock_table_1" | "__mock_table_2" | "__mock_table_3" => 100,
        "__mock_table_tas_2022" => TA_LIST_2022.len(),
        "__mock_table_tas_2023" => TA_LIST_2023.len(),
        "__mock_table_tas_2023_fall" => TA_LIST_2023_FALL.len(),
        "__mock_table_tas_2024" => TA_LIST_2024.len(),
        "__mock_table_tas_2024_fall" => TA_LIST_2024_FALL.len(),
        "__mock_table_schedule_2022" | "__mock_table_schedule" => COURSE_ON_DATE.len(),
        "__mock_agg_input_small" => 1000,
        "__mock_agg_input_big" => 10_000,
        "__mock_external_merge_sort_input" => 100_000,
        "__mock_graph" => GRAPH_NODE_CNT * GRAPH_NODE_CNT,
        "__mock_table_123" => 3,
        "__mock_t1" => 1_000_000,
        "__mock_t4_1m" | "__mock_t5_1m" | "__mock_t6_1m" => 1_000_000,
        "__mock_t7" => 1_000_000,
        "__mock_t8" => 10,
        "__mock_t9" => 10_000_000,
        "__mock_t10" => 10_000,   // 10k
        "__mock_t11" => 1_000_000, // 1M
        _ => 0,
    }
}

// ---------------------------------------------------------------------------
// Shuffle flag
// ---------------------------------------------------------------------------

/// Return whether the output of the given mock table plan should be shuffled.
fn get_shuffled(plan: &MockScanPlanNode) -> bool {
    let table = plan.table.as_str();
    matches!(table, "__mock_t1" | "__mock_t2_100k" | "__mock_t3_1k")
}

// ---------------------------------------------------------------------------
// Tuple generation function
// ---------------------------------------------------------------------------

/// Return a closure that generates tuples for the given mock table plan.
fn get_function_of<'a>(plan: &'a MockScanPlanNode) -> Box<dyn Fn(usize) -> Tuple + 'a> {
    let table = plan.table.as_str();
    match table {
        "__mock_table_1" => {
            Box::new(move |cursor: usize| {
                let values = vec![
                    Value::from_i32(cursor as i32),
                    Value::from_i32((cursor * 100) as i32),
                ];
                Tuple::new_with_values(values, plan.output_schema_ref())
            })
        }
        "__mock_table_2" => {
            Box::new(move |cursor: usize| {
                let values = vec![
                    Value::from_str(format!("{}-💩", cursor).as_str()),
                    Value::from_str(string_repeat("😇", cursor % 8).as_str()),
                ];
                Tuple::new_with_values(values, plan.output_schema_ref())
            })
        }
        "__mock_table_3" => {
            Box::new(move |cursor: usize| {
                let values = if cursor % 2 == 0 {
                    vec![
                        Value::from_i32(cursor as i32),
                        Value::from_str(format!("{}-💩", cursor).as_str()),
                    ]
                } else {
                    vec![
                        Value::null(TypeId::Integer),
                        Value::from_str(format!("{}-💩", cursor).as_str()),
                    ]
                };
                Tuple::new_with_values(values, plan.output_schema_ref())
            })
        }
        "__mock_table_tas_2022" => {
            Box::new(move |cursor: usize| {
                let values = vec![
                    Value::from_str(TA_LIST_2022[cursor]),
                    Value::from_str(TA_OH_2022[cursor]),
                ];
                Tuple::new_with_values(values, plan.output_schema_ref())
            })
        }
        "__mock_table_tas_2023" => {
            Box::new(move |cursor: usize| {
                let values = vec![
                    Value::from_str(TA_LIST_2023[cursor]),
                    Value::from_str(TA_OH_2023[cursor]),
                ];
                Tuple::new_with_values(values, plan.output_schema_ref())
            })
        }
        "__mock_table_tas_2023_fall" => {
            Box::new(move |cursor: usize| {
                let values = vec![
                    Value::from_str(TA_LIST_2023_FALL[cursor]),
                    Value::from_str(TA_OH_2023_FALL[cursor]),
                ];
                Tuple::new_with_values(values, plan.output_schema_ref())
            })
        }
        "__mock_table_tas_2024" => {
            Box::new(move |cursor: usize| {
                let values = vec![
                    Value::from_str(TA_LIST_2024[cursor]),
                    Value::from_str(TA_OH_2024[cursor]),
                ];
                Tuple::new_with_values(values, plan.output_schema_ref())
            })
        }
        "__mock_table_tas_2024_fall" => {
            Box::new(move |cursor: usize| {
                let values = vec![
                    Value::from_str(TA_LIST_2024_FALL[cursor]),
                    Value::from_str(TA_OH_2024_FALL[cursor]),
                ];
                Tuple::new_with_values(values, plan.output_schema_ref())
            })
        }
        "__mock_table_schedule_2022" => {
            Box::new(move |cursor: usize| {
                let values = vec![
                    Value::from_str(COURSE_ON_DATE[cursor]),
                    Value::from_i32(if cursor == 1 || cursor == 3 { 1 } else { 0 }),
                ];
                Tuple::new_with_values(values, plan.output_schema_ref())
            })
        }
        "__mock_table_schedule" => {
            Box::new(move |cursor: usize| {
                let values = vec![
                    Value::from_str(COURSE_ON_DATE[cursor]),
                    Value::from_i32(if cursor == 0 || cursor == 2 { 1 } else { 0 }),
                ];
                Tuple::new_with_values(values, plan.output_schema_ref())
            })
        }
        "__mock_agg_input_small" => {
            Box::new(move |cursor: usize| {
                let values = vec![
                    Value::from_i32(((cursor + 2) % 10) as i32),
                    Value::from_i32(cursor as i32),
                    Value::from_i32(((cursor + 50) % 100) as i32),
                    Value::from_i32((cursor / 100) as i32),
                    Value::from_i32(233),
                    Value::from_str(string_repeat("💩", (cursor % 8) + 1).as_str()),
                ];
                Tuple::new_with_values(values, plan.output_schema_ref())
            })
        }
        "__mock_agg_input_big" => {
            Box::new(move |cursor: usize| {
                let values = vec![
                    Value::from_i32(((cursor + 2) % 10) as i32),
                    Value::from_i32(cursor as i32),
                    Value::from_i32(((cursor + 50) % 100) as i32),
                    Value::from_i32((cursor / 1000) as i32),
                    Value::from_i32(233),
                    Value::from_str(string_repeat("💩", (cursor % 16) + 1).as_str()),
                ];
                Tuple::new_with_values(values, plan.output_schema_ref())
            })
        }
        "__mock_external_merge_sort_input" => {
            Box::new(move |cursor: usize| {
                let values = vec![
                    Value::from_i32(cursor as i32),
                    Value::from_i32(((cursor + 1777) % 15000) as i32),
                    Value::from_i32(((cursor + 3) % 111) as i32),
                ];
                Tuple::new_with_values(values, plan.output_schema_ref())
            })
        }
        "__mock_table_123" => {
            Box::new(move |cursor: usize| {
                let values = vec![Value::from_i32((cursor + 1) as i32)];
                Tuple::new_with_values(values, plan.output_schema_ref())
            })
        }
        "__mock_graph" => {
            Box::new(move |cursor: usize| {
                let src = cursor % GRAPH_NODE_CNT;
                let dst = cursor / GRAPH_NODE_CNT;
                let values = vec![
                    Value::from_i32(src as i32),
                    Value::from_i32(dst as i32),
                    // Note(f24): Use INTEGER for `src_label` and `dst_label` based on
                    // current external merge sort limitation (only supports fixed-length data).
                    Value::from_i32((src * 100) as i32),
                    Value::from_i32((dst * 100) as i32),
                    if src == dst {
                        Value::null(TypeId::Integer)
                    } else {
                        Value::from_i32(1)
                    },
                ];
                Tuple::new_with_values(values, plan.output_schema_ref())
            })
        }
        "__mock_t1" => {
            Box::new(move |cursor: usize| {
                let values = vec![
                    Value::from_i32((cursor / 10000) as i32),
                    Value::from_i32((cursor % 10000) as i32),
                    Value::from_i32(cursor as i32),
                ];
                Tuple::new_with_values(values, plan.output_schema_ref())
            })
        }
        "__mock_t4_1m" => {
            Box::new(move |cursor: usize| {
                let c = cursor % 500000;
                let values = vec![
                    Value::from_i32(c as i32),
                    Value::from_i32((c * 10) as i32),
                ];
                Tuple::new_with_values(values, plan.output_schema_ref())
            })
        }
        "__mock_t5_1m" => {
            Box::new(move |cursor: usize| {
                let c = (cursor + 30000) % 500000;
                let values = vec![
                    Value::from_i32(c as i32),
                    Value::from_i32((c * 10) as i32),
                ];
                Tuple::new_with_values(values, plan.output_schema_ref())
            })
        }
        "__mock_t6_1m" => {
            Box::new(move |cursor: usize| {
                let c = (cursor + 60000) % 500000;
                let values = vec![
                    Value::from_i32(c as i32),
                    Value::from_i32((c * 10) as i32),
                ];
                Tuple::new_with_values(values, plan.output_schema_ref())
            })
        }
        "__mock_t7" => {
            Box::new(move |cursor: usize| {
                let values = vec![
                    Value::from_i32((cursor % 20) as i32),
                    Value::from_i32(cursor as i32),
                    Value::from_i32(cursor as i32),
                ];
                Tuple::new_with_values(values, plan.output_schema_ref())
            })
        }
        "__mock_t8" => {
            Box::new(move |cursor: usize| {
                let values = vec![Value::from_i32(cursor as i32)];
                Tuple::new_with_values(values, plan.output_schema_ref())
            })
        }
        "__mock_t9" => {
            Box::new(move |cursor: usize| {
                let y = 10000000i32
                    - (cursor as i32 / 2
                        + ((cursor / 10000) % 2) as i32 * ((cursor as i32 / 2) % 2));
                let values = vec![
                    Value::from_i32((cursor / 10000) as i32),
                    Value::from_i32(y),
                ];
                Tuple::new_with_values(values, plan.output_schema_ref())
            })
        }
        "__mock_t10" => {
            Box::new(move |cursor: usize| {
                let values = vec![
                    Value::from_i32(cursor as i32),
                    Value::from_i32((cursor * 10) as i32),
                ];
                Tuple::new_with_values(values, plan.output_schema_ref())
            })
        }
        "__mock_t11" => {
            Box::new(move |cursor: usize| {
                let values = vec![
                    Value::from_i32(-1 * (cursor as i32 % 1000) - 1),
                    Value::from_i32((cursor * 10) as i32),
                ];
                Tuple::new_with_values(values, plan.output_schema_ref())
            })
        }
        // By default, return a tuple of all zeros matching the output schema.
        _ => Box::new(move |_cursor: usize| {
            let values: Vec<Value> = plan
                .output_schema_ref()
                .get_columns()
                .iter()
                .map(|col| Value::zero(col.get_type()))
                .collect();
            Tuple::new_with_values(values, plan.output_schema_ref())
        }),
    }
}

// ---------------------------------------------------------------------------
// MockScanExecutor
// ---------------------------------------------------------------------------

/// The MockScanExecutor executor executes a sequential table scan for tests.
pub struct MockScanExecutor<'a> {
    /// The plan node for the scan.
    plan: &'a MockScanPlanNode,
    exec_ctx: &'a ExecutorContext,
    /// The cursor for the current mock scan position.
    cursor: usize,
    /// The tuple generation function for the mock table.
    func: Box<dyn Fn(usize) -> Tuple + 'a>,
    /// The total number of rows in the mock table.
    size: usize,
    /// Shuffled output indices (empty if no shuffling is needed).
    shuffled_idx: Vec<usize>,
}

impl<'a> Executor for MockScanExecutor<'a> {
    fn output_schema_ref(&self) -> &Schema {
        &self.plan.output_schema
    }

    fn executor_context(&self) -> &ExecutorContext {
        self.exec_ctx
    }

    fn next(&mut self, batch_size: usize) -> Option<(Vec<Tuple>, Vec<RID>)> {
        let mut tuple_batch = Vec::new();
        let mut rid_batch = Vec::new();

        while tuple_batch.len() < batch_size && self.cursor < self.size {
            let tuple = match self.shuffled_idx.is_empty() {
                true => (*self.func)(self.cursor),
                false => (*self.func)(self.shuffled_idx[self.cursor])
            };
            let rid = Self::make_dummy_rid();
            self.cursor += 1;
            tuple_batch.push(tuple);
            rid_batch.push(rid);
        }
        match tuple_batch.is_empty() {
            true => None,
            false => Some((tuple_batch, rid_batch))
        }
    }

    fn name(&self) -> &str { "mock scan" }
}

impl<'a> MockScanExecutor<'a> {
    /// Construct a new MockScanExecutor instance.
    pub fn new(exec_ctx: &'a ExecutorContext, plan: &'a MockScanPlanNode) -> Self {
        let size = get_size_of(plan);
        let shuffled_idx = if get_shuffled(plan) {
            let mut indices: Vec<usize> = (0..size).collect();
            indices.shuffle(&mut thread_rng());
            indices
        } else {
            Vec::new()
        };

        MockScanExecutor {
            plan,
            exec_ctx,
            cursor: 0,
            func: get_function_of(plan),
            size,
            shuffled_idx,
        }
    }
    /// Create a dummy RID value.
    fn make_dummy_rid() -> RID {
        RID::from_i64(0)
    }
}
