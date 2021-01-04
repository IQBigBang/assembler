// This file is part of assembler. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/assembler/master/COPYRIGHT. No part of predicator, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2018 The developers of assembler. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/assembler/master/COPYRIGHT.


/// 32-bit memory offset.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MemoryOffset32Bit
{
	/// `segment:offset` form.
	///
	/// Segment register is ignored in 64-bit long mode.
	SegmentOffsetForm32(SegmentRegister, Immediate64Bit),

	/// `offset` form.
	OffsetForm32(Immediate64Bit),
}

impl Default for MemoryOffset32Bit
{
	#[inline(always)]
	fn default() -> Self
	{
		MemoryOffset32Bit::OffsetForm32(Immediate64Bit::default())
	}
}

impl AsDisplacement for MemoryOffset32Bit
{
	type D = u64;
	
	#[inline(always)]
	fn displacement(self) -> Self::D
	{
		self.get_offset().displacement()
	}
}

impl From<(SegmentRegister, Immediate64Bit)> for MemoryOffset32Bit
{
	#[inline(always)]
	fn from(value: (SegmentRegister, Immediate64Bit)) -> Self
	{
		MemoryOffset32Bit::SegmentOffsetForm32(value.0, value.1)
	}
}

impl From<(Option<SegmentRegister>, Immediate64Bit)> for MemoryOffset32Bit
{
	#[inline(always)]
	fn from(value: (Option<SegmentRegister>, Immediate64Bit)) -> Self
	{
		use self::MemoryOffset32Bit::*;
		
		match value.0
		{
			Some(segment_register) => SegmentOffsetForm32(segment_register, value.1),
			None => OffsetForm32(value.1),
		}
	}
}

impl From<Immediate64Bit> for MemoryOffset32Bit
{
	#[inline(always)]
	fn from(value: Immediate64Bit) -> Self
	{
		MemoryOffset32Bit::OffsetForm32(value)
	}
}

impl From<i8> for MemoryOffset32Bit
{
	#[inline(always)]
	fn from(value: i8) -> Self
	{
		MemoryOffset32Bit::OffsetForm32(value.into())
	}
}

impl From<u8> for MemoryOffset32Bit
{
	#[inline(always)]
	fn from(value: u8) -> Self
	{
		MemoryOffset32Bit::OffsetForm32(value.into())
	}
}

impl From<i16> for MemoryOffset32Bit
{
	#[inline(always)]
	fn from(value: i16) -> Self
	{
		MemoryOffset32Bit::OffsetForm32(value.into())
	}
}

impl From<u16> for MemoryOffset32Bit
{
	#[inline(always)]
	fn from(value: u16) -> Self
	{
		MemoryOffset32Bit::OffsetForm32(value.into())
	}
}

impl From<i32> for MemoryOffset32Bit
{
	#[inline(always)]
	fn from(value: i32) -> Self
	{
		MemoryOffset32Bit::OffsetForm32(value.into())
	}
}

impl From<u32> for MemoryOffset32Bit
{
	#[inline(always)]
	fn from(value: u32) -> Self
	{
		MemoryOffset32Bit::OffsetForm32(value.into())
	}
}

impl From<i64> for MemoryOffset32Bit
{
	#[inline(always)]
	fn from(value: i64) -> Self
	{
		MemoryOffset32Bit::OffsetForm32(value.into())
	}
}

impl From<u64> for MemoryOffset32Bit
{
	#[inline(always)]
	fn from(value: u64) -> Self
	{
		MemoryOffset32Bit::OffsetForm32(value.into())
	}
}

impl Into<Option<SegmentRegister>> for MemoryOffset32Bit
{
	#[inline(always)]
	fn into(self) -> Option<SegmentRegister>
	{
		use self::MemoryOffset32Bit::*;
		
		match self
		{
			SegmentOffsetForm32(segment_register, _) => Some(segment_register),
			OffsetForm32(_) => None,
		}
	}
}

impl Into<(Option<SegmentRegister>, Immediate64Bit)> for MemoryOffset32Bit
{
	#[inline(always)]
	fn into(self) -> (Option<SegmentRegister>, Immediate64Bit)
	{
		use self::MemoryOffset32Bit::*;
		
		match self
		{
			SegmentOffsetForm32(segment_register, immediate) => (Some(segment_register), immediate),
			OffsetForm32(immediate) => (None, immediate),
		}
	}
}

impl Into<Immediate64Bit> for MemoryOffset32Bit
{
	#[inline(always)]
	fn into(self) -> Immediate64Bit
	{
		use self::MemoryOffset32Bit::*;
		
		match self
		{
			SegmentOffsetForm32(_, immediate) => immediate,
			OffsetForm32(immediate) => immediate,
		}
	}
}

impl Into<i64> for MemoryOffset32Bit
{
	#[inline(always)]
	fn into(self) -> i64
	{
		let immediate: Immediate64Bit = self.into();
		immediate.into()
	}
}

impl Into<u64> for MemoryOffset32Bit
{
	#[inline(always)]
	fn into(self) -> u64
	{
		let immediate: Immediate64Bit = self.into();
		immediate.into()
	}
}

impl MemoryOffset for MemoryOffset32Bit
{
	#[inline(always)]
	fn get_segment_register(&self) -> Option<SegmentRegister>
	{
		use self::MemoryOffset32Bit::*;
		
		match *self
		{
			SegmentOffsetForm32(segment_register, _) => Some(segment_register),
			OffsetForm32(_) => None,
		}
	}
	
	#[inline(always)]
	fn get_offset(&self) -> Immediate64Bit
	{
		use self::MemoryOffset32Bit::*;
		
		match *self
		{
			SegmentOffsetForm32(_, immediate) => immediate,
			OffsetForm32(immediate) => immediate,
		}
	}
}
