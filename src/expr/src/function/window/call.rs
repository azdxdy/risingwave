// Copyright 2023 RisingWave Labs
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::cmp::Ordering;
use std::fmt::Display;

use risingwave_common::bail;
use risingwave_common::types::DataType;
use risingwave_pb::expr::window_frame::{PbBound, PbExclusion};
use risingwave_pb::expr::{PbWindowFrame, PbWindowFunction};

use super::WindowFuncKind;
use crate::agg::AggArgs;
use crate::Result;

#[derive(Debug, Clone)]
pub struct WindowFuncCall {
    pub kind: WindowFuncKind,
    pub args: AggArgs,
    pub return_type: DataType,
    pub frame: Frame,
}

impl WindowFuncCall {
    pub fn from_protobuf(call: &PbWindowFunction) -> Result<Self> {
        let call = WindowFuncCall {
            kind: WindowFuncKind::from_protobuf(call.get_type()?)?,
            args: AggArgs::from_protobuf(call.get_args())?,
            return_type: DataType::from(call.get_return_type()?),
            frame: Frame::from_protobuf(call.get_frame()?)?,
        };
        Ok(call)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Frame {
    pub bounds: FrameBounds,
    pub exclusion: FrameExclusion,
}

impl Display for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.bounds)?;
        if self.exclusion != FrameExclusion::default() {
            write!(f, " {}", self.exclusion)?;
        }
        Ok(())
    }
}

impl Frame {
    pub fn rows(start: FrameBound<usize>, end: FrameBound<usize>) -> Self {
        Self {
            bounds: FrameBounds::Rows(start, end),
            exclusion: FrameExclusion::default(),
        }
    }

    pub fn rows_with_exclusion(
        start: FrameBound<usize>,
        end: FrameBound<usize>,
        exclusion: FrameExclusion,
    ) -> Self {
        Self {
            bounds: FrameBounds::Rows(start, end),
            exclusion,
        }
    }
}

impl Frame {
    pub fn from_protobuf(frame: &PbWindowFrame) -> Result<Self> {
        use risingwave_pb::expr::window_frame::PbType;
        let bounds = match frame.get_type()? {
            PbType::Unspecified => bail!("unspecified type of `WindowFrame`"),
            PbType::Rows => {
                let start = FrameBound::from_protobuf(frame.get_start()?)?;
                let end = FrameBound::from_protobuf(frame.get_end()?)?;
                FrameBounds::Rows(start, end)
            }
        };
        let exclusion = FrameExclusion::from_protobuf(frame.get_exclusion()?)?;
        Ok(Self { bounds, exclusion })
    }

    pub fn to_protobuf(&self) -> PbWindowFrame {
        use risingwave_pb::expr::window_frame::PbType;
        let exclusion = self.exclusion.to_protobuf() as _;
        match &self.bounds {
            FrameBounds::Rows(start, end) => PbWindowFrame {
                r#type: PbType::Rows as _,
                start: Some(start.to_protobuf()),
                end: Some(end.to_protobuf()),
                exclusion,
            },
        }
    }
}

impl FrameBounds {
    pub fn is_valid(&self) -> bool {
        match self {
            Self::Rows(start, end) => start.partial_cmp(end).map(|o| o.is_le()).unwrap_or(false),
        }
    }

    pub fn start_is_unbounded(&self) -> bool {
        match self {
            Self::Rows(start, _) => matches!(start, FrameBound::UnboundedPreceding),
        }
    }

    pub fn end_is_unbounded(&self) -> bool {
        match self {
            Self::Rows(_, end) => matches!(end, FrameBound::UnboundedFollowing),
        }
    }
}

impl Display for FrameBounds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rows(start, end) => {
                write!(f, "ROWS BETWEEN {} AND {}", start, end)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum FrameBounds {
    Rows(FrameBound<usize>, FrameBound<usize>),
    // Groups(FrameBound<usize>, FrameBound<usize>),
    // Range(FrameBound<ScalarImpl>, FrameBound<ScalarImpl>),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum FrameBound<T> {
    UnboundedPreceding,
    Preceding(T),
    CurrentRow,
    Following(T),
    UnboundedFollowing,
}

impl<T: Ord> PartialOrd for FrameBound<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use FrameBound::*;
        match (self, other) {
            (UnboundedPreceding, UnboundedPreceding) => None,
            (UnboundedPreceding, _) => Some(Ordering::Less),
            (_, UnboundedPreceding) => Some(Ordering::Greater),

            (UnboundedFollowing, UnboundedFollowing) => None,
            (UnboundedFollowing, _) => Some(Ordering::Greater),
            (_, UnboundedFollowing) => Some(Ordering::Less),

            (CurrentRow, CurrentRow) => Some(Ordering::Equal),

            // it's ok to think preceding(0) < current row here
            (Preceding(_), CurrentRow) => Some(Ordering::Less),
            (CurrentRow, Preceding(_)) => Some(Ordering::Greater),

            // it's ok to think following(0) > current row here
            (Following(_), CurrentRow) => Some(Ordering::Greater),
            (CurrentRow, Following(_)) => Some(Ordering::Less),

            (Preceding(n1), Preceding(n2)) => n2.partial_cmp(n1),
            (Following(n1), Following(n2)) => n1.partial_cmp(n2),
            (Preceding(_), Following(_)) => Some(Ordering::Less),
            (Following(_), Preceding(_)) => Some(Ordering::Greater),
        }
    }
}

