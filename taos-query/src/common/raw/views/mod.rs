mod bool_view;

pub use bool_view::BoolView;

mod tinyint_view;

use itertools::Itertools;
pub use tinyint_view::TinyIntView;

mod smallint_view;
pub use smallint_view::SmallIntView;

mod int_view;
pub use int_view::IntView;

mod big_int_view;
pub use big_int_view::BigIntView;

mod float_view;
pub use float_view::FloatView;

mod double_view;
pub use double_view::DoubleView;

mod tinyint_unsigned_view;
pub use tinyint_unsigned_view::UTinyIntView;

mod small_int_unsigned_view;
pub use small_int_unsigned_view::USmallIntView;

mod int_unsigned_view;
pub use int_unsigned_view::UIntView;

mod big_int_unsigned_view;
pub use big_int_unsigned_view::UBigIntView;

mod timestamp_view;
pub use timestamp_view::TimestampView;

mod var_char_view;
pub use var_char_view::VarCharView;

mod n_char_view;
pub use n_char_view::NCharView;

mod json_view;
pub use json_view::JsonView;

mod schema;
pub(crate) use schema::*;

mod nulls;
pub(crate) use nulls::*;

mod offsets;
pub(crate) use offsets::*;

mod lengths;
pub(crate) use lengths::*;

mod from;

use crate::common::{BorrowedValue, Ty, Value};

use std::{ffi::c_void, fmt::Debug, io::Write, iter::FusedIterator};

pub(crate) trait IsColumnView: Sized {
    /// View item data type.
    fn ty(&self) -> Ty;

    fn from_value_iter<'a>(iter: impl Iterator<Item = &'a Value>) -> Self {
        Self::from_borrowed_value_iter::<'a>(iter.map(|v| v.to_borrowed_value()))
    }

    fn from_borrowed_value_iter<'b>(iter: impl Iterator<Item = BorrowedValue<'b>>) -> Self;
}

/// Compatible version for var char.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Version {
    V2,
    V3,
}

// #[derive(Debug)]
pub enum ColumnView {
    Bool(BoolView),           // 1
    TinyInt(TinyIntView),     // 2
    SmallInt(SmallIntView),   // 3
    Int(IntView),             // 4
    BigInt(BigIntView),       // 5
    Float(FloatView),         // 6
    Double(DoubleView),       // 7
    VarChar(VarCharView),     // 8
    Timestamp(TimestampView), // 9
    NChar(NCharView),         // 10
    UTinyInt(UTinyIntView),   // 11
    USmallInt(USmallIntView), // 12
    UInt(UIntView),           // 13
    UBigInt(UBigIntView),     // 14
    Json(JsonView),           // 15
}
unsafe impl Send for ColumnView {}
unsafe impl Sync for ColumnView {}

impl Debug for ColumnView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool(view) => f.debug_tuple("Bool").field(&view.to_vec()).finish(),
            Self::TinyInt(view) => f.debug_tuple("TinyInt").field(&view.to_vec()).finish(),
            Self::SmallInt(view) => f.debug_tuple("SmallInt").field(&view.to_vec()).finish(),
            Self::Int(view) => f.debug_tuple("Int").field(&view.to_vec()).finish(),
            Self::BigInt(view) => f.debug_tuple("BigInt").field(&view.to_vec()).finish(),
            Self::Float(view) => f.debug_tuple("Float").field(&view.to_vec()).finish(),
            Self::Double(view) => f.debug_tuple("Double").field(&view.to_vec()).finish(),
            Self::VarChar(view) => f.debug_tuple("VarChar").field(&view.to_vec()).finish(),
            Self::Timestamp(view) => f.debug_tuple("Timestamp").field(&view.to_vec()).finish(),
            Self::NChar(view) => f.debug_tuple("NChar").field(&view.to_vec()).finish(),
            Self::UTinyInt(view) => f.debug_tuple("UTinyInt").field(&view.to_vec()).finish(),
            Self::USmallInt(view) => f.debug_tuple("USmallInt").field(&view.to_vec()).finish(),
            Self::UInt(view) => f.debug_tuple("UInt").field(&view.to_vec()).finish(),
            Self::UBigInt(view) => f.debug_tuple("UBigInt").field(&view.to_vec()).finish(),
            Self::Json(view) => f.debug_tuple("Json").field(&view.to_vec()).finish(),
        }
    }
}

