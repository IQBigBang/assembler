// This file is part of assembler. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/assembler/master/COPYRIGHT. No part of predicator, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2018 The developers of assembler. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/assembler/master/COPYRIGHT.


/// An instruction stream.
///
/// Has functions for writing x64 instructions, organized by mnemonic and the parameters they need.
///
/// When finished, call `finish()`.
///
/// When writing 8-bit `Jcc` (`JMP` and conditional `JMP` instructions), a `ShortJmpResult` is returned in error if the target effective address could be resolved and its displacement exceeds the size of an `i8`. In this case, the instruction stream is rolled back to point to just before where the instruction started to be emitted. Use this result to try to make a 8-bit `JMP` and then replace it with a 32-bit one if an error occurs.
///
/// Note that unresolved labels (ie those yet to be attached to a location in the instruction stream) will not produce such an error. Instead a panic (in debug builds) or silent error will occur when `finish()` is called.
#[derive(Debug)]
pub struct InstructionStream<'a>
{
	byte_emitter: ByteEmitter,
	executable_anonymous_memory_map: &'a mut ExecutableAnonymousMemoryMap,
	labelled_locations: LabelledLocations,
	instruction_pointers_to_replace_labels_with_8_bit_displacements: Vec<(Label, InstructionPointer)>,
	instruction_pointers_to_replace_labels_with_32_bit_displacements: Vec<(Label, InstructionPointer)>,
}

impl<'a> InstructionStream<'a>
{
	/// `REX.W` prefix.
	const REX_W: u8 = 0x48;
	
	/// `REX.R` prefix.
	pub(crate) const REX_R: u8 = 0x44;
	
	/// `REX.X` prefix.
	pub(crate) const REX_X: u8 = 0x42;
	
	/// `REX.B` prefix.
	pub(crate) const REX_B: u8 = 0x41;
	
	/// `REX` prefix.
	pub(crate) const REX: u8 = 0x40;
	
