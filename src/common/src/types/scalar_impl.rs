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

use std::hash::Hasher;

use super::*;
use crate::array::list_array::{ListRef, ListValue};
use crate::array::struct_array::{StructRef, StructValue};
use crate::{for_all_native_types, for_all_scalar_variants};

/// `ScalarPartialOrd` allows comparison between `Scalar` and `ScalarRef`.
///
/// TODO: see if it is possible to implement this trait directly on `ScalarRef`.
pub trait ScalarPartialOrd: Scalar {
    fn scalar_cmp(&self, other: Self::ScalarRefType<'_>) -> Option<std::cmp::Ordering>;
}

/// Implement `Scalar` and `ScalarRef` for native type.
/// For `PrimitiveArrayItemType`, clone is trivial, so `T` is both `Scalar` and `ScalarRef`.
macro_rules! impl_all_native_scalar {
    ($({ $scalar_type:ty, $variant_name:ident } ),*) => {
        $(
            impl Scalar for $scalar_type {
                type ScalarRefType<'a> = Self;

                fn as_scalar_ref(&self) -> Self {
                    *self
                }
            }

            impl<'scalar> ScalarRef<'scalar> for $scalar_type {
                type ScalarType = Self;

                fn to_owned_scalar(&self) -> Self {
                    *self
                }

                fn hash_scalar<H: std::hash::Hasher>(&self, state: &mut H) {
                    self.hash(state)
                }
            }
        )*
    };
}

for_all_native_types! { impl_all_native_scalar }

/// Implement `Scalar` for `Box<str>`.
/// `Box<str>` could be converted to `&str`.
impl Scalar for Box<str> {
    type ScalarRefType<'a> = &'a str;

    fn as_scalar_ref(&self) -> &str {
        self.as_ref()
    }
}

/// Implement `Scalar` for `Bytes`.
impl Scalar for Box<[u8]> {
    type ScalarRefType<'a> = &'a [u8];

    fn as_scalar_ref(&self) -> &[u8] {
        self
    }
}

/// Implement `Scalar` for `StructValue`.
impl Scalar for StructValue {
    type ScalarRefType<'a> = StructRef<'a>;

    fn as_scalar_ref(&self) -> StructRef<'_> {
        StructRef::ValueRef { val: self }
    }
}

/// Implement `Scalar` for `ListValue`.
impl Scalar for ListValue {
    type ScalarRefType<'a> = ListRef<'a>;

    fn as_scalar_ref(&self) -> ListRef<'_> {
        ListRef::ValueRef { val: self }
    }
}

/// Implement `ScalarRef` for `Box<str>`.
/// `Box<str>` could be converted to `&str`.
impl<'a> ScalarRef<'a> for &'a str {
    type ScalarType = Box<str>;

    fn to_owned_scalar(&self) -> Box<str> {
        (*self).into()
    }

    fn hash_scalar<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hash(state)
    }
}

impl<'a> ScalarRef<'a> for &'a [u8] {
    type ScalarType = Box<[u8]>;

    fn to_owned_scalar(&self) -> Box<[u8]> {
        self.to_vec().into()
    }

    fn hash_scalar<H: Hasher>(&self, state: &mut H) {
        self.hash(state)
    }
}

impl ScalarPartialOrd for Box<str> {
    fn scalar_cmp(&self, other: &str) -> Option<std::cmp::Ordering> {
        self.as_ref().partial_cmp(other)
    }
}

impl<T: PrimitiveArrayItemType + Scalar> ScalarPartialOrd for T {
    fn scalar_cmp(&self, other: Self) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&other)
    }
}

impl ScalarPartialOrd for bool {
    fn scalar_cmp(&self, other: Self) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&other)
    }
}

/// Implement `Scalar` for `bool`.
impl Scalar for bool {
    type ScalarRefType<'a> = bool;

    fn as_scalar_ref(&self) -> bool {
        *self
    }
}

/// Implement `ScalarRef` for `bool`.
impl<'a> ScalarRef<'a> for bool {
    type ScalarType = bool;

    fn to_owned_scalar(&self) -> bool {
        *self
    }

    fn hash_scalar<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hash(state)
    }
}