impl std::ops::Add for ColumnView {
    type Output = ColumnView;

    fn add(self, rhs: Self) -> Self::Output {
        self.concat(&rhs)
    }
}

pub struct ColumnViewIter<'c> {
    view: &'c ColumnView,
    row: usize,
}

impl<'c> Iterator for ColumnViewIter<'c> {
    type Item = BorrowedValue<'c>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.row >= self.view.len() {
            return None;
        }
        let row = self.row;
        self.row += 1;
        Some(unsafe { self.view.get_ref_unchecked(row) })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.row, Some(self.view.len() - self.row))
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let row = self.row + n;
        if row >= self.view.len() {
            return None;
        }
        self.row += n;
        Some(unsafe { self.view.get_ref_unchecked(row) })
    }

    #[inline]
    fn count(self) -> usize {
        self.view.len()
    }
}

impl<'c> ExactSizeIterator for ColumnViewIter<'c> {
    fn len(&self) -> usize {
        self.view.len() - self.row
    }
}

impl<'a> FusedIterator for ColumnViewIter<'a> {}

impl<'a> IntoIterator for &'a ColumnView {
    type Item = BorrowedValue<'a>;

    type IntoIter = ColumnViewIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl ColumnView {
    pub fn from_millis_timestamp(values: Vec<impl Into<Option<i64>>>) -> Self {
        ColumnView::Timestamp(TimestampView::from_millis(values))
    }
    pub fn from_bools(values: Vec<impl Into<Option<bool>>>) -> Self {
        ColumnView::Bool(BoolView::from_iter(values))
    }
    pub fn from_tiny_ints(values: Vec<impl Into<Option<i8>>>) -> Self {
        ColumnView::TinyInt(TinyIntView::from_iter(values))
    }
    pub fn from_small_ints(values: Vec<impl Into<Option<i16>>>) -> Self {
        ColumnView::SmallInt(SmallIntView::from_iter(values))
    }
    pub fn from_ints(values: Vec<impl Into<Option<i32>>>) -> Self {
        ColumnView::Int(IntView::from_iter(values))
    }
    pub fn from_big_ints(values: Vec<impl Into<Option<i64>>>) -> Self {
        ColumnView::BigInt(BigIntView::from_iter(values))
    }
    pub fn from_unsigned_tiny_ints(values: Vec<impl Into<Option<u8>>>) -> Self {
        ColumnView::UTinyInt(UTinyIntView::from_iter(values))
    }
    pub fn from_unsigned_small_ints(values: Vec<impl Into<Option<u16>>>) -> Self {
        ColumnView::USmallInt(USmallIntView::from_iter(values))
    }
    pub fn from_unsigned_ints(values: Vec<impl Into<Option<u32>>>) -> Self {
        ColumnView::UInt(UIntView::from_iter(values))
    }
    pub fn from_unsigned_big_ints(values: Vec<impl Into<Option<u64>>>) -> Self {
        ColumnView::UBigInt(UBigIntView::from_iter(values))
    }
    pub fn from_floats(values: Vec<impl Into<Option<f32>>>) -> Self {
        ColumnView::Float(FloatView::from_iter(values))
    }
    pub fn from_doubles(values: Vec<impl Into<Option<f64>>>) -> Self {
        ColumnView::Double(DoubleView::from_iter(values))
    }
    pub fn from_varchar<
        S: AsRef<str>,
        T: Into<Option<S>>,
        I: ExactSizeIterator<Item = T>,
        V: IntoIterator<Item = T, IntoIter = I>,
    >(
        iter: V,
    ) -> Self {
        ColumnView::VarChar(VarCharView::from_iter(iter))
    }
    pub fn from_nchar<
        S: AsRef<str>,
        T: Into<Option<S>>,
        I: ExactSizeIterator<Item = T>,
        V: IntoIterator<Item = T, IntoIter = I>,
    >(
        iter: V,
    ) -> Self {
        ColumnView::NChar(NCharView::from_iter(iter))
    }

    #[inline]
    pub fn concat_iter<'b, 'a: 'b>(
        &'a self,
        rhs: impl Iterator<Item = BorrowedValue<'b>>,
        ty: Ty,
    ) -> ColumnView {
        match ty {
            Ty::Null => unreachable!(),
            Ty::Bool => ColumnView::Bool(IsColumnView::from_borrowed_value_iter(
                self.iter().chain(rhs),
            )),
            Ty::TinyInt => ColumnView::TinyInt(IsColumnView::from_borrowed_value_iter(
                self.iter().chain(rhs),
            )),
            Ty::SmallInt => ColumnView::SmallInt(IsColumnView::from_borrowed_value_iter(
                self.iter().chain(rhs),
            )),
            Ty::Int => ColumnView::Int(IsColumnView::from_borrowed_value_iter(
                self.iter().chain(rhs),
            )),
            Ty::BigInt => ColumnView::BigInt(IsColumnView::from_borrowed_value_iter(
                self.iter().chain(rhs),
            )),
            Ty::UTinyInt => ColumnView::UTinyInt(IsColumnView::from_borrowed_value_iter(
                self.iter().chain(rhs),
            )),
            Ty::USmallInt => ColumnView::USmallInt(IsColumnView::from_borrowed_value_iter(
                self.iter().chain(rhs),
            )),
            Ty::UInt => ColumnView::UInt(IsColumnView::from_borrowed_value_iter(
                self.iter().chain(rhs),
            )),
            Ty::UBigInt => ColumnView::UBigInt(IsColumnView::from_borrowed_value_iter(
                self.iter().chain(rhs),
            )),
            Ty::Float => ColumnView::Float(IsColumnView::from_borrowed_value_iter(
                self.iter().chain(rhs),
            )),
            Ty::Double => ColumnView::Double(IsColumnView::from_borrowed_value_iter(
                self.iter().chain(rhs),
            )),
            Ty::Timestamp => ColumnView::Timestamp(IsColumnView::from_borrowed_value_iter(
                self.iter().chain(rhs),
            )),
            Ty::VarChar => ColumnView::VarChar(IsColumnView::from_borrowed_value_iter(
                self.iter().chain(rhs),
            )),
            Ty::NChar => ColumnView::NChar(IsColumnView::from_borrowed_value_iter(
                self.iter().chain(rhs),
            )),
            Ty::Json => ColumnView::Json(IsColumnView::from_borrowed_value_iter(
                self.iter().chain(rhs),
            )),
            Ty::VarBinary => todo!(),
            Ty::Decimal => todo!(),
            Ty::Blob => todo!(),
            Ty::MediumBlob => todo!(),
        }
    }

