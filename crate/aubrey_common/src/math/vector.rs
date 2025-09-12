use core::fmt::{Debug, Formatter};
use core::ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign};

/// 汎用ベクトル型
/// - `T`: 成分の型（`i32` or `f32` を想定）
/// - `N`: 次元数（2,3,4 を想定）
/// - `IS_UNIT`: 正規化済みベクトルかどうか（`true` なら単位ベクトルとして扱う）
#[derive(Copy, Clone, PartialEq)]
pub struct BaseVector<T, const N: usize, const IS_UNIT: bool> {
    data: [T; N],
}

impl<T: Debug, const N: usize, const IS_UNIT: bool> Debug for BaseVector<T, N, IS_UNIT> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple(if IS_UNIT { "UnitVector" } else { "Vector" })
            .field(&self.data)
            .finish()
    }
}

impl<T, const N: usize, const IS_UNIT: bool> From<[T; N]> for BaseVector<T, N, IS_UNIT> {
    #[inline]
    fn from(value: [T; N]) -> Self { Self { data: value } }
}

impl<T: Copy, const N: usize, const IS_UNIT: bool> From<BaseVector<T, N, IS_UNIT>> for [T; N] {
    #[inline]
    fn from(value: BaseVector<T, N, IS_UNIT>) -> Self { value.data }
}

impl<T: Copy, const N: usize, const IS_UNIT: bool> BaseVector<T, N, IS_UNIT> {
    #[inline]
    pub const fn from_array(data: [T; N]) -> Self { Self { data } }

    #[inline]
    pub fn splat(v: T) -> Self { Self { data: [v; N] } }

    #[inline]
    pub fn as_array(&self) -> &[T; N] { &self.data }

    #[inline]
    pub fn to_array(self) -> [T; N] { self.data }
}

impl<T, const N: usize, const IS_UNIT: bool> Index<usize> for BaseVector<T, N, IS_UNIT> {
    type Output = T;
    #[inline]
    fn index(&self, index: usize) -> &Self::Output { &self.data[index] }
}

impl<T, const N: usize, const IS_UNIT: bool> IndexMut<usize> for BaseVector<T, N, IS_UNIT> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output { &mut self.data[index] }
}

// 次元ごとの便利コンストラクタとアクセサ
impl<T: Copy, const IS_UNIT: bool> BaseVector<T, 2, IS_UNIT> {
    #[inline]
    pub fn new(x: T, y: T) -> Self { Self { data: [x, y] } }
    #[inline]
    pub fn x(&self) -> T { self.data[0] }
    #[inline]
    pub fn y(&self) -> T { self.data[1] }
    #[inline]
    pub fn set_x(&mut self, v: T) { self.data[0] = v; }
    #[inline]
    pub fn set_y(&mut self, v: T) { self.data[1] = v; }
}

impl<T: Copy, const IS_UNIT: bool> BaseVector<T, 3, IS_UNIT> {
    #[inline]
    pub fn new(x: T, y: T, z: T) -> Self { Self { data: [x, y, z] } }
    #[inline]
    pub fn x(&self) -> T { self.data[0] }
    #[inline]
    pub fn y(&self) -> T { self.data[1] }
    #[inline]
    pub fn z(&self) -> T { self.data[2] }
    #[inline]
    pub fn set_x(&mut self, v: T) { self.data[0] = v; }
    #[inline]
    pub fn set_y(&mut self, v: T) { self.data[1] = v; }
    #[inline]
    pub fn set_z(&mut self, v: T) { self.data[2] = v; }
}

impl<T: Copy, const IS_UNIT: bool> BaseVector<T, 4, IS_UNIT> {
    #[inline]
    pub fn new(x: T, y: T, z: T, w: T) -> Self { Self { data: [x, y, z, w] } }
    #[inline]
    pub fn x(&self) -> T { self.data[0] }
    #[inline]
    pub fn y(&self) -> T { self.data[1] }
    #[inline]
    pub fn z(&self) -> T { self.data[2] }
    #[inline]
    pub fn w(&self) -> T { self.data[3] }
    #[inline]
    pub fn set_x(&mut self, v: T) { self.data[0] = v; }
    #[inline]
    pub fn set_y(&mut self, v: T) { self.data[1] = v; }
    #[inline]
    pub fn set_z(&mut self, v: T) { self.data[2] = v; }
    #[inline]
    pub fn set_w(&mut self, v: T) { self.data[3] = v; }
}

// ベクトル演算（成分ごと）
impl<T, const N: usize, const IS_UNIT: bool> Add for BaseVector<T, N, IS_UNIT>
where
    T: Copy + Add<Output = T>,
{
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        let mut out = self;
        for i in 0..N { out.data[i] = out.data[i] + rhs.data[i]; }
        out
    }
}