	#[inline(always)]
	pub(crate) fn new(executable_anonymous_memory_map: &'a mut ExecutableAnonymousMemoryMap, likely_number_of_labels_hint: usize) -> Self
	{
		executable_anonymous_memory_map.make_writable();
		
		Self
		{
			byte_emitter: ByteEmitter::new(executable_anonymous_memory_map),
			executable_anonymous_memory_map,
			labelled_locations: LabelledLocations::new(likely_number_of_labels_hint),
			instruction_pointers_to_replace_labels_with_8_bit_displacements: Vec::with_capacity(likely_number_of_labels_hint),
			instruction_pointers_to_replace_labels_with_32_bit_displacements: Vec::with_capacity(likely_number_of_labels_hint),
		}
	}
	
	/// Resolves all remaining labels and makes code executable.
	///
	/// Will panic in debug builds if labels can not be resolved, 8-bit JMPs are too far away or 32-bit JMPs have displacements of more than 2Gb!
	#[inline(always)]
	pub fn finish(mut self)
	{
		for (label, insert_at_instruction_pointer) in self.instruction_pointers_to_replace_labels_with_8_bit_displacements
		{
			let target_instruction_pointer = self.labelled_locations.potential_target_instruction_pointer(label);
			debug_assert!(target_instruction_pointer.is_valid(), "unresolved label '{:?}'", label);
			let result = self.byte_emitter.insert_8_bit_effective_address_displacement(insert_at_instruction_pointer, target_instruction_pointer);
			debug_assert!(result.is_err(), "8-bit JMP for label '{:?}' was too far", label)
		}
		
		for (label, insert_at_instruction_pointer) in self.instruction_pointers_to_replace_labels_with_32_bit_displacements
		{
			let target_instruction_pointer = self.labelled_locations.potential_target_instruction_pointer(label);
			debug_assert!(target_instruction_pointer.is_valid(), "unresolved label '{:?}'", label);
			
			self.byte_emitter.insert_32_bit_effective_address_displacement(insert_at_instruction_pointer, target_instruction_pointer)
		}
		
		self.executable_anonymous_memory_map.make_executable()
	}
	
	#[inline(always)]
	fn bookmark(&mut self)
	{
		self.byte_emitter.store_bookmark()
	}
	
	/// Returns an error if displacement would exceed 8 bits.
	#[inline(always)]
	fn displacement_label_8bit(&mut self, label: Label) -> ShortJmpResult
	{
		let target_instruction_pointer = self.labelled_locations.potential_target_instruction_pointer(label);
		if target_instruction_pointer.is_valid()
		{
			let insert_at_instruction_pointer = self.byte_emitter.instruction_pointer;
			match self.byte_emitter.insert_8_bit_effective_address_displacement(insert_at_instruction_pointer, target_instruction_pointer)
			{
				Ok(()) => Ok(()),
				Err(()) =>
				{
					self.byte_emitter.reset_to_bookmark();
					Err(())
				}
			}
		}
		else
		{
			let instruction_pointer = self.instruction_pointer();
			self.instruction_pointers_to_replace_labels_with_8_bit_displacements.push((label, instruction_pointer));
			self.byte_emitter.skip_u8();
			Ok(())
		}
	}
	
	/// Does not return an error if displacement would exceed 32 bits, but panics in debug builds.
	///
	/// Errors are very unlikely indeed for such overly large displacements, are almost certainly a mistake and can not realistically be recovered from, in any event.
	#[inline(always)]
	fn displacement_label_32bit(&mut self, label: Label)
	{
		let target_instruction_pointer = self.labelled_locations.potential_target_instruction_pointer(label);
		if target_instruction_pointer.is_valid()
		{
			let insert_at_instruction_pointer = self.byte_emitter.instruction_pointer;
			self.byte_emitter.insert_32_bit_effective_address_displacement(insert_at_instruction_pointer, target_instruction_pointer)
		}
		else
		{
			let instruction_pointer = self.instruction_pointer();
			self.instruction_pointers_to_replace_labels_with_32_bit_displacements.push((label, instruction_pointer));
			self.byte_emitter.skip_u32();
		}
	}
	
	/// Creates an unique label and uses it to label the current location.
	#[inline(always)]
	pub fn create_and_attach_label(&mut self) -> Label
	{
		let label = self.create_label();
		self.attach_label(label);
		label
	}
	
	/// Creates an unique label, scoped to this instance of the Instruction Stream.
	///
	/// The label is created unattached.
	#[inline(always)]
	pub fn create_label(&mut self) -> Label
	{
		self.labelled_locations.create_label()
	}
	
	/// Labels the current location.
	///
	/// It is an error to use the same label to label more than one location (or to label the current location with the same label twice or more).
	///
	/// This only checked for in debug builds where it causes a runtime panic.
	///
	/// Labels should be created using `self.create_label()`; no checks are made for labels created with another instance and attached to this one.
	#[inline(always)]
	pub fn attach_label(&mut self, label: Label)
	{
		let instruction_pointer = self.instruction_pointer();
		self.labelled_locations.set(label, instruction_pointer)
	}
	
	/// Creates a function pointer to the current location that takes no arguments and returns a result of type `R`.
	///
	/// Resultant function will not execute (and in all likelihood cause an uncaught signal to occur) until `self.finish()` is called.
	#[inline(always)]
	pub fn nullary_function_pointer<R>(&self) -> unsafe extern "C" fn() -> R
	{
		unsafe { transmute(self.instruction_pointer()) }
	}
	
	/// Creates a function pointer to the current location that takes one argument of type `A` and returns a result of type `R`.
	///
	/// Resultant function will not execute (and in all likelihood cause an uncaught signal to occur) until `self.finish()` is called.
	#[inline(always)]
	pub fn unary_function_pointer<R, A>(&self) -> unsafe extern "C" fn(A) -> R
	{
		unsafe { transmute(self.instruction_pointer()) }
	}
	
	/// Creates a function pointer to the current location that takes two argument of types `A` and `B` and returns a result of type `R`.
	///
	/// Resultant function will not execute (and in all likelihood cause an uncaught signal to occur) until `self.finish()` is called.
	#[inline(always)]
	pub fn binary_function_pointer<R, A, B>(&self) -> unsafe extern "C" fn(A, B) -> R
	{
		unsafe { transmute(self.instruction_pointer()) }
	}
	
	/// Creates a function pointer to the current location that takes three argument of types `A`, `B` and `C` and returns a result of type `R`.
	///
	/// Resultant function will not execute (and in all likelihood cause an uncaught signal to occur) until `self.finish()` is called.
	#[inline(always)]
	pub fn ternary_function_pointer<R, A, B, C>(&self) -> unsafe extern "C" fn(A, B, C) -> R
	{
		unsafe { transmute(self.instruction_pointer()) }
	}
	
	/// Creates a function pointer to the current location that takes four argument of types `A`, `B`, `C` and `D` and returns a result of type `R`.
	///
	/// Resultant function will not execute (and in all likelihood cause an uncaught signal to occur) until `self.finish()` is called.
	#[inline(always)]
	pub fn quaternary_function_pointer<R, A, B, C, D>(&self) -> unsafe extern "C" fn(A, B, C, D) -> R
	{
		unsafe { transmute(self.instruction_pointer()) }
	}
	
	/// Creates a function pointer to the current location that takes five argument of types `A`, `B`, `C`, `D` and `E` and returns a result of type `R`.
	///
	/// Resultant function will not execute (and in all likelihood cause an uncaught signal to occur) until `self.finish()` is called.
	#[inline(always)]
	pub fn quinary_function_pointer<R, A, B, C, D, E>(&self) -> unsafe extern "C" fn(A, B, C, D, E) -> R
	{
		unsafe { transmute(self.instruction_pointer()) }
	}
	
	/// Creates a function pointer to the current location that takes six argument of types `A`, `B`, `C`, `D`, `E` and `F` and returns a result of type `R`.
	///
	/// Resultant function will not execute (and in all likelihood cause an uncaught signal to occur) until `self.finish()` is called.
	#[inline(always)]
	pub fn senary_function_pointer<R, A, B, C, D, E, F>(&self) -> unsafe extern "C" fn(A, B, C, D, E, F) -> R
	{
		unsafe { transmute(self.instruction_pointer()) }
	}
	
	/// Emits (pushes) a byte into the instruction stream at the current location.
	///
	/// The byte can be data or instructions.
	#[inline(always)]
	pub fn emit_byte(&mut self, byte: u8)
	{
		self.byte_emitter.emit_u8(byte)
	}
	
	/// Emits (pushes) a word (2 bytes) into the instruction stream at the current location.
	///
	/// The word can be data or instructions.
	///
	/// The word will be swapped into the little endian form (a no-op on x64 platforms).
	#[inline(always)]
	pub fn emit_word(&mut self, word: u16)
	{
		self.byte_emitter.emit_u16(word)
	}
	
	/// Emits (pushes) a double word (4 bytes) into the instruction stream at the current location.
	///
	/// The word can be data or instructions.
	///
	/// The word will be swapped into the little endian form (a no-op on x64 platforms).
	#[inline(always)]
	pub fn emit_double_word(&mut self, double_word: u32)
	{
		self.byte_emitter.emit_u32(double_word)
	}
	
	/// Emits (pushes) a quad word (8 bytes) into the instruction stream at the current location.
	///
	/// The word can be data or instructions.
	///
	/// The word will be swapped into the little endian form (a no-op on x64 platforms).
	#[inline(always)]
	pub fn emit_quad_word(&mut self, quad_word: u64)
	{
		self.byte_emitter.emit_u64(quad_word)
	}
	
	/// Emits (pushes) a double quad word (16 bytes) into the instruction stream at the current location.
	///
	/// The word can be data or instructions.
	///
	/// The word will be swapped into the little endian form (a no-op on x64 platforms).
	#[inline(always)]
	pub fn emit_double_quad_word(&mut self, double_quad_word: u128)
	{
		self.byte_emitter.emit_u128(double_quad_word)
	}
	
	/// Emits (pushes) zero or more bytes into the instruction stream at the current location.
	///
	/// Bytes can be data or instructions.
	#[inline(always)]
	pub fn emit_bytes(&mut self, bytes: &[u8])
	{
		self.byte_emitter.emit_bytes(bytes)
	}
	
	/// Emits (pushes) `NOP`s (No Operation) opcodes into the instruction stream at the current location to ensure the desired `alignment`.
	///
	/// Efficient for alignments up to 64 (needed for AVX-512).
	#[inline(always)]
	pub fn emit_alignment(&mut self, alignment: usize)
	{
		let offset = self.instruction_pointer() % alignment;
		
		const NOP: u8 = 0x90;
		
		match offset
		{
			0 => (),
			
			1 => self.emit_byte(0x90),
			
			2 => self.emit_word(0x9090),
			
			3 => self.emit_bytes(&[NOP, NOP, NOP]),
			
			4 => self.emit_double_word(0x90909090),
			
			5 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP]),
			
			6 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP]),
			
			7 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			8 => self.emit_quad_word(0x9090909090909090),
			
			9 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			10 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			11 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			12 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			13 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			14 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			15 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			16 => self.emit_double_quad_word(0x90909090909090909090909090909090),
			
			17 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			18 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			19 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			20 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			21 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			22 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			23 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			24 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			25 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			26 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			27 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			28 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			29 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			30 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			31 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			32 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			33 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			34 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			35 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			36 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			37 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			38 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			39 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			40 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			41 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			42 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			43 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			44 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			45 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			46 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			47 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			48 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			49 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			50 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			51 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			52 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			53 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			54 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			55 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			56 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			57 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			58 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			59 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			60 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			61 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			62 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			63 => self.emit_bytes(&[NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]),
			
			_ => for _ in 0 .. (alignment - offset)
			{
				self.emit_byte(0x90);
			},
		}
	}
	
	#[inline(always)]
	fn instruction_pointer(&self) -> InstructionPointer
	{
		self.byte_emitter.instruction_pointer
	}
	
	// See Figure 2-9, Intel Manual Volume 2A Section 2-15 (May 2018).
	#[inline(always)]
	fn vex_7(&mut self, mmmmm: u8, L: u8, pp: u8, w: u8, vvvv: impl Register, rm: impl MemoryOrRegister, r: impl Register)
	{
		rm.emit_vex_prefix(&mut self.byte_emitter, mmmmm, L, pp, w, vvvv, r)
	}
	
	// See Figure 2-9, Intel Manual Volume 2A Section 2-15 (May 2018).
	#[inline(always)]
	fn vex_5(&mut self, mmmmm: u8, L: u8, pp: u8, w: u8, vvvv: impl Register)
	{
		if mmmmm == 0x01 && w == 0
		{
			self.byte_emitter.emit_2_byte_vex_prefix(0x80, vvvv, L, pp)
		}
		else
		{
			self.byte_emitter.emit_3_byte_vex_prefix(0x80, 0x40, 0x20, mmmmm, w, vvvv, L, pp)
		}
	}
	
	#[inline(always)]
	fn prefix_fwait(&mut self, byte: u8)
	{
		self.byte_emitter.emit_u8(byte)
	}
	
	#[inline(always)]
	fn prefix_group1(&mut self, byte: u8)
	{
		self.byte_emitter.emit_u8(byte)
	}
	
	#[inline(always)]
	fn prefix_group2(&mut self, memory_operand_or_branch_hint: impl PrefixGroup2)
	{
		memory_operand_or_branch_hint.emit_prefix_group2(&mut self.byte_emitter)
	}
	
	#[inline(always)]
	fn prefix_group3(&mut self)
	{
		self.byte_emitter.emit_u8(0x66)
	}
	
	#[inline(always)]
	fn prefix_group4(&mut self, memory: impl Memory)
	{
		self.prefix_group4_if_address_override(memory.memory_operand().has_address_override_for_32_bit())
	}
	
	#[inline(always)]
	fn prefix_group4_if_address_override(&mut self, address_override_for_32_bit: bool)
	{
		if address_override_for_32_bit
		{
			const AddressOverridePrefix: u8 = 0x67;
			self.byte_emitter.emit_u8(AddressOverridePrefix)
		}
	}
	
	#[inline(always)]
	fn rex_3(&mut self, rm: impl MemoryOrRegister, r: impl Register, byte: u8)
	{
		rm.emit_rex_3(&mut self.byte_emitter, r, byte)
	}
	
	#[inline(always)]
	fn rex_2(&mut self, rm: impl MemoryOrRegister, byte: u8)
	{
		rm.emit_rex_2(&mut self.byte_emitter, byte)
	}
	
	#[inline(always)]
	fn rex_1(&mut self, byte: u8)
	{
		self.byte_emitter.emit_u8_if_not_zero(byte)
	}
	
	#[inline(always)]
	fn opcode_1(&mut self, opcode: u8)
	{
		self.byte_emitter.emit_u8(opcode)
	}
	
	#[inline(always)]
	fn opcode_2(&mut self, opcode1: u8, opcode2: impl ToOpcode)
	{
		self.opcode_1(opcode1);
		self.opcode_1(opcode2.to_opcode());
	}
	
	#[inline(always)]
	fn opcode_3(&mut self, opcode1: u8, opcode2: u8, opcode3: impl ToOpcode)
	{
		self.opcode_1(opcode1);
		self.opcode_1(opcode2);
		self.opcode_1(opcode3.to_opcode());
	}
	
	#[inline(always)]
	fn mod_rm_sib(&mut self, rm: impl MemoryOrRegister, reg: impl Register)
	{
		rm.emit_mod_rm_sib(&mut self.byte_emitter, reg)
	}
	
	#[inline(always)]
	fn displacement_immediate_1<D: Displacement>(&mut self, displacement: impl AsDisplacement<D=D>)
	{
		displacement.displacement().emit(&mut self.byte_emitter)
	}
	
	#[inline(always)]
	fn displacement_immediate_2(&mut self, displacement1: Immediate8Bit, displacement2: Immediate16Bit)
	{
		// NOTE: This order is correct, with the second displacement emitted before the first.
		self.displacement_immediate_1(displacement2);
		self.displacement_immediate_1(displacement1);
	}
}

include!("InstructionStream.instructions.rs");