    /// Concatenate another column view, output a new column view with exact type of self.
    #[inline]
    pub fn concat(&self, rhs: &ColumnView) -> ColumnView {
        self.concat_as(rhs, self.as_ty())
    }

    /// Concatenate another column view, output a new column view with specified type `ty`.
    #[inline]
    pub fn concat_as(&self, rhs: &ColumnView, ty: Ty) -> ColumnView {
        self.concat_iter(rhs.iter(), ty)
    }

    /// Generate single element view for specified type `ty`.
    pub fn null(n: usize, ty: Ty) -> Self {
        match ty {
            Ty::Null => panic!("type should be known"),
            Ty::Bool => Self::from_bools(vec![None; n]),
            Ty::TinyInt => Self::from_tiny_ints(vec![None; n]),
            Ty::SmallInt => Self::from_small_ints(vec![None; n]),
            Ty::Int => Self::from_ints(vec![None; n]),
            Ty::BigInt => Self::from_big_ints(vec![None; n]),
            Ty::UTinyInt => Self::from_unsigned_tiny_ints(vec![None; n]),
            Ty::USmallInt => Self::from_unsigned_small_ints(vec![None; n]),
            Ty::UInt => Self::from_unsigned_ints(vec![None; n]),
            Ty::UBigInt => Self::from_unsigned_big_ints(vec![None; n]),
            Ty::Float => Self::from_floats(vec![None; n]),
            Ty::Double => Self::from_doubles(vec![None; n]),
            Ty::Timestamp => Self::from_millis_timestamp(vec![None; n]),
            Ty::VarChar => Self::from_varchar::<&'static str, _, _, _>(vec![None; n]),
            Ty::NChar => Self::from_nchar::<&'static str, _, _, _>(vec![None; n]),
            Ty::Json => todo!(),
            Ty::VarBinary => todo!(),
            Ty::Decimal => todo!(),
            Ty::Blob => todo!(),
            Ty::MediumBlob => todo!(),
        }
    }

