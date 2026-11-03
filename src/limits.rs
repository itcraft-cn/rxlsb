//! 行列上限（Excel 规范）与溢出策略。
//!
//! Excel XLSB 的硬上限为 **16384 列**、**1048576 行**。当写入请求超过上限时，
//! 行为由全局策略决定：
//!
//! - [`OverflowBehavior::Error`]：返回 [`XlsbError::LimitExceeded`]；
//! - [`OverflowBehavior::Cut`]：静默截断到上限。
//!
//! 列与行**各自独立配置**；[`set_overflow`] 可一次性设置两者。
//! 所有配置是**全局**的，可在任意时刻修改，但每次写入操作在**开始时快照**当前策略，
//! 因此修改只影响其后的操作。按维度“后设置覆盖先设置”。
//!
//! ```
//! use rxlsb::limits::{set_overflow, OverflowBehavior};
//!
//! // 列超限截断、行超限报错
//! set_overflow(OverflowBehavior::Cut, OverflowBehavior::Error);
//! ```
//!
//! 内部通过 [`OverflowPolicy`] trait 实现策略模式：全局配置只保存枚举值，
//! 操作开始时据此选择对应的策略 `impl`。

use std::sync::atomic::{AtomicU8, Ordering};

use crate::error::{Result, XlsbError};

/// Excel 规范最大列数（XFD）。
pub const MAX_COLUMNS: usize = 16_384;
/// Excel 规范最大行数。
pub const MAX_ROWS: usize = 1_048_576;

/// 超出上限时的行为。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum OverflowBehavior {
    /// 返回错误。
    Error = 0,
    /// 截断到上限。
    Cut = 1,
}

/// 溢出处理策略（策略模式）。
///
/// 每次写入操作开始时根据全局配置选择具体 `impl`，并使用该策略处理本次操作。
pub trait OverflowPolicy: Send + Sync {
    /// 解析列数：返回本次实际使用的列数。
    fn columns(&self, requested: usize) -> Result<usize>;

    /// 解析行区间：返回本次实际使用的行数（截断时可能小于 `requested`）。
    fn rows(&self, start_row: usize, requested: usize) -> Result<usize>;
}

/// 超限即报错的策略。
pub struct ErrorPolicy;

impl OverflowPolicy for ErrorPolicy {
    fn columns(&self, requested: usize) -> Result<usize> {
        if requested > MAX_COLUMNS {
            return Err(XlsbError::LimitExceeded {
                dimension: "columns",
                requested,
                limit: MAX_COLUMNS,
            });
        }
        Ok(requested)
    }

    fn rows(&self, start_row: usize, requested: usize) -> Result<usize> {
        if start_row.saturating_add(requested) > MAX_ROWS {
            return Err(XlsbError::LimitExceeded {
                dimension: "rows",
                requested: start_row.saturating_add(requested),
                limit: MAX_ROWS,
            });
        }
        Ok(requested)
    }
}

/// 超限截断的策略。
pub struct CutPolicy;

impl OverflowPolicy for CutPolicy {
    fn columns(&self, requested: usize) -> Result<usize> {
        Ok(requested.min(MAX_COLUMNS))
    }

    fn rows(&self, start_row: usize, requested: usize) -> Result<usize> {
        if start_row >= MAX_ROWS {
            return Ok(0);
        }
        Ok(requested.min(MAX_ROWS - start_row))
    }
}

static ERROR_POLICY: ErrorPolicy = ErrorPolicy;
static CUT_POLICY: CutPolicy = CutPolicy;

static COLUMN_BEHAVIOR: AtomicU8 = AtomicU8::new(OverflowBehavior::Error as u8);
static ROW_BEHAVIOR: AtomicU8 = AtomicU8::new(OverflowBehavior::Error as u8);

fn decode(raw: u8) -> OverflowBehavior {
    if raw == OverflowBehavior::Cut as u8 {
        OverflowBehavior::Cut
    } else {
        OverflowBehavior::Error
    }
}

/// 设置列溢出的行为（覆盖之前的列设置）。
pub fn set_column_overflow(behavior: OverflowBehavior) {
    COLUMN_BEHAVIOR.store(behavior as u8, Ordering::Relaxed);
}

/// 设置行溢出的行为（覆盖之前的行设置）。
pub fn set_row_overflow(behavior: OverflowBehavior) {
    ROW_BEHAVIOR.store(behavior as u8, Ordering::Relaxed);
}

/// 一次性设置列与行的行为（覆盖两者）。
pub fn set_overflow(columns: OverflowBehavior, rows: OverflowBehavior) {
    set_column_overflow(columns);
    set_row_overflow(rows);
}

/// 读取当前列溢出行为。
pub fn column_overflow() -> OverflowBehavior {
    decode(COLUMN_BEHAVIOR.load(Ordering::Relaxed))
}

/// 读取当前行溢出行为。
pub fn row_overflow() -> OverflowBehavior {
    decode(ROW_BEHAVIOR.load(Ordering::Relaxed))
}

#[inline]
fn policy_of(behavior: OverflowBehavior) -> &'static dyn OverflowPolicy {
    match behavior {
        OverflowBehavior::Error => &ERROR_POLICY,
        OverflowBehavior::Cut => &CUT_POLICY,
    }
}

/// 当前列策略快照。
#[inline]
pub(crate) fn column_policy() -> &'static dyn OverflowPolicy {
    policy_of(column_overflow())
}

/// 当前行策略快照。
#[inline]
pub(crate) fn row_policy() -> &'static dyn OverflowPolicy {
    policy_of(row_overflow())
}

/// 底层维度限制快照：在所有写入路径（批量/流式/模板）开始时创建一次。
///
/// 创建时解析列数并快照行策略，之后的写入都通过本对象判断，保证“只对设置后的行为生效”。
#[derive(Clone, Copy)]
pub(crate) struct SheetLimits {
    columns: usize,
    row_policy: &'static dyn OverflowPolicy,
}

impl SheetLimits {
    /// 开始一次 sheet 写入：解析起始列数并快照行策略。
    pub(crate) fn begin(requested_columns: usize) -> Result<Self> {
        Ok(Self {
            columns: column_policy().columns(requested_columns)?,
            row_policy: row_policy(),
        })
    }

    /// 本次实际使用的列数（可能被截断）。
    #[inline]
    pub(crate) fn columns(&self) -> usize {
        self.columns
    }

    /// 解析行区间，返回本次实际使用的行数（可能被截断）。
    #[inline]
    pub(crate) fn rows(&self, start_row: usize, requested: usize) -> Result<usize> {
        self.row_policy.rows(start_row, requested)
    }
}