impl FrameBound<usize> {
    pub fn from_protobuf(bound: &PbBound) -> Result<Self> {
        use risingwave_pb::expr::window_frame::bound::PbOffset;
        use risingwave_pb::expr::window_frame::PbBoundType;

        let offset = bound.get_offset()?;
        let bound = match offset {
            PbOffset::Integer(offset) => match bound.get_type()? {
                PbBoundType::Unspecified => bail!("unspecified type of `FrameBound<usize>`"),
                PbBoundType::UnboundedPreceding => Self::UnboundedPreceding,
                PbBoundType::Preceding => Self::Preceding(*offset as usize),
                PbBoundType::CurrentRow => Self::CurrentRow,
                PbBoundType::Following => Self::Following(*offset as usize),
                PbBoundType::UnboundedFollowing => Self::UnboundedFollowing,
            },
            PbOffset::Datum(_) => bail!("offset of `FrameBound<usize>` must be `Integer`"),
        };
        Ok(bound)
    }

    pub fn to_protobuf(&self) -> PbBound {
        use risingwave_pb::expr::window_frame::bound::PbOffset;
        use risingwave_pb::expr::window_frame::PbBoundType;

        let (r#type, offset) = match self {
            Self::UnboundedPreceding => (PbBoundType::UnboundedPreceding, PbOffset::Integer(0)),
            Self::Preceding(offset) => (PbBoundType::Preceding, PbOffset::Integer(*offset as _)),
            Self::CurrentRow => (PbBoundType::CurrentRow, PbOffset::Integer(0)),
            Self::Following(offset) => (PbBoundType::Following, PbOffset::Integer(*offset as _)),
            Self::UnboundedFollowing => (PbBoundType::UnboundedFollowing, PbOffset::Integer(0)),
        };
        PbBound {
            r#type: r#type as _,
            offset: Some(offset),
        }
    }
}

impl Display for FrameBound<usize> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FrameBound::UnboundedPreceding => write!(f, "UNBOUNDED PRECEDING")?,
            FrameBound::Preceding(n) => write!(f, "{} PRECEDING", n)?,
            FrameBound::CurrentRow => write!(f, "CURRENT ROW")?,
            FrameBound::Following(n) => write!(f, "{} FOLLOWING", n)?,
            FrameBound::UnboundedFollowing => write!(f, "UNBOUNDED FOLLOWING")?,
        }
        Ok(())
    }
}

impl FrameBound<usize> {
    /// Convert the bound to sized offset from current row. `None` if the bound is unbounded.
    pub fn to_offset(&self) -> Option<isize> {
        match self {
            FrameBound::UnboundedPreceding | FrameBound::UnboundedFollowing => None,
            FrameBound::CurrentRow => Some(0),
            FrameBound::Preceding(n) => Some(-(*n as isize)),
            FrameBound::Following(n) => Some(*n as isize),
        }
    }

    /// View the bound as frame start, and get the number of preceding rows.
    pub fn n_preceding_rows(&self) -> Option<usize> {
        self.to_offset().map(|x| x.min(0).unsigned_abs())
    }

    /// View the bound as frame end, and get the number of following rows.
    pub fn n_following_rows(&self) -> Option<usize> {
        self.to_offset().map(|x| x.max(0) as usize)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
pub enum FrameExclusion {
    CurrentRow,
    // Group,
    // Ties,
    #[default]
    NoOthers,
}

impl Display for FrameExclusion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FrameExclusion::CurrentRow => write!(f, "EXCLUDE CURRENT ROW")?,
            FrameExclusion::NoOthers => write!(f, "EXCLUDE NO OTHERS")?,
        }
        Ok(())
    }
}

impl FrameExclusion {
    pub fn from_protobuf(exclusion: PbExclusion) -> Result<Self> {
        let excl = match exclusion {
            PbExclusion::Unspecified => bail!("unspecified type of `FrameExclusion`"),
            PbExclusion::CurrentRow => Self::CurrentRow,
            PbExclusion::NoOthers => Self::NoOthers,
        };
        Ok(excl)
    }

    pub fn to_protobuf(self) -> PbExclusion {
        match self {
            Self::CurrentRow => PbExclusion::CurrentRow,
            Self::NoOthers => PbExclusion::NoOthers,
        }
    }
}