    /// It's equal to the cols
    pub fn len(&self) -> usize {
        match self {
            ColumnView::Bool(view) => view.len(),
            ColumnView::TinyInt(view) => view.len(),
            ColumnView::SmallInt(view) => view.len(),
            ColumnView::Int(view) => view.len(),
            ColumnView::BigInt(view) => view.len(),
            ColumnView::Float(view) => view.len(),
            ColumnView::Double(view) => view.len(),
            ColumnView::VarChar(view) => view.len(),
            ColumnView::Timestamp(view) => view.len(),
            ColumnView::NChar(view) => view.len(),
            ColumnView::UTinyInt(view) => view.len(),
            ColumnView::USmallInt(view) => view.len(),
            ColumnView::UInt(view) => view.len(),
            ColumnView::UBigInt(view) => view.len(),
            ColumnView::Json(view) => view.len(),
        }
    }

    /// Check if a value at `row` is null
    #[inline]
    pub(super) unsafe fn is_null_unchecked(&self, row: usize) -> bool {
        match self {
            ColumnView::Bool(view) => view.is_null_unchecked(row),
            ColumnView::TinyInt(view) => view.is_null_unchecked(row),
            ColumnView::SmallInt(view) => view.is_null_unchecked(row),
            ColumnView::Int(view) => view.is_null_unchecked(row),
            ColumnView::BigInt(view) => view.is_null_unchecked(row),
            ColumnView::Float(view) => view.is_null_unchecked(row),
            ColumnView::Double(view) => view.is_null_unchecked(row),
            ColumnView::VarChar(view) => view.is_null_unchecked(row),
            ColumnView::Timestamp(view) => view.is_null_unchecked(row),
            ColumnView::NChar(view) => view.is_null_unchecked(row),
            ColumnView::UTinyInt(view) => view.is_null_unchecked(row),
            ColumnView::USmallInt(view) => view.is_null_unchecked(row),
            ColumnView::UInt(view) => view.is_null_unchecked(row),
            ColumnView::UBigInt(view) => view.is_null_unchecked(row),
            ColumnView::Json(view) => view.is_null_unchecked(row),
        }
    }

    pub fn get(&self, row: usize) -> Option<BorrowedValue> {
        if row < self.len() {
            Some(unsafe { self.get_ref_unchecked(row) })
        } else {
            None
        }
    }

    /// Get one value at `row` index of the column view.
    #[inline]
    pub(super) unsafe fn get_ref_unchecked(&self, row: usize) -> BorrowedValue {
        match self {
            ColumnView::Bool(view) => view.get_value_unchecked(row),
            ColumnView::TinyInt(view) => view.get_value_unchecked(row),
            ColumnView::SmallInt(view) => view.get_value_unchecked(row),
            ColumnView::Int(view) => view.get_value_unchecked(row),
            ColumnView::BigInt(view) => view.get_value_unchecked(row),
            ColumnView::Float(view) => view.get_value_unchecked(row),
            ColumnView::Double(view) => view.get_value_unchecked(row),
            ColumnView::VarChar(view) => view.get_value_unchecked(row),
            ColumnView::Timestamp(view) => view.get_value_unchecked(row),
            ColumnView::NChar(view) => view.get_value_unchecked(row),
            ColumnView::UTinyInt(view) => view.get_value_unchecked(row),
            ColumnView::USmallInt(view) => view.get_value_unchecked(row),
            ColumnView::UInt(view) => view.get_value_unchecked(row),
            ColumnView::UBigInt(view) => view.get_value_unchecked(row),
            ColumnView::Json(view) => view.get_value_unchecked(row),
        }
    }

