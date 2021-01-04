// This file is part of assembler. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/assembler/master/COPYRIGHT. No part of predicator, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2018 The developers of assembler. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/assembler/master/COPYRIGHT.


/// An immediate value.
pub trait Immediate: From<i8> + From<u8>
{
	/// Zero.
	const Zero: Self;
	
	/// One.
	const One: Self;
	
	/// Minimum.
	const Minimum: Self;
	
	/// Maximum.
	const Maximum: Self;
	
	/// Signed integer type of the underlying value.
	type SignedInteger;
	
	/// Underlying signed value.
	#[inline(always)]
	fn value(self) -> Self::SignedInteger;
}