/// Implement `Scalar` for `Decimal`.
impl Scalar for Decimal {
    type ScalarRefType<'a> = Decimal;

    fn as_scalar_ref(&self) -> Decimal {
        *self
    }
}

/// Implement `ScalarRef` for `Decimal`.
impl<'a> ScalarRef<'a> for Decimal {
    type ScalarType = Decimal;

    fn to_owned_scalar(&self) -> Decimal {
        *self
    }

    fn hash_scalar<H: std::hash::Hasher>(&self, state: &mut H) {
        self.normalize().hash(state)
    }
}

/// Implement `Scalar` for `Interval`.
impl Scalar for Interval {
    type ScalarRefType<'a> = Interval;

    fn as_scalar_ref(&self) -> Interval {
        *self
    }
}

/// Implement `ScalarRef` for `Interval`.
impl<'a> ScalarRef<'a> for Interval {
    type ScalarType = Interval;

    fn to_owned_scalar(&self) -> Interval {
        *self
    }

    fn hash_scalar<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hash(state)
    }
}

/// Implement `Scalar` for `Date`.
impl Scalar for Date {
    type ScalarRefType<'a> = Date;

    fn as_scalar_ref(&self) -> Date {
        *self
    }
}

/// Implement `ScalarRef` for `Date`.
impl<'a> ScalarRef<'a> for Date {
    type ScalarType = Date;

    fn to_owned_scalar(&self) -> Date {
        *self
    }

    fn hash_scalar<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hash(state)
    }
}

/// Implement `Scalar` for `Timestamp`.
impl Scalar for Timestamp {
    type ScalarRefType<'a> = Timestamp;

    fn as_scalar_ref(&self) -> Timestamp {
        *self
    }
}

/// Implement `ScalarRef` for `Timestamp`.
impl<'a> ScalarRef<'a> for Timestamp {
    type ScalarType = Timestamp;

    fn to_owned_scalar(&self) -> Timestamp {
        *self
    }

    fn hash_scalar<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hash(state)
    }
}

/// Implement `Scalar` for `Time`.
impl Scalar for Time {
    type ScalarRefType<'a> = Time;

    fn as_scalar_ref(&self) -> Time {
        *self
    }
}

/// Implement `ScalarRef` for `Time`.
impl<'a> ScalarRef<'a> for Time {
    type ScalarType = Time;

    fn to_owned_scalar(&self) -> Time {
        *self
    }

    fn hash_scalar<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hash(state)
    }
}

/// Implement `Scalar` for `StructValue`.
impl<'a> ScalarRef<'a> for StructRef<'a> {
    type ScalarType = StructValue;

    fn to_owned_scalar(&self) -> StructValue {
        let fields = self.iter_fields_ref().map(|f| f.to_owned_datum()).collect();
        StructValue::new(fields)
    }

    fn hash_scalar<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hash_scalar_inner(state)
    }
}

/// Implement `Scalar` for `ListValue`.
impl<'a> ScalarRef<'a> for ListRef<'a> {
    type ScalarType = ListValue;

    fn to_owned_scalar(&self) -> ListValue {
        let fields = self
            .iter()
            .map(|f| f.map(|s| s.into_scalar_impl()))
            .collect();
        ListValue::new(fields)
    }

    fn hash_scalar<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hash_scalar_inner(state)
    }
}

impl ScalarImpl {
    pub fn get_ident(&self) -> &'static str {
        macro_rules! impl_all_get_ident {
            ($({ $variant_name:ident, $suffix_name:ident, $scalar:ty, $scalar_ref:ty } ),*) => {
                match self {
                    $( Self::$variant_name(_) => stringify!($variant_name), )*
                }
            };
        }
        for_all_scalar_variants! { impl_all_get_ident }
    }
}

impl<'scalar> ScalarRefImpl<'scalar> {
    pub fn get_ident(&self) -> &'static str {
        macro_rules! impl_all_get_ident {
            ($({ $variant_name:ident, $suffix_name:ident, $scalar:ty, $scalar_ref:ty } ),*) => {
                match self {
                    $( Self::$variant_name(_) => stringify!($variant_name), )*
                }
            };
        }
        for_all_scalar_variants! { impl_all_get_ident }
    }
}