    /// Get pointer to value.
    #[inline]
    pub(super) unsafe fn get_raw_value_unchecked(&self, row: usize) -> (Ty, u32, *const c_void) {
        match self {
            ColumnView::Bool(view) => view.get_raw_value_unchecked(row),
            ColumnView::TinyInt(view) => view.get_raw_value_unchecked(row),
            ColumnView::SmallInt(view) => view.get_raw_value_unchecked(row),
            ColumnView::Int(view) => view.get_raw_value_unchecked(row),
            ColumnView::BigInt(view) => view.get_raw_value_unchecked(row),
            ColumnView::Float(view) => view.get_raw_value_unchecked(row),
            ColumnView::Double(view) => view.get_raw_value_unchecked(row),
            ColumnView::VarChar(view) => view.get_raw_value_unchecked(row),
            ColumnView::Timestamp(view) => view.get_raw_value_unchecked(row),
            ColumnView::NChar(view) => view.get_raw_value_unchecked(row),
            ColumnView::UTinyInt(view) => view.get_raw_value_unchecked(row),
            ColumnView::USmallInt(view) => view.get_raw_value_unchecked(row),
            ColumnView::UInt(view) => view.get_raw_value_unchecked(row),
            ColumnView::UBigInt(view) => view.get_raw_value_unchecked(row),
            ColumnView::Json(view) => view.get_raw_value_unchecked(row),
        }
    }

    pub fn iter(&self) -> ColumnViewIter {
        ColumnViewIter { view: self, row: 0 }
    }

    pub fn slice(&self, range: std::ops::Range<usize>) -> Option<Self> {
        match self {
            ColumnView::Bool(view) => view.slice(range).map(ColumnView::Bool),
            ColumnView::TinyInt(view) => view.slice(range).map(ColumnView::TinyInt),
            ColumnView::SmallInt(view) => view.slice(range).map(ColumnView::SmallInt),
            ColumnView::Int(view) => view.slice(range).map(ColumnView::Int),
            ColumnView::BigInt(view) => view.slice(range).map(ColumnView::BigInt),
            ColumnView::Float(view) => view.slice(range).map(ColumnView::Float),
            ColumnView::Double(view) => view.slice(range).map(ColumnView::Double),
            ColumnView::VarChar(view) => view.slice(range).map(ColumnView::VarChar),
            ColumnView::Timestamp(view) => view.slice(range).map(ColumnView::Timestamp),
            ColumnView::NChar(view) => view.slice(range).map(ColumnView::NChar),
            ColumnView::UTinyInt(view) => view.slice(range).map(ColumnView::UTinyInt),
            ColumnView::USmallInt(view) => view.slice(range).map(ColumnView::USmallInt),
            ColumnView::UInt(view) => view.slice(range).map(ColumnView::UInt),
            ColumnView::UBigInt(view) => view.slice(range).map(ColumnView::UBigInt),
            ColumnView::Json(view) => view.slice(range).map(ColumnView::Json),
        }
    }

