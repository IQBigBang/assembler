// This file is part of assembler. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/assembler/master/COPYRIGHT. No part of predicator, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2018 The developers of assembler. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/assembler/master/COPYRIGHT.


/// Register 0 (`ST(0)`) is defined on this enumeration but optimal encodings of it in combination with mnemonics exist and should be preferred as they increase code density.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum X87Register
{
	/// Register `ST(0)`.
	ST0 = 0,
	
	/// Register `ST(1)`.
	ST1 = 1,
	
	/// Register `ST(2)`.
	ST2 = 2,
	
	/// Register `ST(3)`.
	ST3 = 3,
	
	/// Register `ST(4)`.
	ST4 = 4,
	
	/// Register `ST(5)`.
	ST5 = 5,
	
	/// Register `ST(6)`.
	ST6 = 6,
	
	/// Register `ST(7)`.
	ST7 = 7,
}

impl Default for X87Register
{
	#[inline(always)]
	fn default() -> Self
	{
		X87Register::ST0
	}
}

impl ToOpcode for X87Register
{
	#[inline(always)]
	fn to_opcode(self) -> u8
	{
		self.index() & 0x07
	}
}

impl Register for X87Register
{
	#[inline(always)]
	fn index(self) -> u8
	{
		self as u8
	}
}