impl<T, const N: usize, const IS_UNIT: bool> AddAssign for BaseVector<T, N, IS_UNIT>
where
    T: Copy + Add<Output = T>,
{
    #[inline]
    fn add_assign(&mut self, rhs: Self) { for i in 0..N { self.data[i] = self.data[i] + rhs.data[i]; } }
}

impl<T, const N: usize, const IS_UNIT: bool> Sub for BaseVector<T, N, IS_UNIT>
where
    T: Copy + Sub<Output = T>,
{
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        let mut out = self;
        for i in 0..N { out.data[i] = out.data[i] - rhs.data[i]; }
        out
    }
}

impl<T, const N: usize, const IS_UNIT: bool> SubAssign for BaseVector<T, N, IS_UNIT>
where
    T: Copy + Sub<Output = T>,
{
    #[inline]
    fn sub_assign(&mut self, rhs: Self) { for i in 0..N { self.data[i] = self.data[i] - rhs.data[i]; } }
}

impl<T, const N: usize, const IS_UNIT: bool> Neg for BaseVector<T, N, IS_UNIT>
where
    T: Copy + Neg<Output = T>,
{
    type Output = Self;
    #[inline]
    fn neg(self) -> Self::Output {
        let mut out = self;
        for i in 0..N { out.data[i] = -out.data[i]; }
        out
    }
}

// スカラー積（スカラー×ベクトル / ベクトル×スカラー）
impl<T, const N: usize, const IS_UNIT: bool> Mul<T> for BaseVector<T, N, IS_UNIT>
where
    T: Copy + Mul<Output = T>,
{
    type Output = Self;
    #[inline]
    fn mul(self, rhs: T) -> Self::Output {
        let mut out = self;
        for i in 0..N { out.data[i] = out.data[i] * rhs; }
        out
    }
}

impl<T, const N: usize, const IS_UNIT: bool> MulAssign<T> for BaseVector<T, N, IS_UNIT>
where
    T: Copy + Mul<Output = T>,
{
    #[inline]
    fn mul_assign(&mut self, rhs: T) { for i in 0..N { self.data[i] = self.data[i] * rhs; } }
}

impl<T, const N: usize, const IS_UNIT: bool> Div<T> for BaseVector<T, N, IS_UNIT>
where
    T: Copy + Div<Output = T>,
{
    type Output = Self;
    #[inline]
    fn div(self, rhs: T) -> Self::Output {
        let mut out = self;
        for i in 0..N { out.data[i] = out.data[i] / rhs; }
        out
    }
}

impl<T, const N: usize, const IS_UNIT: bool> DivAssign<T> for BaseVector<T, N, IS_UNIT>
where
    T: Copy + Div<Output = T>,
{
    #[inline]
    fn div_assign(&mut self, rhs: T) { for i in 0..N { self.data[i] = self.data[i] / rhs; } }
}

impl<T, const N: usize, const IS_UNIT: bool> BaseVector<T, N, IS_UNIT>
where
    T: Copy + Add<Output = T> + Mul<Output = T> + Default,
{
    /// 内積
    #[inline]
    pub fn dot(&self, rhs: &Self) -> T {
        let mut acc: T = T::default();
        for i in 0..N { acc = acc + (self.data[i] * rhs.data[i]); }
        acc
    }
}

// f32 向けの長さ・正規化など
impl<const N: usize, const IS_UNIT: bool> BaseVector<f32, N, IS_UNIT> {
    #[inline]
    pub fn length_squared(&self) -> f32 { self.dot(self) }

    #[inline]
    pub fn length(&self) -> f32 { self.length_squared().sqrt() }

    /// 正規化したベクトルを返す（ゼロベクトルはそのまま返す）
    #[inline]
    pub fn normalized(&self) -> BaseVector<f32, N, true> {
        let len = self.length();
        if len == 0.0 { BaseVector::<f32, N, true>::from_array(self.data) } else { (*self / len).cast_unit() }
    }

    /// だいたい単位長かチェック（1e-4）
    #[inline]
    pub fn approx_is_unit(&self) -> bool { (self.length() - 1.0).abs() < 1e-4 }

    #[inline]
    fn cast_unit(self) -> BaseVector<f32, N, true> { BaseVector::<f32, N, true>::from_array(self.data) }
}

// i32 向けの長さ^2（ユークリッド距離の二乗）
impl<const N: usize, const IS_UNIT: bool> BaseVector<i32, N, IS_UNIT> {
    #[inline]
    pub fn length_squared(&self) -> i32 { self.dot(self) }
}

pub type Vector2f = BaseVector<f32, 2, false>;
pub type Vector3f = BaseVector<f32, 3, false>;
pub type Vector4f = BaseVector<f32, 4, false>;
pub type Vector2i = BaseVector<i32, 2, false>;
pub type Vector3i = BaseVector<i32, 3, false>;
pub type Vector4i = BaseVector<i32, 4, false>;