    // pub fn into_column(&self) -> Column {
    //     match self {
    //         ColumnView::Bool(view) => Column::Bool(
    //             bitvec_simd::BitVec::from_bool_iterator(view.is_null_iter()),
    //             view.as_raw_slice().to_vec(),
    //         ),
    //         ColumnView::TinyInt(view) => Column::TinyInt(
    //             bitvec_simd::BitVec::from_bool_iterator(view.is_null_iter()),
    //             view.as_raw_slice().to_vec(),
    //         ),
    //         ColumnView::SmallInt(view) => Column::SmallInt(
    //             bitvec_simd::BitVec::from_bool_iterator(view.is_null_iter()),
    //             view.as_raw_slice().to_vec(),
    //         ),
    //         ColumnView::Int(view) => Column::Int(
    //             bitvec_simd::BitVec::from_bool_iterator(view.is_null_iter()),
    //             view.as_raw_slice().to_vec(),
    //         ),
    //         ColumnView::BigInt(view) => Column::BigInt(
    //             bitvec_simd::BitVec::from_bool_iterator(view.is_null_iter()),
    //             view.as_raw_slice().to_vec(),
    //         ),
    //         ColumnView::Float(view) => Column::Float(
    //             bitvec_simd::BitVec::from_bool_iterator(view.is_null_iter()),
    //             view.as_raw_slice().to_vec(),
    //         ),
    //         ColumnView::Double(view) => Column::Double(
    //             bitvec_simd::BitVec::from_bool_iterator(view.is_null_iter()),
    //             view.as_raw_slice().to_vec(),
    //         ),
    //         ColumnView::VarChar(view) => Column::VarBinary(
    //             bitvec_simd::BitVec::from_bool_iterator(view.is_null_iter()),
    //             view.iter()
    //                 .map(|v| v.map(|s| s.as_bytes().to_vec()).unwrap_or_default())
    //                 .collect(),
    //         ),
    //         ColumnView::Timestamp(_) => todo!(),
    //         ColumnView::NChar(view) => {
    //             Column::NChar(view.iter().map(|s| s.map(|s| s.to_string())).collect())
    //         }
    //         ColumnView::UTinyInt(view) => Column::UTinyInt(
    //             bitvec_simd::BitVec::from_bool_iterator(view.is_null_iter()),
    //             view.as_raw_slice().to_vec(),
    //         ),
    //         ColumnView::USmallInt(view) => Column::USmallInt(
    //             bitvec_simd::BitVec::from_bool_iterator(view.is_null_iter()),
    //             view.as_raw_slice().to_vec(),
    //         ),
    //         ColumnView::UInt(view) => Column::UInt(
    //             bitvec_simd::BitVec::from_bool_iterator(view.is_null_iter()),
    //             view.as_raw_slice().to_vec(),
    //         ),
    //         ColumnView::UBigInt(view) => Column::UBigInt(
    //             bitvec_simd::BitVec::from_bool_iterator(view.is_null_iter()),
    //             view.as_raw_slice().to_vec(),
    //         ),
    //         ColumnView::Json(_) => todo!(),
    //     }
    // }

    pub(super) fn write_raw_into<W: Write>(&self, wtr: &mut W) -> std::io::Result<usize> {
        match self {
            ColumnView::Bool(view) => view.write_raw_into(wtr),
            ColumnView::TinyInt(view) => view.write_raw_into(wtr),
            ColumnView::SmallInt(view) => view.write_raw_into(wtr),
            ColumnView::Int(view) => view.write_raw_into(wtr),
            ColumnView::BigInt(view) => view.write_raw_into(wtr),
            ColumnView::Float(view) => view.write_raw_into(wtr),
            ColumnView::Double(view) => view.write_raw_into(wtr),
            ColumnView::VarChar(view) => view.write_raw_into(wtr),
            ColumnView::Timestamp(view) => view.write_raw_into(wtr),
            ColumnView::NChar(view) => view.write_raw_into(wtr),
            ColumnView::UTinyInt(view) => view.write_raw_into(wtr),
            ColumnView::USmallInt(view) => view.write_raw_into(wtr),
            ColumnView::UInt(view) => view.write_raw_into(wtr),
            ColumnView::UBigInt(view) => view.write_raw_into(wtr),
            ColumnView::Json(view) => view.write_raw_into(wtr),
        }
    }

