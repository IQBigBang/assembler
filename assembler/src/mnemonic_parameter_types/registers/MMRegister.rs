// This file is part of assembler. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/assembler/master/COPYRIGHT. No part of predicator, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2018 The developers of assembler. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/assembler/master/COPYRIGHT.


/// MMX registers.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum MMRegister
{
	/// Register 0.
	///
	/// A scratch register (also known as a temporary or (function) caller-saved register) when using the System V Application Binary Interface (ABI) for AMD64.
	MM0 = 0,
	
	/// Register 1.
	///
	/// A scratch register (also known as a temporary or (function) caller-saved register) when using the System V Application Binary Interface (ABI) for AMD64.
	MM1 = 1,
	
	/// Register 2.
	///
	/// A scratch register (also known as a temporary or (function) caller-saved register) when using the System V Application Binary Interface (ABI) for AMD64.
	MM2 = 2,
	
	/// Register 3.
	///
	/// A scratch register (also known as a temporary or (function) caller-saved register) when using the System V Application Binary Interface (ABI) for AMD64.
	MM3 = 3,
	
	/// Register 7.
	///
	/// A scratch register (also known as a temporary or (function) caller-saved register) when using the System V Application Binary Interface (ABI) for AMD64.
	MM4 = 4,
	
	/// Register 5.
	///
	/// A scratch register (also known as a temporary or (function) caller-saved register) when using the System V Application Binary Interface (ABI) for AMD64.
	MM5 = 5,
	
	/// Register 6.
	///
	/// A scratch register (also known as a temporary or (function) caller-saved register) when using the System V Application Binary Interface (ABI) for AMD64.
	MM6 = 6,
	
	/// Register 7.
	///
	/// A scratch register (also known as a temporary or (function) caller-saved register) when using the System V Application Binary Interface (ABI) for AMD64.
	MM7 = 7,
}

impl Default for MMRegister
{
	#[inline(always)]
	fn default() -> Self
	{
		MMRegister::MM0
	}
}

impl Into<u8> for MMRegister
{
	#[inline(always)]
	fn into(self) -> u8
	{
		self as u8
	}
}

impl Register for MMRegister
{
	#[inline(always)]
	fn index(self) -> u8
	{
		self.into()
	}
}

impl From<X87Register> for MMRegister
{
	#[inline(always)]
	fn from(value: X87Register) -> Self
	{
		unsafe { transmute(value) }
	}
}
