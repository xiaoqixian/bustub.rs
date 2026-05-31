use std::{collections::HashMap, sync::Mutex};

use crate::{common::SlotOffset, concurrency::transaction::UndoLink};

#[allow(dead_code)]
pub(crate) struct PageVersionInfo {
    prev_link: Mutex<HashMap<SlotOffset, UndoLink>>
}

pub struct TransanctionManager {}