    fn as_ty(&self) -> Ty {
        match self {
            ColumnView::Bool(_) => Ty::Bool,
            ColumnView::TinyInt(_) => Ty::TinyInt,
            ColumnView::SmallInt(_) => Ty::SmallInt,
            ColumnView::Int(_) => Ty::Int,
            ColumnView::BigInt(_) => Ty::BigInt,
            ColumnView::Float(_) => Ty::Float,
            ColumnView::Double(_) => Ty::Double,
            ColumnView::VarChar(_) => Ty::VarChar,
            ColumnView::Timestamp(_) => Ty::Timestamp,
            ColumnView::NChar(_) => Ty::NChar,
            ColumnView::UTinyInt(_) => Ty::UTinyInt,
            ColumnView::USmallInt(_) => Ty::USmallInt,
            ColumnView::UInt(_) => Ty::UInt,
            ColumnView::UBigInt(_) => Ty::UBigInt,
            ColumnView::Json(_) => Ty::Json,
        }
    }

    pub unsafe fn as_timestamp_view(&self) -> &TimestampView {
        match self {
            ColumnView::Timestamp(view) => view,
            _ => unreachable!(),
        }
    }

    pub(crate) fn to_nulls_vec(&self) -> Vec<bool> {
        (0..self.len())
            .map(|i| unsafe { self.is_null_unchecked(i) })
            .collect()
    }
}

pub fn views_to_raw_block(views: &[ColumnView]) -> Vec<u8> {
    let mut header = super::Header::default();

    header.nrows = views.first().map(|v| v.len()).unwrap_or(0) as _;
    header.ncols = views.len() as _;

    let ncols = views.len();

    let mut bytes = Vec::new();
    bytes.extend(header.as_bytes());

    let schemas = views
        .iter()
        .map(|view| {
            let ty = view.as_ty();
            ColSchema {
                ty,
                len: ty.fixed_length() as _,
            }
        })
        .collect_vec();
    let schema_bytes = unsafe {
        std::slice::from_raw_parts(
            schemas.as_ptr() as *const u8,
            ncols * std::mem::size_of::<ColSchema>(),
        )
    };
    bytes.write_all(schema_bytes).unwrap();

    let length_offset = bytes.len();
    bytes.resize(bytes.len() + ncols * std::mem::size_of::<u32>(), 0);

    let mut lengths = Vec::with_capacity(ncols);
    lengths.resize(ncols, 0);
    for (i, view) in views.iter().enumerate() {
        let cur = bytes.len();
        let n = view.write_raw_into(&mut bytes).unwrap();
        let len = bytes.len();
        debug_assert!(cur + n == len);
        if !view.as_ty().is_primitive() {
            lengths[i] = (n - header.nrows() * 4) as _;
        } else {
            lengths[i] = (header.nrows() * view.as_ty().fixed_length()) as _;
        }
    }
    unsafe {
        (*(bytes.as_mut_ptr() as *mut super::Header)).length = bytes.len() as _;
        std::ptr::copy(
            lengths.as_ptr(),
            bytes.as_mut_ptr().offset(length_offset as isize) as *mut u32,
            lengths.len(),
        );
    }
    bytes
}

impl From<Value> for ColumnView {
    fn from(value: Value) -> Self {
        match value {
            Value::Null(ty) => ColumnView::null(1, ty),
            Value::Bool(v) => vec![v].into(),
            Value::TinyInt(v) => vec![v].into(),
            Value::SmallInt(v) => vec![v].into(),
            Value::Int(v) => vec![v].into(),
            Value::BigInt(v) => vec![v].into(),
            Value::Float(v) => vec![v].into(),
            Value::Double(v) => vec![v].into(),
            Value::VarChar(v) => ColumnView::from_varchar::<String, _, _, _>(vec![v]),
            Value::Timestamp(v) => ColumnView::Timestamp(TimestampView::from_timestamp(vec![v])),
            Value::NChar(v) => ColumnView::from_nchar::<String, _, _, _>(vec![v]),
            Value::UTinyInt(v) => vec![v].into(),
            Value::USmallInt(v) => vec![v].into(),
            Value::UInt(v) => vec![v].into(),
            Value::UBigInt(v) => vec![v].into(),
            _ => todo!(),
        }
    }
}
