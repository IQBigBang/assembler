// This file is part of assembler. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/assembler/master/COPYRIGHT. No part of predicator, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2017 The developers of assembler. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/assembler/master/COPYRIGHT.


/// A register.
pub trait Register: MemoryOrRegister
{
	const IsRegister8Bit: bool = false;
	
	/// Index.
	#[inline(always)]
	fn index(self) -> u8;
	
	#[inline(always)]
	fn requires_rex_byte(self) -> bool
	{
		Self::IsRegister8Bit && self.index() > 3
	}
	
	#[inline(always)]
	fn requires_rex_bit(self) -> bool
	{
		self.index() > 7
	}
}

impl<R: Register> MemoryOrRegister for R
{
	#[inline(always)]
	fn emit_mod_rm_sib(self, byte_emitter: &mut ByteEmitter, reg: impl Register)
	{
		const ModRegisterAddressingMode: u8 = 0b11;
		
		let rm = self;
		let mod_rm_and_sib = (ModRegisterAddressingMode << 6) | ((reg.index() << 3) & 0b0011_1000) | (rm.index() & 0x07);
		byte_emitter.emit_u8(mod_rm_and_sib)
	}
	
	#[inline(always)]
	fn emit_rex_3(self, byte_emitter: &mut ByteEmitter, r: impl Register, mut byte: u8)
	{
		byte |= if r.requires_rex_byte()
		{
			OrdinaryInstructionStream::REX
		}
		else
		{
			0x00
		};
		
		byte |= if r.requires_rex_bit()
		{
			OrdinaryInstructionStream::REX_R
		}
		else
		{
			0x00
		};
		
		self.emit_rex_3(byte_emitter, rm, byte);
	}
	
	#[inline(always)]
	fn emit_rex_2(self, byte_emitter: &mut ByteEmitter, mut byte: u8)
	{
		let self = rm;
		
		byte |= if rm.requires_rex_byte()
		{
			OrdinaryInstructionStream::REX
		}
		else
		{
			0x00
		};
		
		byte |= if rm.requires_rex_bit()
		{
			OrdinaryInstructionStream::REX_B
		}
		else
		{
			0x00
		};
		
		byte_emitter.emit_u8_if_not_zero(byte);
	}
}
